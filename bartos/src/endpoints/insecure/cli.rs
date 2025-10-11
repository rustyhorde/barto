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
    web::{Bytes, Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, Session, handle};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use futures_util::StreamExt as _;
use libbarto::{BartoCli, BartosToBartoCli};
use sqlx::MySqlPool;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use vergen_pretty::{Pretty, PrettyExt, vergen_pretty_env};

use crate::{config::Config, endpoints::insecure::Name};

pub(crate) async fn cli(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
    token: Data<CancellationToken>,
    _config: Data<Config>,
    _pool: Data<MySqlPool>,
) -> Result<impl Responder> {
    let describe = name.describe(&request);
    info!("cli connection from '{describe}'");
    let ws_token = token.get_ref().clone();
    let (response, session, msg_stream) = handle(&request, body)?;
    let mut ws_session = session.clone();
    let mut agms = msg_stream.aggregate_continuations();

    let _handle = spawn(async move {
        loop {
            select! {
                () = ws_token.cancelled() => {
                    trace!("cancellation token triggered, closing websocket");
                    let _ = ws_session.close(None).await;
                    break;
                }
                res_opt = agms.next() => {
                    if let Some(Ok(msg)) = res_opt {
                        match msg {
                            AggregatedMessage::Text(_byte_string) => error!("unexpected text message"),
                            AggregatedMessage::Binary(bytes) => if let Err(e) = handle_binary(bytes, &mut ws_session).await {
                                error!("{e}");
                            },
                            AggregatedMessage::Ping(_bytes) => error!("unexpected ping message"),
                            AggregatedMessage::Pong(_bytes) => error!("unexpected pong message"),
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
        info!("websocket disconnected '{describe}'");
        let _ = session.close(None).await;
    });

    Ok(response)
}

async fn handle_binary(bytes: Bytes, session: &mut Session) -> anyhow::Result<()> {
    match decode_from_slice(&bytes, standard()) {
        Err(e) => error!("unable to decode binary message: {e}"),
        Ok((msg, _)) => match msg {
            BartoCli::Info => {
                info!("received info message");
                let pretty = Pretty::builder().env(vergen_pretty_env!()).build();
                let pretty_ext = PrettyExt::from(pretty);
                let info = BartosToBartoCli::Info(pretty_ext);
                let encoded = encode_to_vec(&info, standard())?;
                session.binary(encoded).await?;
            }
            BartoCli::Updates { name } => {
                info!("received updates message for '{name}'");
                let updates = BartosToBartoCli::Updates;
                let encoded = encode_to_vec(&updates, standard())?;
                session.binary(encoded).await?;
            }
        },
    }
    Ok(())
}
