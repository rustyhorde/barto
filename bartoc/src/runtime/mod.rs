// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use std::{ffi::OsString, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::StreamExt;
use libbarto::{init_tracing, load};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{select, spawn, sync::mpsc::unbounded_channel, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, protocol::frame::coding::CloseCode},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace};

use crate::{
    config::Config,
    error::Error,
    handler::{BartocMessage, Handler},
};

use self::cli::Cli;

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
    let config = load::<Cli, Config, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;

    // Initialize tracing
    init_tracing(&config, &cli, None).with_context(|| Error::TracingInit)?;

    trace!("configuration loaded");
    trace!("tracing initialized");

    let token = CancellationToken::new();
    let stream_token = token.clone();
    let heartbeat_token = token.clone();
    let (tx, mut rx) = unbounded_channel();
    let (ws_stream, _) = connect_async("wss://localhost.ozias.net:21526/v1/ws/worker").await?;
    trace!("websocket connected");
    let (sink, mut stream) = ws_stream.split();
    let mut handler = Handler::builder()
        .sink(sink)
        .tx(tx.clone())
        .token(heartbeat_token)
        .build();
    handler.heartbeat();
    trace!("bartoc heartbeat started");

    let sink_handle = spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = handler.handle_msg(msg).await {
                error!("{e}");
                trace!("shutting down sink handler");
                break;
            }
        }
    });

    // Setup the signal handling
    let sighan_handle = spawn(async move { handle_signals(token).await });

    loop {
        select! {
            () = stream_token.cancelled() => {
                let cr = Some((u16::from(CloseCode::Normal), "cancellation token triggered, shutting down bartoc".into()));
                if let Err(e) = tx.send(BartocMessage::close(cr)) {
                    error!("unable to send close message to bartos: {e}");
                }
                if let Err(e) = tx.send(BartocMessage::Close) {
                    error!("unable to send close message to handler: {e}");
                }
                trace!("cancellation token triggered, shutting down bartoc");
                // sleep a bit to allow the close message to be sent to bartos
                sleep(Duration::from_secs(1)).await;
                break;
            }
            next_opt = stream.next() => {
                if let Some(msg_res) = next_opt {
                    match msg_res {
                        Ok(msg) => match msg {
                            Message::Text(_utf8_bytes) => error!("text message received, ignoring"),
                            Message::Binary(_bytes) => todo!(),
                            Message::Ping(bytes) => {
                                trace!("ping message received, sending pong");
                                if let Err(e) = tx.send(BartocMessage::Ping(bytes.into())) {
                                    error!("unable to send ping message to handler: {e}");
                                }
                            }
                            Message::Pong(bytes) => {
                                trace!("pong message received");
                                if let Err(e) = tx.send(BartocMessage::Pong(bytes.into())) {
                                    error!("unable to send pong message to handler: {e}");
                                }
                            },
                            Message::Close(close_frame) => {
                                trace!("close message received, shutting down bartoc");
                                if let Some(cf) = &close_frame {
                                    let code = u16::from(cf.code);
                                    if cf.reason.is_empty() {
                                        trace!("close reason: code={code} no reason given");
                                    } else {
                                        trace!("close reason: code={code} reason={}", cf.reason);
                                    }
                                } else {
                                    trace!("close reason: none");
                                }
                                if let Err(e) = tx.send(BartocMessage::Close) {
                                    error!("unable to send close message to handler: {e}");
                                }
                                stream_token.cancel();
                            },
                            Message::Frame(_frame) => error!("frame message received, ignoring"),
                        },
                        Err(e) => {
                            error!("websocket error: {e}");
                            stream_token.cancel();
                            tx.send(BartocMessage::Close)?;
                        }
                    }
                }
            }
        }
    }

    sink_handle.await?;
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
