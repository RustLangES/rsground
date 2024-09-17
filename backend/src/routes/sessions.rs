use actix_web::{rt::spawn, web::Payload, Error, HttpRequest, HttpResponse};
use actix_ws::{handle, AggregatedMessage};
use futures_util::StreamExt as _;

pub async fn session_ws(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(_)) => {
                    session.text("received").await.unwrap();
                },
                Ok(AggregatedMessage::Ping(ping)) => {
                    session.pong(&ping).await.unwrap();
                }
                _ => (),
            }
        }
    });

    Ok(res)
}
