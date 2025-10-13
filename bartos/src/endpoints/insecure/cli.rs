// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{collections::HashMap, sync::LazyLock};

use actix_web::{
    HttpRequest, Responder, Result,
    rt::spawn,
    web::{Bytes, Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, Session, handle};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use futures_util::StreamExt as _;
use libbarto::{BartoCli, BartosToBartoCli, OutputTableName, UuidWrapper};
use regex::Regex;
use sqlx::MySqlPool;
use tokio::{select, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use vergen_pretty::{Pretty, PrettyExt, vergen_pretty_env};

use crate::{common::Clients, config::Config, endpoints::insecure::Name};

#[allow(dead_code)]
static GARUDA_UPDATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(\d+/\d+\) upgrading ([^ ]+) .*").expect("failed to create garuda update regex")
});

pub(crate) async fn cli(
    request: HttpRequest,
    body: Payload,
    name: Query<Name>,
    token: Data<CancellationToken>,
    config: Data<Config>,
    pool: Data<MySqlPool>,
    clients_mutex: Data<Mutex<Clients>>,
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
                            AggregatedMessage::Binary(bytes) => if let Err(e) = handle_binary(
                                    bytes,
                                    &mut ws_session,
                                    config.as_ref(),
                                    pool.as_ref(),
                                    clients_mutex.clone(),
                                ).await {
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

async fn handle_binary(
    bytes: Bytes,
    session: &mut Session,
    config: &Config,
    pool: &MySqlPool,
    clients_mutex: Data<Mutex<Clients>>,
) -> anyhow::Result<()> {
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
                let updates = select_data(&name, config, pool).await?;
                info!("received updates message for '{name}'");
                let updates = BartosToBartoCli::Updates(updates);
                let encoded = encode_to_vec(&updates, standard())?;
                session.binary(encoded).await?;
            }
            BartoCli::Cleanup => {
                info!("received cleanup message");
                let counts = delete_data(config, pool).await?;
                info!("deleted {} output rows", counts.0);
                info!("deleted {} exit status rows", counts.1);
                let cleanup = BartosToBartoCli::Cleanup(counts);
                let encoded = encode_to_vec(&cleanup, standard())?;
                session.binary(encoded).await?;
            }
            BartoCli::Clients => {
                info!("received clients message");
                let clients = clients_mutex.lock().await;
                let mapped_clients = clients
                    .clients()
                    .iter()
                    .map(|c| (UuidWrapper(*c.0), c.1.clone()))
                    .collect::<HashMap<UuidWrapper, String>>();
                let clients = BartosToBartoCli::Clients(mapped_clients);
                let encoded = encode_to_vec(&clients, standard())?;
                session.binary(encoded).await?;
            }
        },
    }
    Ok(())
}

async fn select_data(name: &str, config: &Config, pool: &MySqlPool) -> anyhow::Result<Vec<String>> {
    match config.mariadb().output_table() {
        OutputTableName::Output => output_data(name, pool).await,
        OutputTableName::OutputTest => output_test_data(name, pool).await,
    }
}

async fn output_data(name: &str, pool: &MySqlPool) -> anyhow::Result<Vec<String>> {
    let records = sqlx::query!(
        r#"SELECT output.data FROM output WHERE output.bartoc_name = ? order by timestamp"#,
        name,
    )
    .fetch_all(pool)
    .await?;

    let mut results = records
        .into_iter()
        .map(|r| r.data)
        .filter_map(|s| {
            GARUDA_UPDATE_RE
                .captures(&s)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        })
        .collect::<Vec<String>>();
    results.sort();
    Ok(results)
}

async fn output_test_data(name: &str, pool: &MySqlPool) -> anyhow::Result<Vec<String>> {
    let records = sqlx::query!(
        r#"SELECT output_test.data FROM output_test WHERE output_test.bartoc_name = ? order by timestamp"#,
        name,
    )
    .fetch_all(pool)
    .await?;

    let mut results = records
        .into_iter()
        .map(|r| r.data)
        .filter_map(|s| {
            GARUDA_UPDATE_RE
                .captures(&s)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        })
        .collect::<Vec<String>>();
    results.sort();
    Ok(results)
}

async fn delete_data(config: &Config, pool: &MySqlPool) -> anyhow::Result<(u64, u64)> {
    match config.mariadb().output_table() {
        OutputTableName::Output => delete_output_data(pool).await,
        OutputTableName::OutputTest => delete_output_test_data(pool).await,
    }
}

async fn delete_output_data(pool: &MySqlPool) -> anyhow::Result<(u64, u64)> {
    midnight(pool).await?;
    let output_count =
        sqlx::query!("DELETE FROM output WHERE timestamp < timestamp(subdate(utc_date(), 1))")
            .execute(pool)
            .await?
            .rows_affected();
    let exit_status_count =
        sqlx::query!("DELETE FROM exit_status WHERE timestamp < timestamp(subdate(utc_date(), 1))")
            .execute(pool)
            .await?
            .rows_affected();
    Ok((output_count, exit_status_count))
}

async fn delete_output_test_data(pool: &MySqlPool) -> anyhow::Result<(u64, u64)> {
    midnight(pool).await?;
    let output_count =
        sqlx::query!("DELETE FROM output_test WHERE timestamp < timestamp(subdate(utc_date(), 1))")
            .execute(pool)
            .await?
            .rows_affected();
    let exit_status_count = sqlx::query!(
        "DELETE FROM exit_status_test WHERE timestamp < timestamp(subdate(utc_date(), 1))"
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok((output_count, exit_status_count))
}

async fn midnight(pool: &MySqlPool) -> anyhow::Result<()> {
    let midnight = sqlx::query!("SELECT timestamp(subdate(utc_date(), 1)) as ts")
        .fetch_one(pool)
        .await?;
    if let Some(ts) = midnight.ts {
        info!("deleting entries older than {}", ts);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::GARUDA_UPDATE_RE;

    #[test]
    fn test_garuda_update_re() {
        let text = "(1/1) upgrading something blah de dah";
        assert!(GARUDA_UPDATE_RE.is_match(text));
        GARUDA_UPDATE_RE
            .captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .map(|s| assert_eq!(s, "something"))
            .expect("failed to capture");
    }

    #[test]
    fn test_garuda_update_re_no_match() {
        let text = "this is not a match";
        assert!(!GARUDA_UPDATE_RE.is_match(text));
    }
}
