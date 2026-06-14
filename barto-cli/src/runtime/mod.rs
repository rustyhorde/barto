// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;
mod secrets;

use std::{
    ffi::OsString,
    io::{Write, stdout},
    sync::Arc,
};

use anyhow::{Context as _, Result};
use bincode_next::{config::standard, encode_to_vec};
use clap::Parser as _;
use futures_util::{SinkExt as _, StreamExt as _};
use libbarto::{
    BartoCli, CliUpdateKind, header, init_tracing, load, load_client_cert_and_key,
    load_pinned_root_store,
};
use tokio_tungstenite::{
    Connector, connect_async_tls_with_config,
    tungstenite::{Message, client::ClientRequestBuilder, http::Uri},
};
use tracing::trace;

use crate::{config::Config, error::Error, handler::Handler, runtime::cli::Commands};

use self::cli::Cli;

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗        ██████╗██╗     ██╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗      ██╔════╝██║     ██║
██████╔╝███████║██████╔╝   ██║   ██║   ██║█████╗██║     ██║     ██║
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║╚════╝██║     ██║     ██║
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝      ╚██████╗███████╗██║
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝        ╚═════╝╚══════╝╚═╝";

#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn run<I, T>(args: Option<I>) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Parse the command line
    let cli = if let Some(args) = args {
        Cli::try_parse_from(args)?
    } else {
        Cli::try_parse()?
    };

    // Secrets commands are handled locally — no config load or bartos connection.
    if let Commands::Secrets(ref args) = *cli.command() {
        return secrets::handle(&args.command);
    }

    // Load the configuration
    let mut config = load::<Cli, Config, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;

    // Display the bartoc header
    let writer: Option<&mut dyn Write> = if config.enable_std_output() {
        Some(&mut stdout())
    } else {
        None
    };
    header::<Config, dyn Write>(&config, HEADER_PREFIX, writer)?;

    // Initialize tracing
    let _ = config.set_enable_std_output(true);
    init_tracing(&config, config.tracing().file(), &cli, None)
        .with_context(|| Error::TracingInit)?;

    trace!("configuration loaded");
    trace!("tracing initialized");

    let url = format!(
        "{}://{}:{}/v1/ws/cli?name={}",
        config.bartos().prefix(),
        config.bartos().host(),
        config.bartos().port(),
        config.name()
    );
    trace!("connecting to bartos at {url}");
    let uri: Uri = url.parse()?;
    let ws_req = if let Some(token) = config.bartos().api_key() {
        trace!("adding Bearer auth header to WebSocket upgrade");
        ClientRequestBuilder::new(uri).with_header("Authorization", format!("Bearer {token}"))
    } else {
        ClientRequestBuilder::new(uri)
    };
    let (ws_stream, _) =
        connect_async_tls_with_config(ws_req, None, false, Some(make_tls_connector(&config)?))
            .await?;
    trace!("websocket connected");
    let (mut sink, stream) = ws_stream.split();
    let mut handler = Handler::builder().stream(stream).build();

    sink.send(build_message(cli.command())?).await?;
    trace!("message sent");

    handler.handle().await?;

    sink.send(Message::Close(None)).await?;
    trace!("close sent");
    handler.wait_for_close().await;
    Ok(())
}

