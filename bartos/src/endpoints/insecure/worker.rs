// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use actix_web::{
    HttpRequest, Responder, Result,
    error::ErrorInternalServerError,
    rt::spawn,
    web::{Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, Session, handle};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use futures_util::StreamExt as _;
use libbarto::{
    Bartoc, BartosToBartoc, Initialize, Output, OutputKind, Status, UuidWrapper, parse_ts_ping,
};
use sqlx::MySqlPool;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use uuid::Uuid;

use crate::{config::Config, endpoints::insecure::Name};

pub(crate) async fn worker(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
    token: Data<CancellationToken>,
    config: Data<Config>,
    pool: Data<MySqlPool>,
) -> Result<impl Responder> {
    info!("worker connection from {}", name.describe(&request));
    let (response, session, msg_stream) = handle(&request, body)?;
    let mut agms = msg_stream.aggregate_continuations();
    let ws_token = token.get_ref().clone();
    let mut ws_session = session.clone();
    let mut init_session = session.clone();
    if let Err(e) = initialize(&mut init_session, name, config).await {
        error!("unable to initialize worker session: {e}");
        let _ = init_session.close(None).await;
        return Err(e);
    }

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
                            AggregatedMessage::Binary(bytes) => {
                                if let Ok((bartoc_msg, _)) = decode_from_slice(&bytes, standard()) {
                                    match bartoc_msg {
                                        Bartoc::Record(data) => {
                                            match data {
                                                libbarto::Data::Output(output) => {
                                                    info!("handling output data: {}", output);
                                                    let _id = insert_output(&pool, &output).await.unwrap_or_else(|e| {
                                                        error!("unable to insert output into database: {e}");
                                                        0
                                                    });
                                                }
                                                libbarto::Data::Status(status) => {
                                                    info!("handling status data: {}", status);
                                                    let _id = insert_status(&pool, &status).await.unwrap_or_else(|e| {
                                                        error!("unable to insert status into database: {e}");
                                                        0
                                                    });
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    error!("unable to decode binary message");
                                }
                            },
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

async fn initialize(session: &mut Session, name: Query<Name>, config: Data<Config>) -> Result<()> {
    let name = name.name().clone().unwrap_or_else(|| "default".to_string());
    let schedules_opt = config.schedules().get(&name);
    let init_bytes = if let Some(schedules) = schedules_opt {
        let count = schedules.schedules().len();
        info!("sending bartoc '{}' {} schedules", name, count);
        let uuid = UuidWrapper(Uuid::new_v4());
        let init = Initialize::builder()
            .id(uuid)
            .schedules(schedules.clone())
            .build();
        encode_to_vec(BartosToBartoc::Initialize(init), standard()).map_err(|e| {
            error!("unable to encode initialization message: {e}");
            ErrorInternalServerError("internal server error")
        })?
    } else {
        vec![]
    };
    session.binary(init_bytes).await.map_err(|e| {
        error!("unable to send initialization message: {e}");
        ErrorInternalServerError("internal server error")
    })?;
    Ok(())
}

async fn insert_output(pool: &MySqlPool, output: &Output) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO output (bartoc_uuid, bartoc_name, cmd_uuid, timestamp, kind, data)
VALUES (?, ?, ?, ?, ?, ?)"#,
        output.bartoc_uuid().0,
        output.bartoc_name(),
        output.cmd_uuid().0,
        output.timestamp().0,
        <OutputKind as Into<&'static str>>::into(output.kind()),
        output.data()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}

async fn insert_status(pool: &MySqlPool, status: &Status) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO exit_status (cmd_uuid, exit_code, success)
VALUES (?, ?, ?)"#,
        status.cmd_uuid().0,
        status.exit_code(),
        status.success()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}
