// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

use actix_web::{
    HttpRequest, Responder, Result,
    rt::spawn,
    web::{Bytes, Data, Payload, Query},
};
use actix_ws::{AggregatedMessage, Session, handle};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use futures_util::StreamExt as _;
use libbarto::{
    BartoCli, BartosToBartoCli, CliUpdateKind, ClientData, Garuda, OutputTableName, UpdateKind,
    UuidWrapper,
};
use regex::Regex;
use sqlx::{Column, MySqlPool, Row};
use time::{
    OffsetDateTime,
    macros::{offset, time},
};
use tokio::{select, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use uuid::Uuid;
use vergen_pretty::{Pretty, PrettyExt, vergen_pretty_env};

use crate::{common::Clients, config::Config, endpoints::insecure::Name};

// pacman
// Packages (5) git-2.51.1-1  libarchive-3.8.2-1  linux-6.17.3.arch1-1  python-charset-normalizer-3.4.4-1  python-cryptography-46.0.3-1
// Packages (2) dhcpcd-10.2.4-1  libxml2-2.15.1-1
//
// Total Download Size:   0.96 MiB
// Total Installed Size:  3.45 MiB
// Net Upgrade Size:      0.00 MiB

// apt
// The following packages will be upgraded:
//   libtdb-dev libtdb1"
// 2 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.

// homebrew
// ==> Upgrading 2 outdated packages:
// protobuf 32.1 -> 33.0
// ==> Fetching downloads for: openexr and protobuf
// ==> Fetching openexr
// ==> Downloading https://ghcr.io/v2/homebrew/core/openexr/blobs/sha256:6e4279cef58092ba7d95c6f805b77ca4a8e3420010b0093d17d5ce058b749fd7
// ==> Fetching protobuf
// ==> Downloading https://ghcr.io/v2/homebrew/core/protobuf/blobs/sha256:220d0c9358fda8b85ce23cbb53596547f63895480c16498a6d3b8031710d4b21
// ==> Upgrading openexr
// "  3.4.1 -> 3.4.2 "
// ==> Pouring openexr--3.4.2.arm64_sequoia.bottle.tar.gz
// 🍺  /opt/homebrew/Cellar/openexr/3.4.2: 212 files, 4.9MB
// Removing: /opt/homebrew/Cellar/openexr/3.4.1... (212 files, 4.9MB)
// "  32.1 -> 33.0 "
// ==> Pouring protobuf--33.0.arm64_sequoia.bottle.tar.gz
// 🍺  /opt/homebrew/Cellar/protobuf/33.0: 364 files, 16.5MB

#[allow(dead_code)]
static GARUDA_UPDATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(\d+/\d+\) upgrading ([^ ]+) .*").expect("failed to create garuda update regex")
});
static GARUDA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(chaotic-aur|extra)\/([^ ]+)\s+([^ ]+)\s+([^ ]+)\s+(.+ MiB)\s+(.+ MiB)")
        .expect("failed to create chaotic aur regex")
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
            BartoCli::Info { json } => {
                info!("received info message");
                let pretty = Pretty::builder().env(vergen_pretty_env!());

                let btbc: BartosToBartoCli = if json {
                    let new_pretty = pretty.flatten(true);
                    BartosToBartoCli::InfoJson(serde_json::to_string(&new_pretty.build())?)
                } else {
                    let pretty_ext = PrettyExt::from(pretty.build());
                    BartosToBartoCli::Info(pretty_ext)
                };
                let encoded = encode_to_vec(&btbc, standard())?;
                session.binary(encoded).await?;
            }
            BartoCli::Updates { name, kind } => {
                let msg = match kind {
                    CliUpdateKind::Garuda => {
                        info!("received updates message for '{name}' (garuda)");
                        let updates = garuda_update_data(&name, config, pool).await?;
                        BartosToBartoCli::Updates(UpdateKind::Garuda(updates))
                    }
                    CliUpdateKind::Other => {
                        info!("received updates message for '{name}' (other)");
                        BartosToBartoCli::Updates(UpdateKind::Other)
                    }
                };
                let encoded = encode_to_vec(&msg, standard())?;
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
                    .collect::<HashMap<UuidWrapper, ClientData>>();
                let clients = BartosToBartoCli::Clients(mapped_clients);
                let encoded = encode_to_vec(&clients, standard())?;
                session.binary(encoded).await?;
            }
            BartoCli::Query { query } => {
                info!("received query message");
                let results = sqlx::query(&query).fetch_all(pool).await?;
                let mut map = BTreeMap::new();
                for (i, row) in results.iter().enumerate() {
                    let mut row_map = BTreeMap::new();
                    for (j, column) in row.columns().iter().enumerate() {
                        if let Ok(value) = row.try_get::<u64, usize>(j) {
                            let _old = row_map.insert(column.name().to_string(), value.to_string());
                        } else if let Ok(value) = row.try_get::<OffsetDateTime, usize>(j) {
                            let value = value.to_offset(offset!(-4));
                            let _old = row_map.insert(column.name().to_string(), value.to_string());
                        } else if let Ok(value) = row.try_get::<String, usize>(j) {
                            let _old = row_map.insert(column.name().to_string(), value);
                        } else if let Ok(value) = row.try_get::<Uuid, usize>(j) {
                            let _old = row_map.insert(column.name().to_string(), value.to_string());
                        }
                    }
                    let _old = map.insert(i, row_map);
                }
                info!("query returned {} rows", map.len());
                let query_result = BartosToBartoCli::Query(map);
                let encoded = encode_to_vec(&query_result, standard())?;
                session.binary(encoded).await?;
            }
        },
    }
    Ok(())
}