fn build_message(command: &Commands) -> Result<Message> {
    let payload = match command {
        // Secrets are handled before reaching this point — see run().
        Commands::Secrets(_) => unreachable!("secrets handled before build_message"),
        Commands::Info { json } => encode_to_vec(BartoCli::Info { json: *json }, standard())?,
        Commands::Updates { name, update_kind } => {
            let kind = CliUpdateKind::try_from(update_kind.as_str())?;
            encode_to_vec(
                BartoCli::Updates {
                    name: name.clone(),
                    kind,
                },
                standard(),
            )?
        }
        Commands::Cleanup => encode_to_vec(BartoCli::Cleanup, standard())?,
        Commands::Clients { versions } => {
            if *versions {
                encode_to_vec(BartoCli::ClientVersions, standard())?
            } else {
                encode_to_vec(BartoCli::Clients, standard())?
            }
        }
        Commands::Query { query } => encode_to_vec(
            BartoCli::Query {
                query: query.clone(),
            },
            standard(),
        )?,
        Commands::List { name, cmd_name_opt } => {
            if let Some(cmd_name) = cmd_name_opt {
                encode_to_vec(
                    BartoCli::List {
                        name: name.clone(),
                        cmd_name: cmd_name.clone(),
                    },
                    standard(),
                )?
            } else {
                encode_to_vec(BartoCli::ListCommands { name: name.clone() }, standard())?
            }
        }
        Commands::Failed => encode_to_vec(BartoCli::Failed, standard())?,
        Commands::Cmd { cmd_name } => encode_to_vec(
            BartoCli::Cmd {
                cmd_name: cmd_name.clone(),
            },
            standard(),
        )?,
    };
    Ok(Message::Binary(payload.into()))
}

fn make_tls_connector(config: &Config) -> Result<Connector> {
    use rustls::{ClientConfig, RootCertStore};
    let root_store = if let Some(ca_cert_path) = config.bartos().ca_cert() {
        load_pinned_root_store(ca_cert_path)?
    } else {
        let mut store = RootCertStore::empty();
        store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        store
    };
    let builder = ClientConfig::builder().with_root_certificates(root_store);
    let tls = match (config.bartos().client_cert(), config.bartos().client_key()) {
        (Some(cert_path), Some(key_path)) => {
            let (cert_chain, key) = load_client_cert_and_key(cert_path, key_path)?;
            builder.with_client_auth_cert(cert_chain, key)?
        }
        _ => builder.with_no_client_auth(),
    };
    Ok(Connector::Rustls(Arc::new(tls)))
}

#[cfg(test)]
mod tests {
    use bincode_next::{config::standard, decode_from_slice};
    use libbarto::BartoCli;
    use tokio_tungstenite::tungstenite::Message;

    use super::build_message;
    use crate::runtime::cli::Commands;

    fn payload(msg: Message) -> Vec<u8> {
        match msg {
            Message::Binary(bytes) => bytes.to_vec(),
            other => panic!("expected binary message, got {other:?}"),
        }
    }

    #[test]
    fn build_message_info() {
        let msg = build_message(&Commands::Info { json: true }).expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::Info { json: true }));
    }

    #[test]
    fn build_message_updates() {
        let msg = build_message(&Commands::Updates {
            name: "host1".to_string(),
            update_kind: "garuda".to_string(),
        })
        .expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::Updates { .. }));
    }

    #[test]
    fn build_message_cleanup_and_failed() {
        assert!(matches!(
            build_message(&Commands::Cleanup).expect("build"),
            Message::Binary(_)
        ));
        assert!(matches!(
            build_message(&Commands::Failed).expect("build"),
            Message::Binary(_)
        ));
    }

    #[test]
    fn build_message_clients_variants() {
        let msg = build_message(&Commands::Clients { versions: false }).expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::Clients));

        let msg = build_message(&Commands::Clients { versions: true }).expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::ClientVersions));
    }

    #[test]
    fn build_message_query() {
        let msg = build_message(&Commands::Query {
            query: "select 1".to_string(),
        })
        .expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::Query { .. }));
    }

    #[test]
    fn build_message_list_variants() {
        let msg = build_message(&Commands::List {
            name: "host1".to_string(),
            cmd_name_opt: Some("backup".to_string()),
        })
        .expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::List { .. }));

        let msg = build_message(&Commands::List {
            name: "host1".to_string(),
            cmd_name_opt: None,
        })
        .expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::ListCommands { .. }));
    }

    #[test]
    fn build_message_cmd() {
        let msg = build_message(&Commands::Cmd {
            cmd_name: "backup".to_string(),
        })
        .expect("build");
        let (decoded, _): (BartoCli, _) =
            decode_from_slice(&payload(msg), standard()).expect("decode");
        assert!(matches!(decoded, BartoCli::Cmd { .. }));
    }
}
