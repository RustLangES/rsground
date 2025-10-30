mod client;
mod server;

use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use futures::StreamExt;
use itertools::Itertools;
use rsground_runner::hakoniwa_ext::HakoniwaChildExt;
use rsground_runner::lsp::notification::{Initialized, Notification};
use rsground_runner::lsp::request::{Initialize, Request};
use rsground_runner::lsp::{
    InitializedParams, LspId, LspInput, LspNotify, LspOutput, LspRequest, LspResponse,
};
use rsground_runner::{PipeWriter, Runner};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::utils::{define_local_logger, ArcStr, Truncate};
use crate::ws::messages::InternalMessage;

use super::project_runner::{start_job, AbortSender};

define_local_logger!(local_log as "backend::lsp" {
    client_stdin,
    initialize,
    notify => "notification",
    request,
    response,
    stderr,
    stdin,
    stdout,
});

const LSP_INITIALIZATION_ID: &str = "rsground::initialize";

pub struct ProjectLsp {
    broadcast: broadcast::Sender<InternalMessage>,
    instance: Option<(AbortSender, PipeWriter)>,
    project_id: Uuid,
    queue: VecDeque<Stdin>,
    ready: bool,
    runner: Arc<Runner>,
}

impl ProjectLsp {
    pub async fn start(
        project_id: Uuid,
        broadcast: broadcast::Sender<InternalMessage>,
        runner: Arc<Runner>,
    ) -> Addr<Self> {
        let project_lsp = Self {
            broadcast,
            instance: None,
            project_id,
            queue: VecDeque::new(),
            ready: false,
            runner,
        };

        project_lsp.start()
    }
}

impl Actor for ProjectLsp {
    type Context = Context<ProjectLsp>;
}

pub trait ProjectLspActor {
    fn send_request<T: Request, const QUEUE: bool>(
        &self,
        id: impl serde::Serialize,
        params: &T::Params,
    );
    fn send_notify<T: Notification, const QUEUE: bool>(&self, params: &T::Params);
}

impl ProjectLspActor for Addr<ProjectLsp> {
    fn send_request<T: Request, const QUEUE: bool>(
        &self,
        id: impl serde::Serialize,
        params: &T::Params,
    ) {
        match LspInput::request::<T>(id, params) {
            Ok(req) => self.do_send(Stdin::<QUEUE>(req)),
            Err(err) => {
                local_log::stdin::error!("Serialize: {err:?}");
            }
        }
    }

    fn send_notify<T: Notification, const QUEUE: bool>(&self, params: &T::Params) {
        match LspInput::notify::<T>(params) {
            Ok(req) => self.do_send(Stdin::<QUEUE>(req)),
            Err(err) => {
                local_log::stdin::error!("Serialize: {err:?}");
            }
        }
    }
}

// Client-side messages

#[derive(Message)]
#[rtype(result = "()")]
pub struct Execute;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stdin<const QUEUE: bool = true>(String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientStdin(pub ArcStr, pub serde_json::Value);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Kill;

impl Handler<Execute> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, _: Execute, ctx: &mut Self::Context) -> Self::Result {
        if self.instance.is_some() {
            local_log::warn!(target: self.project_id, "rust-analyzer already started");
            return;
        }

        local_log::debug!(target: self.project_id, "Starting");

        let Ok((child, stdin, stdout, stderr)) = self.runner.start_rls() else {
            local_log::error!(target: self.project_id, "Failed to start rust-analyzer");
            return;
        };

        start_job(
            self,
            ctx,
            move |abort, this, ctx| {
                this.instance.replace((abort, stdin));

                ctx.address().send_request::<Initialize, false>(
                    LSP_INITIALIZATION_ID,
                    &*client::LSP_INITIALIZATION,
                );
            },
            async move |abort| child.wait_or_abort(abort).await,
            |status, this, _| {
                local_log::debug!(target: this.project_id, "Exit status: {status:?}");
                drop(this.instance.take());
            },
        );

        ctx.add_stream(stdout.into_lsp());
        ctx.add_stream(stderr.stream::<2048>().map(Stderr));
    }
}