async fn garuda_update_data(
    name: &str,
    config: &Config,
    pool: &MySqlPool,
) -> anyhow::Result<Vec<Garuda>> {
    match config.mariadb().output_table() {
        OutputTableName::Output => output_data(name, pool).await,
        OutputTableName::OutputTest => output_test_data(name, pool).await,
    }
}

async fn output_data(name: &str, pool: &MySqlPool) -> anyhow::Result<Vec<Garuda>> {
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
            GARUDA_RE.captures(&s).map(|caps| {
                Garuda::builder()
                    .channel(caps.get(1).map_or("", |m| m.as_str()))
                    .package(caps.get(2).map_or("", |m| m.as_str()))
                    .old_version(caps.get(3).map_or("", |m| m.as_str()))
                    .new_version(caps.get(4).map_or("", |m| m.as_str()))
                    .size_change(caps.get(5).map_or("", |m| m.as_str()))
                    .download_size(caps.get(6).map_or("", |m| m.as_str()))
                    .build()
            })
        })
        .collect::<Vec<Garuda>>();
    results.sort();
    Ok(results)
}

async fn output_test_data(name: &str, pool: &MySqlPool) -> anyhow::Result<Vec<Garuda>> {
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
            GARUDA_RE.captures(&s).map(|caps| {
                Garuda::builder()
                    .channel(caps.get(1).map_or("", |m| m.as_str()))
                    .package(caps.get(2).map_or("", |m| m.as_str()))
                    .old_version(caps.get(3).map_or("", |m| m.as_str()))
                    .new_version(caps.get(4).map_or("", |m| m.as_str()))
                    .size_change(caps.get(5).map_or("", |m| m.as_str()))
                    .download_size(caps.get(6).map_or("", |m| m.as_str()))
                    .build()
            })
        })
        .collect::<Vec<Garuda>>();
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
    let midnight = midnight()?;
    let output_count = sqlx::query!("DELETE FROM output WHERE timestamp < ?", midnight)
        .execute(pool)
        .await?
        .rows_affected();
    let exit_status_count = sqlx::query!("DELETE FROM exit_status WHERE timestamp < ?", midnight)
        .execute(pool)
        .await?
        .rows_affected();
    Ok((output_count, exit_status_count))
}

async fn delete_output_test_data(pool: &MySqlPool) -> anyhow::Result<(u64, u64)> {
    let midnight = midnight()?;
    let output_count = sqlx::query!("DELETE FROM output_test WHERE timestamp < ?", midnight)
        .execute(pool)
        .await?
        .rows_affected();
    let exit_status_count =
        sqlx::query!("DELETE FROM exit_status_test WHERE timestamp < ?", midnight)
            .execute(pool)
            .await?
            .rows_affected();
    Ok((output_count, exit_status_count))
}

fn midnight() -> anyhow::Result<OffsetDateTime> {
    let now = OffsetDateTime::now_local()?;
    let midnight = now.replace_time(time!(0:0:0));
    info!("deleting records older than: {midnight}");
    Ok(midnight)
}

#[cfg(test)]
mod test {
    use crate::endpoints::insecure::cli::GARUDA_RE;

    use super::GARUDA_UPDATE_RE;

    use anyhow::Result;

    const NO_MATCH: &str = "this is not a match";

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
        assert!(!GARUDA_UPDATE_RE.is_match(NO_MATCH));
    }

    #[test]
    fn test_package_update_re() -> Result<()> {
        let text = "extra/kio    6.19.0-1     6.19.0-2       0.00 MiB       3.59 MiB";
        assert!(GARUDA_RE.is_match(text));
        let caps = GARUDA_RE
            .captures(text)
            .ok_or(anyhow::anyhow!("failed to capture"))?;
        assert_eq!(caps.get(1).map(|m| m.as_str()), Some("extra"));
        assert_eq!(caps.get(2).map(|m| m.as_str()), Some("kio"));
        assert_eq!(caps.get(3).map(|m| m.as_str()), Some("6.19.0-1"));
        assert_eq!(caps.get(4).map(|m| m.as_str()), Some("6.19.0-2"));
        assert_eq!(caps.get(5).map(|m| m.as_str()), Some("0.00 MiB"));
        assert_eq!(caps.get(6).map(|m| m.as_str()), Some("3.59 MiB"));
        Ok(())
    }
}
