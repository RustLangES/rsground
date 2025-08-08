use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use rsground_runner::Runner;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::ws::messages::{OutputChannel, ServerMessage};

use super::project_runner::{start_job, AbortSender};

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
            log::trace!("Execute started for {}", self.project_id);

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

        async move |mut stdio| {
            let buf = &mut [0; 2048];

            loop {
                let Ok(size) = stdio.read(buf).await else {
                    log::error!(concat!("Cannot read ", stringify!($channel)));
                    break;
                };

                if size == 0 {
                    break;
                }

                // log::trace!(concat!(stringify!($channel), ": {:x?}"), &buf[..size]);
                _ = broadcast.send(ServerMessage::SyncOutput {
                    channel: $crate::ws::messages::OutputChannel::$channel,
                    buf: buf[..size].to_vec(),
                });
            }
        }
    }};
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
                log::trace!("[Producer] compiling in {}", this.project_id);
                this.instance.replace(abort);
            },
            async move |abort| {
                let (status, _, _) = Runner::stream_output(
                    &mut runner.cmd_rustc(["--color", "always", "/home/main.rs"]),
                    stream!(broadcast, Stdout),
                    stream!(broadcast, Stderr),
                    Some(abort),
                )
                .await
                .map_err(|err| {
                    log::error!("[Producer] compilation failed in {project_id}: {err}");
                    _ = broadcast.send(ServerMessage::SyncOutput {
                        channel: OutputChannel::Stderr,
                        buf: err.to_string().into_bytes(),
                    });
                    _ = broadcast.send(ServerMessage::SyncOutputEnd { exit_code: 126 });
                })?;

                if !status.success() {
                    log::error!("[Producer] compilation failed in {project_id}");
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
                log::trace!("[Producer] patching in {}", this.project_id);
                this.instance.replace(abort);
            },
            async move |_| {
                let output = runner.patch_binary("/home/main").await.map_err(|err| {
                    log::error!("[Producer] patching failed in {project_id}: {err}");
                    _ = broadcast.send(ServerMessage::SyncOutput {
                        channel: OutputChannel::Stderr,
                        buf: err.to_string().into_bytes(),
                    });
                    _ = broadcast.send(ServerMessage::SyncOutputEnd { exit_code: 126 });
                })?;

                if !output.status.success() {
                    log::error!("[Producer] patch failed in {project_id}: {output:#?}");
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
                log::trace!("[Producer] running in {}", this.project_id);
                this.instance.replace(abort);
            },
            async move |abort| {
                let (exit_code, _, _) = Runner::stream_output(
                    &mut runner.cmd("/home/main", [] as [&str; 0]),
                    stream!(broadcast, Stdout),
                    stream!(broadcast, Stderr),
                    Some(abort),
                )
                .await
                .map_err(|err| {
                    log::trace!("[Producer] run failed in {project_id}: {err}");
                    _ = broadcast.send(ServerMessage::SyncOutput {
                        channel: OutputChannel::Stderr,
                        buf: err.to_string().into_bytes(),
                    });
                    _ = broadcast.send(ServerMessage::SyncOutputEnd { exit_code: 126 });
                })?;

                log::trace!("[Producer] finish in {project_id}");

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