impl Handler<Kill> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, _: Kill, _: &mut Self::Context) -> Self::Result {
        self.ready = false;
        if let Some((abort, _)) = self.instance.take() {
            _ = abort.send(());
        }
    }
}

impl<const QUEUE: bool> Handler<Stdin<QUEUE>> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, Stdin(msg): Stdin<QUEUE>, _: &mut Self::Context) -> Self::Result {
        if !QUEUE || self.ready {
            if let Some((_, stdin)) = self.instance.as_mut() {
                let msg = format!("Content-Length: {}\r\n\r\n{msg}", msg.len());
                if let Err(err) = stdin.write_all(msg.as_bytes()).and_then(|_| stdin.flush()) {
                    local_log::stdin::error!(target: self.project_id, "Cannot write: {err}")
                }
            }
        } else {
            self.queue.push_back(Stdin(msg));
        }
    }
}

impl Handler<ClientStdin> for ProjectLsp {
    type Result = ();

    fn handle(
        &mut self,
        ClientStdin(client_id, msg): ClientStdin,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let Ok(msg) = serde_json::from_value::<LspOutput>(msg).map_err(
            |err| local_log::client_stdin::error!(target: self.project_id, "Deserialize: {err:?}"),
        ) else {
            return;
        };

        // Handle initialization
        match msg {
            LspOutput::Request(LspRequest { id, ref method, .. })
                if method == Initialize::METHOD =>
            {
                _ = self.broadcast.send(InternalMessage::Lsp {
                    client_id: Some(client_id.into()),
                    data: LspInput::response_value::<Initialize>(
                        id,
                        Ok(&*server::LSP_INITIALIZATION),
                    )
                    .expect("Empty params should not fail"),
                });
                return;
            }
            LspOutput::Notify(LspNotify { ref method, .. }) if method == Initialized::METHOD => {
                return
            }
            _ => {}
        }

        macro_rules! with_id {
            ($id:expr, |$inner:ident| $body:expr) => {
                serde_json::to_string($id)
                    .map(|id| LspId::String(format!("client_id::{client_id}::{id}").into()))
                    .map(|$inner| $body)
            };
        }

        let msg = match msg {
            LspOutput::Response(LspResponse::Ok {
                jsonrpc,
                id,
                result,
            }) => with_id!(&id, |id| LspOutput::Response(LspResponse::Ok {
                id,
                jsonrpc,
                result,
            })),
            LspOutput::Response(LspResponse::Err { jsonrpc, id, error }) => {
                with_id!(&id, |id| LspOutput::Response(LspResponse::Err {
                    id,
                    jsonrpc,
                    error,
                }))
            }
            LspOutput::Request(req) => {
                with_id!(&req.id, |id| LspOutput::Request(LspRequest { id, ..req }))
            }
            notify @ LspOutput::Notify(..) => Ok(notify),
        };

        let Ok(msg) = msg
            .and_then(|msg| serde_json::to_string(&msg))
            .map_err(|err| {
                local_log::client_stdin::error!(
                    target: self.project_id,
                    "Cannot serialize: {err}"
                )
            })
        else {
            return;
        };

        ctx.notify(Stdin::<true>(msg));
    }
}

// Internal-side messages

type Stdout = Result<LspOutput, io::Error>;

