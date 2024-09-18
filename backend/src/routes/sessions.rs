use core::panic;
use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext, WsResponseBuilder};
use actix_web::{web::Payload, error::ErrorInternalServerError, Error, HttpRequest, HttpResponse};
use log::{error, info};
use crate::utils::{communication::RunnerRequest, runner::Runner};

struct SessionWsActor {
    runner: Runner
}

impl Actor for SessionWsActor {
    type Context = WebsocketContext<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Err(err) = self.runner.delete() {
            error!("Error deleting runner: {err:?}");
            // TODO: implement recovery for deleting runners?
            panic!();
        }

        info!("Runner {} deleted because the connection was closed.", self.runner.hash());
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for SessionWsActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => {
                ctx.text(match RunnerRequest::try_from(&text.to_string()) {
                    Ok(request) => request.act(&self.runner).to_string(),
                    Err(err) => err.to_string()
                })
            },
            Ok(Message::Ping(ping)) => {
                ctx.pong(&ping);
            },
            _ => {}
        }
    }
}

pub async fn session_ws(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    let runner = match Runner::create() {
        Ok(runner) => {
            info!(
                "Runner {} created for connection coming from: \"{}\".",
                runner.hash(),
                req.peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or("unknown".to_string())
            );
            runner
        },
        Err(err) => {
            error!("{err:?}");
            return Err(ErrorInternalServerError(err));
        }
    };

    let (_actor, response) = WsResponseBuilder::new(SessionWsActor { runner }, &req, stream)
        .start_with_addr()?;

    Ok(response)
}
