// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::BTreeMap;

use actix_web::{
    HttpRequest, Responder, Result,
    error::ErrorInternalServerError,
    rt::spawn,
    web::{Bytes, Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, Session, handle};
use bincode_next::{config::standard, decode_from_slice, encode_to_vec};
use futures_util::StreamExt as _;
use libbarto::{
    Bartoc, BartosToBartoc, Initialize, Output, OutputKind, OutputTableName, Schedules, Status,
    StatusTableName, UuidWrapper, hmac_sign, parse_hmac_key, parse_signing_key, parse_ts_ping,
    sign_payload,
};
use sqlx::MySqlPool;
use tokio::{
    select,
    sync::{Mutex, RwLock, broadcast},
    time::{Duration, Instant, interval},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use uuid::Uuid;

use crate::{
    common::{Clients, WorkerSignal},
    config::Config,
    endpoints::insecure::{Name, bearer_auth_ok},
};

#[allow(clippy::too_many_arguments)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn worker(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
    token: Data<CancellationToken>,
    config: Data<Config>,
    pool: Data<MySqlPool>,
    clients: Data<Mutex<Clients>>,
    live_schedules: Data<RwLock<BTreeMap<String, Schedules>>>,
    worker_bcast: Data<broadcast::Sender<WorkerSignal>>,
) -> Result<impl Responder> {
    let describe = name.describe(&request);
    info!("worker connection from '{describe}'");
    if !bearer_auth_ok(&request, config.api_key().as_deref()) {
        info!("worker connection from '{describe}' rejected: missing or invalid Bearer token");
        return Err(actix_web::error::ErrorUnauthorized("unauthorized"));
    }
    let id = Uuid::new_v4();
    let (response, session, msg_stream) = handle(&request, body)?;
    let mut agms = msg_stream.aggregate_continuations();
    let ws_token = token.get_ref().clone();
    let mut ws_session = session.clone();
    let mut init_session = session.clone();
    let config_c = config.clone();
    let clients_c = clients.clone();
    // Capture client name before it is moved into initialize()
    let client_name = name.name();
    let mut worker_rx = worker_bcast.subscribe();
    if let Err(e) = initialize(id, &mut init_session, request, name, config, clients).await {
        error!("unable to initialize worker session: {e}");
        let _ = init_session.close(None).await;
        return Err(e);
    }

    let _handle = spawn(async move {
        const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
        const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
        let mut last_heartbeat = Instant::now();
        let mut hb_interval = interval(HEARTBEAT_INTERVAL);
        loop {
            select! {
                () = ws_token.cancelled() => {
                    trace!("cancellation token triggered, closing websocket");
                    let _ = ws_session.close(None).await;
                    break;
                }
                _ = hb_interval.tick() => {
                    if last_heartbeat.elapsed() > CLIENT_TIMEOUT {
                        error!("client '{describe}' heartbeat timed out, disconnecting");
                        break;
                    }
                }
                res = agms.next() => {
                    match res {
                        Some(Ok(msg)) => {
                            last_heartbeat = Instant::now();
                            if handle_ws_msg(id, msg, &config_c, pool.as_ref(), clients_c.clone(), &mut ws_session).await {
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            error!("websocket error: {e}");
                            break;
                        }
                        None => {
                            trace!("websocket stream closed");
                            break;
                        }
                    }
                }
                signal = worker_rx.recv() => {
                    match signal {
                        Ok(WorkerSignal::Reload) => {
                            let schedules_guard = live_schedules.read().await;
                            let schedules = schedules_guard.get(&client_name).cloned();
                            drop(schedules_guard);
                            let init_bytes = build_init_bytes(id, schedules, &config_c);
                            if let Err(e) = ws_session.binary(init_bytes).await {
                                error!("unable to send updated schedules to '{describe}': {e}");
                            } else {
                                info!("sent updated schedules to '{describe}'");
                            }
                        }
                        Ok(WorkerSignal::Cleanup) => {
                            let cleanup_bytes = build_cleanup_bytes(&config_c);
                            if let Err(e) = ws_session.binary(cleanup_bytes).await {
                                error!("unable to send cleanup signal to '{describe}': {e}");
                            } else {
                                info!("sent cleanup signal to '{describe}'");
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        info!("websocket disconnected '{describe}'");
        let _ = session.close(None).await;
        let mut clients = clients_c.lock().await;
        let _old = clients.remove_client(&id);
        trace!("removed client '{describe}' from active clients");
    });

    Ok(response)
}

/// Returns `true` if the loop should break (connection close or unrecoverable error).
#[cfg_attr(coverage_nightly, coverage(off))]
async fn handle_ws_msg(
    id: Uuid,
    msg: AggregatedMessage,
    config: &Config,
    pool: &MySqlPool,
    clients: Data<Mutex<Clients>>,
    ws_session: &mut Session,
) -> bool {
    match msg {
        AggregatedMessage::Text(_) => error!("unexpected text message"),
        AggregatedMessage::Binary(bytes) => {
            handle_binary(id, bytes, config, pool, clients)
                .await
                .unwrap_or_else(|e| {
                    error!("unable to handle binary message: {e}");
                });
        }
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
            return true;
        }
    }
    false
}

/// Builds and returns the encoded (and optionally signed) Initialize payload.
/// Returns an empty vec when there are no schedules for this client.
fn build_init_bytes(id: Uuid, schedules: Option<Schedules>, config: &Config) -> Vec<u8> {
    let Some(schedules) = schedules else {
        return vec![];
    };
    let count = schedules.schedules().len();
    trace!("building initialize payload with {count} schedules");
    let uuid = UuidWrapper(id);
    let init = Initialize::builder().id(uuid).schedules(schedules).build();
    let payload = match encode_to_vec(BartosToBartoc::Initialize(init), standard()) {
        Ok(p) => p,
        Err(e) => {
            error!("unable to encode initialize message: {e}");
            return vec![];
        }
    };
    sign_worker_payload(payload, config)
}

/// Builds and returns the encoded (and optionally signed) Cleanup payload, asking the
/// worker to clean up old entries from its local redb database.
fn build_cleanup_bytes(config: &Config) -> Vec<u8> {
    trace!("building cleanup payload");
    let payload = match encode_to_vec(BartosToBartoc::Cleanup, standard()) {
        Ok(p) => p,
        Err(e) => {
            error!("unable to encode cleanup message: {e}");
            return vec![];
        }
    };
    sign_worker_payload(payload, config)
}

/// Applies the optional HMAC-SHA256 envelope and Ed25519 signature to a worker-bound
/// payload, matching the auth configuration. Shared by all bartos → bartoc messages.
fn sign_worker_payload(payload: Vec<u8>, config: &Config) -> Vec<u8> {
    let payload = if let Some(hmac_key_str) = config.hmac_key() {
        trace!("wrapping worker message with HMAC-SHA256 envelope");
        hmac_sign(&parse_hmac_key(hmac_key_str), &payload)
    } else {
        payload
    };
    if let Some(sk_b64) = config.signing_key() {
        match parse_signing_key(sk_b64) {
            Ok(sk) => {
                trace!("signing worker message with Ed25519 key");
                sign_payload(&sk, &payload)
            }
            Err(e) => {
                error!("invalid signing key, sending unsigned: {e}");
                payload
            }
        }
    } else {
        payload
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn initialize(
    id: Uuid,
    session: &mut Session,
    request: HttpRequest,
    name: Query<Name>,
    config: Data<Config>,
    clients: Data<Mutex<Clients>>,
) -> Result<()> {
    let describe = name.describe(&request);
    let mut clients = clients.lock().await;
    let old_opt = clients.remove_client_by_name(&name.name());
    if let Some(_old) = old_opt {
        info!("removed old client with same name '{}'", name.name());
    }
    let _old = clients.add_client(id, &name.name(), &Name::ip(&request));
    let name = name.name();
    let schedules_opt = config.schedules().get(&name).cloned();
    let init_bytes = build_init_bytes(id, schedules_opt, &config);
    if !init_bytes.is_empty() {
        let count = config
            .schedules()
            .get(&name)
            .map_or(0, |s| s.schedules().len());
        info!("sending bartoc '{describe}' {count} schedules");
    }
    session.binary(init_bytes).await.map_err(|e| {
        error!("unable to send initialization message: {e}");
        ErrorInternalServerError("internal server error")
    })?;
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn handle_binary(
    id: Uuid,
    bytes: Bytes,
    config: &Config,
    pool: &MySqlPool,
    clients_mutex: Data<Mutex<Clients>>,
) -> Result<()> {
    trace!("handling binary message");
    match decode_from_slice(&bytes, standard()) {
        Err(e) => error!("unable to decode binary message: {e}"),
        Ok((bartoc_msg, _)) => match bartoc_msg {
            Bartoc::Record(data) => match data {
                libbarto::Data::Output(output) => match config.mariadb().output_table() {
                    OutputTableName::Output => {
                        trace!("handling output data: {}", output);
                        let _id = insert_output(pool, &output).await.unwrap_or_else(|e| {
                            error!("unable to insert output into database: {e}");
                            0
                        });
                    }
                    OutputTableName::OutputTest => {
                        trace!("handling output data: {}", output);
                        let _id = insert_output_test(pool, &output).await.unwrap_or_else(|e| {
                            error!("unable to insert output into database: {e}");
                            0
                        });
                    }
                },
                libbarto::Data::Status(status) => match config.mariadb().status_table() {
                    StatusTableName::Status => {
                        trace!("handling status data: {}", status);
                        let _id = insert_status(pool, &status).await.unwrap_or_else(|e| {
                            error!("unable to insert status into database: {e}");
                            0
                        });
                    }
                    StatusTableName::StatusTest => {
                        trace!("handling status data: {}", status);
                        let _id = insert_status_test(pool, &status).await.unwrap_or_else(|e| {
                            error!("unable to insert status into database: {e}");
                            0
                        });
                    }
                },
            },
            Bartoc::ClientInfo(bi) => {
                info!("received client info: {bi}");
                let mut clients = clients_mutex.lock().await;
                clients.add_client_data(&id, bi);
            }
        },
    }
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn insert_output(pool: &MySqlPool, output: &Output) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO output (bartoc_uuid, bartoc_name, cmd_uuid, cmd_name, timestamp, kind, data)
VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        output.bartoc_uuid().0,
        output.bartoc_name(),
        output.cmd_uuid().0,
        output.cmd_name(),
        output.timestamp().0,
        <OutputKind as Into<&'static str>>::into(output.kind()),
        output.data()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn insert_output_test(pool: &MySqlPool, output: &Output) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO output_test (bartoc_uuid, bartoc_name, cmd_uuid, cmd_name, timestamp, kind, data)
VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        output.bartoc_uuid().0,
        output.bartoc_name(),
        output.cmd_uuid().0,
        output.cmd_name(),
        output.timestamp().0,
        <OutputKind as Into<&'static str>>::into(output.kind()),
        output.data()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn insert_status(pool: &MySqlPool, status: &Status) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO exit_status (cmd_uuid, timestamp, exit_code, success)
VALUES (?, ?, ?, ?)"#,
        status.cmd_uuid().0,
        status.timestamp().0,
        status.exit_code(),
        status.success()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn insert_status_test(pool: &MySqlPool, status: &Status) -> anyhow::Result<u64> {
    let id = sqlx::query!(
        r#"INSERT INTO exit_status_test (cmd_uuid, timestamp, exit_code, success)
VALUES (?, ?, ?, ?)"#,
        status.cmd_uuid().0,
        status.timestamp().0,
        status.exit_code(),
        status.success()
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(id)
}

#[cfg(test)]
mod tests {
    use bincode_next::{config::standard, decode_from_slice};
    use libbarto::{BartosToBartoc, Schedules};
    use uuid::Uuid;

    use super::{build_cleanup_bytes, build_init_bytes, sign_worker_payload};
    use crate::config::Config;

    fn empty_schedules() -> Schedules {
        // `Schedules` only derives `Builder` under libbarto's own test cfg, so build
        // it through its `Deserialize` impl here.
        serde_json::from_str(r#"{"schedules":[]}"#).expect("deserialize Schedules")
    }

    #[test]
    fn build_init_bytes_none_is_empty() {
        let bytes = build_init_bytes(Uuid::new_v4(), None, &Config::default());
        assert!(bytes.is_empty());
    }

    #[test]
    fn build_init_bytes_some_is_non_empty() {
        let bytes = build_init_bytes(Uuid::new_v4(), Some(empty_schedules()), &Config::default());
        assert!(!bytes.is_empty());
    }

    #[test]
    fn build_cleanup_bytes_round_trips() {
        // Default config has no HMAC/signing configured, so the payload is the raw
        // encoded message and decodes straight back.
        let bytes = build_cleanup_bytes(&Config::default());
        let (decoded, _): (BartosToBartoc, _) =
            decode_from_slice(&bytes, standard()).expect("decode");
        assert!(matches!(decoded, BartosToBartoc::Cleanup));
    }

    #[test]
    fn sign_worker_payload_passthrough_without_auth() {
        let payload = vec![1_u8, 2, 3, 4];
        assert_eq!(
            sign_worker_payload(payload.clone(), &Config::default()),
            payload
        );
    }
}