impl StreamHandler<Stdout> for ProjectLsp {
    fn handle(&mut self, msg: Stdout, ctx: &mut Self::Context) {
        let Ok(msg) = msg.map_err(|err| local_log::error!(target: self.project_id, "{err:?}"))
        else {
            return;
        };

        match msg {
            LspOutput::Response(res) if res.id() == LSP_INITIALIZATION_ID => match res {
                LspResponse::Ok { result, .. } => {
                    local_log::initialize::info!(target: self.project_id, "Initialize received");
                    local_log::initialize::trace!(target: self.project_id, "{:?}", result.truncate::<256>());

                    self.ready = true;

                    if let Ok(noti) = LspInput::notify::<Initialized>(&InitializedParams {}) {
                        ctx.notify(Stdin::<false>(noti));
                    }

                    while let Some(msg) = self.queue.pop_front() {
                        ctx.notify(msg);
                    }
                }
                LspResponse::Err { error, .. } => {
                    local_log::initialize::error!(target: self.project_id, "{error:?}");
                    ctx.notify(Kill);
                }
            },

            LspOutput::Response(res) => {
                local_log::response::trace!(target: self.project_id, "{:?}", (&res).truncate::<256>());
                let (client_id, res_id) = res
                    .id()
                    .clone()
                    .into_string()
                    .and_then(|s| {
                        s.strip_prefix("client_id::")
                            .and_then(|s| s.splitn(2, "::").collect_tuple::<(_, _)>())
                            .map(|(client_id, original_id)| {
                                (
                                    Arc::from(client_id),
                                    serde_json::from_str::<LspId>(original_id).inspect_err(|err| {
                                        local_log::response::error!(
                                            target: self.project_id,
                                            "Cannot deserialize original_id ({original_id}): {err}"
                                        )
                                    }),
                                )
                            })
                    })
                    .and_then(|(client_id, original_id)| {
                        original_id.ok().map(|original_id| (client_id, original_id))
                    })
                    .unzip();

                // Update res id if required to recover the original id
                let res = match res_id {
                    None => res,
                    Some(id) => match res {
                        LspResponse::Ok {
                            result, jsonrpc, ..
                        } => LspResponse::Ok {
                            id,
                            result,
                            jsonrpc,
                        },
                        LspResponse::Err { error, jsonrpc, .. } => {
                            LspResponse::Err { id, error, jsonrpc }
                        }
                    },
                };

                match serde_json::to_value(res) {
                    Ok(data) => {
                        _ = self
                            .broadcast
                            .send(InternalMessage::Lsp { client_id, data });
                    }
                    Err(err) => {
                        local_log::response::error!(target: self.project_id, "Cannot serialize {err:?}");
                    }
                }
            }
            LspOutput::Request(req) => {
                local_log::request::trace!(target: self.project_id, "{req:?}");
                match serde_json::to_value(req) {
                    Ok(data) => {
                        _ = self.broadcast.send(InternalMessage::Lsp {
                            client_id: None,
                            data,
                        });
                    }
                    Err(err) => {
                        local_log::request::error!(target: self.project_id, "Cannot serialize {err:?}");
                    }
                }
            }
            LspOutput::Notify(noti) => {
                local_log::notify::trace!(target: self.project_id, "{noti:?}");
                match serde_json::to_value(noti) {
                    Ok(data) => {
                        _ = self.broadcast.send(InternalMessage::Lsp {
                            client_id: None,
                            data,
                        });
                    }
                    Err(err) => {
                        local_log::response::error!(target: self.project_id, "Cannot serialize {err:?}");
                    }
                }
            }
        }
    }
}

struct Stderr(Result<Vec<u8>, io::Error>);

impl StreamHandler<Stderr> for ProjectLsp {
    fn handle(&mut self, Stderr(msg): Stderr, _: &mut Self::Context) {
        let Ok(msg) = msg
            .map_err(|err| local_log::stderr::error!(target: self.project_id, "{err:?}"))
            .map(String::from_utf8)
            .and_then(|s| {
                s.map_err(|err| local_log::stderr::error!(target: self.project_id, "Deserialize: {err:?}"))
            })
        else {
            return;
        };

        const TIMESTAMP_LEN: usize = "YYYY-MM-DDTHH:mm:ss".len();

        // if has space exacly where timestamp ends
        let has_timestamp = msg
            .get(..TIMESTAMP_LEN)
            .is_some_and(|s| s.chars().all(|c| c != '.'))
            && msg
                .get(TIMESTAMP_LEN..TIMESTAMP_LEN + 1)
                .is_some_and(|c| c == ".");

        let msg = if has_timestamp {
            msg.chars()
                .skip(TIMESTAMP_LEN)
                .skip_while(|c| *c != ' ')
                .skip(1)
                .collect()
        } else {
            msg
        };

        local_log::stderr::warn!(target: self.project_id, "{msg:?}");
    }
}
