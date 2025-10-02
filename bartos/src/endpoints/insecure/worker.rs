// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use actix_web::{
    HttpRequest, Responder, Result,
    rt::spawn,
    web::{Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, handle};
use futures_util::StreamExt as _;
use libbarto::parse_ts_ping;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

use crate::{config::Config, endpoints::insecure::Name};

pub(crate) async fn worker(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
    token: Data<CancellationToken>,
    config: Data<Config>,
) -> Result<impl Responder> {
    info!("worker connection from {}", name.describe(&request));
    let (response, session, msg_stream) = handle(&request, body)?;
    let mut agms = msg_stream.aggregate_continuations();
    let ws_token = token.get_ref().clone();
    let mut ws_session = session.clone();
    let schedule = name
        .name()
        .as_ref()
        .and_then(|name| config.schedules().get(name));
    info!("worker schedule: {schedule:?}");
    let _handle = spawn(async move {
        loop {
            select! {
                () = ws_token.cancelled() => {
                    info!("cancellation token triggered, closing websocket");
                    let _ = ws_session.close(None).await;
                    break;
                }
                res = agms.next() => {
                    if let Some(Ok(msg)) = res {
                        match msg {
                            AggregatedMessage::Text(_byte_string) => error!("unexpected text message"),
                            AggregatedMessage::Binary(_bytes) => todo!(),
                            AggregatedMessage::Ping(bytes) => {
                                trace!("handling ping message");
                                if let Some(dur) = parse_ts_ping(&bytes) {
                                    trace!("ping duration: {}s", dur.as_secs_f64());
                                }
                                if let Err(e) = ws_session.pong(&bytes).await {
                                    error!("unable to send pong: {e}");
                                }
                            }
                            AggregatedMessage::Pong(bytes) => {
                                trace!("handling pong message");
                                if let Some(dur) = parse_ts_ping(&bytes) {
                                    trace!("pong duration: {}s", dur.as_secs_f64());
                                }
                            }
                            AggregatedMessage::Close(close_reason) => {
                                trace!("handling close message");
                                if let Some(cr) = &close_reason {
                                    let code = u16::from(cr.code);
                                    if let Some(desc) = &cr.description {
                                        trace!("close reason: code={code} reason={desc}");
                                    } else {
                                        trace!("close reason: code={code} no reason given");
                                    }
                                } else {
                                    trace!("close reason: none");
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        info!("websocket disconnected");
        let _ = session.close(None).await;
    });

    Ok(response)
}
