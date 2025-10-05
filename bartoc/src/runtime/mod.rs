// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use std::{
    ffi::OsString,
    io::{Write, stdout},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::StreamExt;
use libbarto::{header, init_tracing};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{
    select, spawn,
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::sleep,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::frame::coding::CloseCode};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

use crate::{
    config::{Config, load_bartoc},
    db::{
        BartocDatabase,
        data::output::{OutputKey, OutputValue},
    },
    error::Error,
    handler::{BartocMessage, Handler, stream::WsHandler},
};

pub(crate) use self::cli::Cli;

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗  ██████╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗██╔════╝
██████╔╝███████║██████╔╝   ██║   ██║   ██║██║     
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║██║     
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝╚██████╗
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝  ╚═════╝";

#[allow(clippy::too_many_lines)]
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

    // Load the configuration
    let config = load_bartoc::<Cli, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;

    // Initialize tracing
    init_tracing(&config, &cli, None).with_context(|| Error::TracingInit)?;

    trace!("configuration loaded");
    trace!("tracing initialized");

    // Display the bartoc header
    let writer: Option<&mut dyn Write> = if config.enable_std_output() {
        Some(&mut stdout())
    } else {
        None
    };
    header::<Config, dyn Write>(&config, HEADER_PREFIX, writer)?;

    // Create or open the database
    let mut db = BartocDatabase::new(&config)?;

    let token = CancellationToken::new();
    let stream_token = token.clone();
    let heartbeat_token = token.clone();
    let output_token = token.clone();
    let (tx, mut rx) = unbounded_channel();
    let (output_tx, mut output_rx) = unbounded_channel();
    let (ws_stream, _) =
        connect_async("wss://localhost.ozias.net:21526/v1/ws/worker?name=garuda").await?;
    trace!("websocket connected");
    let (sink, mut stream) = ws_stream.split();
    let mut handler = Handler::builder()
        .sink(sink)
        .tx(tx.clone())
        .output_tx(output_tx.clone())
        .token(heartbeat_token)
        .build();
    handler.heartbeat();
    trace!("bartoc heartbeat started");
    let mut ws_handler = WsHandler::builder()
        .tx(tx.clone())
        .token(stream_token.clone())
        .build();

    let sink_handle = spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = handler.handle_msg(msg).await {
                error!("{e}");
                trace!("shutting down sink handler");
                break;
            }
        }
    });

    let output_handle = spawn(async move {
        loop {
            select! {
                () = output_token.cancelled() => {
                    trace!("cancellation token triggered, shutting down output handler");
                    break;
                }
                rx_opt = output_rx.recv() => {
                    if let Some(output) = rx_opt && let Err(e) = db.write_kv(&OutputKey::from(&output), &OutputValue::from(&output)) {
                        error!("unable to write output to database: {e}");
                    }
                },
            }
        }
    });

    // Setup the signal handling
    let sighan_handle = spawn(async move { handle_signals(token).await });

    info!("bartoc started!");
    loop {
        select! {
            () = stream_token.cancelled() => {
                handle_cancellation(tx).await;
                break;
            }
            next_opt = stream.next() => {
                if let Err(e) = ws_handler.handle_msg(next_opt) {
                    error!("{e}");
                }
            }
        }
    }

    sink_handle.await?;
    output_handle.await?;
    let _res = sighan_handle.await?;
    Ok(())
}

#[cfg(unix)]
async fn handle_signals(token: CancellationToken) -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sighup = signal(SignalKind::hangup())?;

    select! {
        () = token.cancelled() => {
            trace!("cancellation token triggered, shutting down signal handler");
        }
        _ = sigint.recv() => {
            trace!("received SIGINT, shutting down bartoc");
            token.cancel();
        }
        _ = sigterm.recv() => {
            trace!("received SIGTERM, shutting down bartoc");
            token.cancel();
        }
        _ = sighup.recv() => {
            trace!("received SIGHUP, reloading configuration");
        }
    }
    Ok(())
}

#[cfg(not(unix))]
async fn handle_signals(token: CancellationToken) -> Result<()> {
    select! {
        () = token.cancelled() => {
            trace!("cancellation token triggered, shutting down signal handler");
            Ok(())
        }
        res = ctrl_c() => {
            if let Err(e) = res {
                error!("unable to listen for CTRL-C: {e}");
                Err(e.into())
            } else {
                trace!("received CTRL-C, shutting down bartoc");
                token.cancel();
                Ok(())
            }
        }
    }
}

async fn handle_cancellation(tx: UnboundedSender<BartocMessage>) {
    let cr = Some((
        u16::from(CloseCode::Normal),
        "cancellation token triggered, shutting down bartoc".into(),
    ));
    if let Err(e) = tx.send(BartocMessage::close(cr)) {
        error!("unable to send close message to bartos: {e}");
    }
    if let Err(e) = tx.send(BartocMessage::Close) {
        error!("unable to send close message to handler: {e}");
    }
    trace!("cancellation token triggered, shutting down bartoc");
    // sleep a bit to allow the close message to be sent to bartos
    sleep(Duration::from_secs(1)).await;
}
