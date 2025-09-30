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
    web::{Payload, Query},
};
use actix_ws::{AggregatedMessage, handle};
use futures_util::StreamExt as _;
use libbarto::parse_ts_ping;
use tracing::{error, info, trace};

use crate::endpoints::insecure::Name;

pub(crate) async fn worker(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
) -> Result<impl Responder> {
    info!("worker connection from {}", name.describe(&request));
    let (response, mut session, msg_stream) = handle(&request, body)?;
    let mut agms = msg_stream.aggregate_continuations();
    let _handle = spawn(async move {
        while let Some(Ok(msg)) = agms.next().await {
            match msg {
                AggregatedMessage::Text(_byte_string) => todo!(),
                AggregatedMessage::Binary(_bytes) => todo!(),
                AggregatedMessage::Ping(bytes) => {
                    trace!("handling ping message");
                    if let Some(dur) = parse_ts_ping(&bytes) {
                        trace!("ping duration: {}s", dur.as_secs_f64());
                    }
                    // self.hb = Instant::now();
                    if let Err(e) = session.pong(&bytes).await {
                        error!("unable to send pong: {e}");
                    }
                }
                AggregatedMessage::Pong(bytes) => {
                    trace!("handling pong message");
                    if let Some(dur) = parse_ts_ping(&bytes) {
                        trace!("pong duration: {}s", dur.as_secs_f64());
                    }
                    // self.hb = Instant::now();
                }
                AggregatedMessage::Close(close_reason) => {
                    info!("close received: {:?}", close_reason);
                    break;
                }
            }
        }

        info!("websocket disconnected");
        let _ = session.close(None).await;
    });

    Ok(response)
}
