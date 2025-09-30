// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use std::ffi::OsString;

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use libbarto::{init_tracing, load};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{select, spawn, sync::mpsc::unbounded_channel};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace};

use crate::{config::Config, error::Error};

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
    let cloned_token = token.clone();
    let (tx, mut rx) = unbounded_channel();
    let (ws_stream, _) = connect_async("wss://localhost.ozias.net:21526/v1/ws/worker").await?;
    trace!("websocket connected");

    let (mut sink, mut stream) = ws_stream.split();
    let sink_handle = spawn(async move {
        while let Some(msg) = rx.recv().await {
            match &msg {
                Message::Close(close_reason) => {
                    if let Some(reason) = close_reason {
                        trace!(
                            "websocket close message received: code={}, reason={}",
                            reason.code, reason.reason
                        );
                    } else {
                        trace!("websocket close message received");
                    }
                    trace!("shutting down bartoc");
                    break;
                }
                _ => {
                    if let Err(e) = sink.send(msg).await {
                        error!("unable to send message to websocket: {e}");
                        break;
                    }
                }
            }
        }
    });

    // Setup the signal handling
    let sighan_handle = spawn(async move { handle_signals(token).await });

    loop {
        select! {
            () = cloned_token.cancelled() => {
                trace!("cancellation token triggered, shutting down bartoc");
                let close_frame = CloseFrame {
                    code: CloseCode::Normal,
                    reason: "cancellation token triggered, shutting down bartoc".into(),
                };
                if let Err(e) = tx.send(Message::Close(Some(close_frame))) {
                    error!("unable to send close message to websocket: {e}");
                }
                break;
            }
            next_opt = stream.next() => {
                if let Some(msg_res) = next_opt {
                    match msg_res {
                        Ok(msg) => match msg {
                            Message::Text(_utf8_bytes) => todo!(),
                            Message::Binary(_bytes) => todo!(),
                            Message::Ping(bytes) => {
                                if let Err(e) = tx.send(Message::Pong(bytes)) {
                                    error!("unable to send pong message to websocket: {e}");
                                }
                            }
                            Message::Pong(_bytes) => todo!(),
                            Message::Close(_close_frame) => todo!(),
                            Message::Frame(_frame) => todo!(),
                        },
                        Err(e) => {
                            error!("websocket error: {e}");
                            tx.send(Message::Close(None))?;
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
    tokio::signal::ctrl_c().await?;
    trace!("received CTRL-C, shutting down bartoc");
    token.cancel();
    Ok(())
}

// fn handle_ws_client_error(e: WsClientError) {
//     match e {
//         WsClientError::InvalidResponseStatus(status_code) => {
//             error!("invalid response status code: {status_code}");
//         }
//         WsClientError::InvalidUpgradeHeader => {
//             error!("invalid upgrade header");
//         }
//         WsClientError::InvalidConnectionHeader(header_value) => {
//             error!("invalid connection header: {header_value:?}");
//         }
//         WsClientError::MissingConnectionHeader => {
//             error!("missing connection header");
//         }
//         WsClientError::MissingWebSocketAcceptHeader => {
//             error!("missing websocket accept header");
//         }
//         WsClientError::InvalidChallengeResponse(_, header_value) => {
//             error!("invalid challenge response: {header_value:?}");
//         }
//         WsClientError::Protocol(protocol_error) => {
//             error!("protocol error: {protocol_error}");
//         }
//         WsClientError::SendRequest(send_request_error) => {
//             error!("send request error: {send_request_error}");
//         }
//     }
// }
