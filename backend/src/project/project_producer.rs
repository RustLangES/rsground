use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use futures::{StreamExt, TryStreamExt};
use rsground_runner::Runner;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::ws::messages::{OutputChannel, ServerMessage};

use super::define_local_logger;
use super::project_runner::{start_job, AbortSender};

define_local_logger! {local_log as "backend::producer" {
    compile,
    patch,
    run,
    stream,
}}

pub struct ProjectProducer {
    pub project_id: Uuid,
    pub broadcast: broadcast::Sender<ServerMessage>,
    pub runner: Arc<Runner>,
    pub instance: Option<AbortSender>,
}

impl ProjectProducer {
    pub async fn create(
        project_id: Uuid,
        broadcast: broadcast::Sender<ServerMessage>,
        runner: Arc<Runner>,
    ) -> Addr<Self> {
        let project_executer = Self {
            project_id,
            broadcast,
            runner,
            instance: None,
        };

        project_executer.start()
    }
}

impl Actor for ProjectProducer {
    type Context = Context<ProjectProducer>;
}

// Client-side messages

#[derive(Message)]
#[rtype(result = "()")]
pub struct Execute;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Abort;

impl Handler<Execute> for ProjectProducer {
    type Result = ();

    fn handle(&mut self, _: Execute, ctx: &mut Self::Context) -> Self::Result {
        if !self.instance.is_some() {
            local_log::trace!(target: self.project_id, "Started");

            _ = self.broadcast.send(ServerMessage::SyncOutputStart);

            ctx.notify(Compile);
        }
    }
}

impl Handler<Abort> for ProjectProducer {
    type Result = ();

    fn handle(&mut self, _: Abort, _: &mut Self::Context) -> Self::Result {
        if let Some(instance) = self.instance.take() {
            _ = instance.send(());
        }
    }
}

// Internal messages

#[derive(Message)]
#[rtype(result = "()")]
struct Compile;

#[derive(Message)]
#[rtype(result = "()")]
struct Patch;

#[derive(Message)]
#[rtype(result = "()")]
struct Run;

macro_rules! stream {
    ($broadcast:ident, $channel:ident) => {{
        let broadcast = $broadcast.clone();

        async move |stdio| {
            stdio
                .stream::<1024>()
                .map_ok(|msg| {
                    broadcast.send(ServerMessage::SyncOutput {
                        channel: $crate::ws::messages::OutputChannel::$channel,
                        buf: msg,
                    })
                })
                .map_err(|err| {
                    log::error!(concat!("Cannot read ", stringify!($channel), ": {}"), err)
                })
                .for_each(async |_| {})
                .await;
        }
    }};

    (@err $mod:ident, $broadcast:ident, $project_id:ident) => {|err| {
        local_log::$mod::error!(target: $project_id, "Failed: {err}");
        _ = $broadcast.send(ServerMessage::SyncOutput {
            channel: OutputChannel::Stderr,
            buf: err.to_string().into_bytes(),
        });
        _ = $broadcast.send(ServerMessage::SyncOutputEnd { exit_code: 126 });
    }}
}

impl Handler<Compile> for ProjectProducer {
    type Result = ();

    fn handle(&mut self, _: Compile, ctx: &mut Self::Context) -> Self::Result {
        let project_id = self.project_id;
        let broadcast = self.broadcast.clone();
        let runner = self.runner.clone();

        start_job(
            self,
            ctx,
            |abort, this, _| {
                local_log::compile::trace!(target: this.project_id, "Started");
                this.instance.replace(abort);
            },
            async move |abort| {
                let (status, _, _) = Runner::stream_output(
                    &mut runner.cmd_bash("cargo", ["build", "--color", "always"]),
                    stream!(broadcast, Stdout),
                    stream!(broadcast, Stderr),
                    Some(abort),
                )
                .await
                .map_err(stream!(@err compile, broadcast, project_id))?;

                if !status.success() {
                    local_log::compile::error!(target: project_id, "Failed");
                    _ = broadcast.send(ServerMessage::SyncOutputEnd {
                        exit_code: status.code as u8,
                    });
                    return Err(());
                }

                Ok(())
            },
            |result: Result<(), ()>, this, ctx| {
                if result.is_ok() {
                    ctx.notify(Patch);
                } else {
                    // Stop execution and unlock producer
                    drop(this.instance.take());
                }
            },
        );
    }
}

impl Handler<Patch> for ProjectProducer {
    type Result = ();

    fn handle(&mut self, _: Patch, ctx: &mut Self::Context) -> Self::Result {
        let project_id = self.project_id;
        let broadcast = self.broadcast.clone();
        let runner = self.runner.clone();

        start_job(
            self,
            ctx,
            |abort, this, _| {
                local_log::patch::trace!(target: this.project_id, "Started");
                this.instance.replace(abort);
            },
            async move |_| {
                let output = runner
                    .patch_binary("/home/target/debug/rsground-main")
                    .await
                    .map_err(stream!(@err patch, broadcast, project_id))?;

                if !output.status.success() {
                    local_log::patch::error!(target: project_id, "Failed: {output:#?}");
                    _ = broadcast.send(ServerMessage::SyncOutput {
                        channel: OutputChannel::Stdout,
                        buf: output.stdout,
                    });
                    _ = broadcast.send(ServerMessage::SyncOutput {
                        channel: OutputChannel::Stderr,
                        buf: output.stderr,
                    });
                    _ = broadcast.send(ServerMessage::SyncOutputEnd { exit_code: 126 });

                    return Err(());
                }

                Ok(())
            },
            |result, this, ctx| {
                if result.is_ok() {
                    ctx.notify(Run);
                } else {
                    // Stop execution and unlock producer
                    drop(this.instance.take());
                }
            },
        );
    }
}

impl Handler<Run> for ProjectProducer {
    type Result = ();

    fn handle(&mut self, _: Run, ctx: &mut Self::Context) -> Self::Result {
        let project_id = self.project_id;
        let broadcast = self.broadcast.clone();
        let runner = self.runner.clone();

        start_job(
            self,
            ctx,
            |abort, this, _| {
                local_log::run::trace!(target: this.project_id, "Started");
                this.instance.replace(abort);
            },
            async move |abort| {
                let (exit_code, _, _) = Runner::stream_output(
                    &mut runner.cmd("/home/target/debug/rsground-main", [] as [&str; 0]),
                    stream!(broadcast, Stdout),
                    stream!(broadcast, Stderr),
                    Some(abort),
                )
                .await
                .map_err(stream!(@err run, broadcast, project_id))?;

                local_log::run::trace!(target: project_id, "Finish");

                _ = broadcast.send(ServerMessage::SyncOutputEnd {
                    exit_code: exit_code.code as u8,
                });

                Ok(())
            },
            |_: Result<(), ()>, this, _| {
                // Stop execution and unlock producer
                drop(this.instance.take());
            },
        );
    }
}
