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
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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
    db::BartocDatabase,
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

    let mut retry_count = *config.retry_count();
    let mut error_count = 0;
    let shutdown = Arc::new(AtomicBool::new(false));

    while retry_count > 0 {
        let sd_r = Arc::clone(&shutdown);
        let sd_c = Arc::clone(&shutdown);
        let res: Result<()> = async {
            let token = CancellationToken::new();
            let sig_token = token.clone();
            let stream_token = token.clone();
            let heartbeat_token = token.clone();
            let output_token = token.clone();
            let (tx, mut rx) = unbounded_channel();
            let (data_tx, data_rx) = unbounded_channel();
            let url = format!(
                "{}://{}:{}/v1/ws/worker?name={}",
                config.bartos().prefix(),
                config.bartos().host(),
                config.bartos().port(),
                config.name()
            );
            trace!("connecting to bartos at {url}");
            let (ws_stream, _) = connect_async(&url).await?;
            trace!("websocket connected");
            let (sink, mut stream) = ws_stream.split();
            let mut handler = Handler::builder()
                .sink(sink)
                .tx(tx.clone())
                .data_tx(data_tx.clone())
                .token(heartbeat_token)
                .bartoc_name(config.name().clone())
                .build();
            handler.heartbeat(config.client_timeout());
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

            // Setup the database handler
            let db_tx = tx.clone();
            let mut db = BartocDatabase::new(&config, db_tx)?;

            let db_handle = spawn(async move {
                if let Err(e) = db.monitor(data_rx, output_token).await {
                    error!("database handler error: {e}");
                }
            });

            // Setup the signal handling
            let sighan_handle = spawn(async move { handle_signals(sig_token, sd_c).await });

            info!("{} bartoc started!", config.name());
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
            db_handle.await?;
            let _res = sighan_handle.await?;
            Ok(())
        }
        .await;

        if let Err(e) = res {
            error!("{e}");
        }

        if handle_shutdown(&shutdown, sd_r, &mut retry_count, &mut error_count).await {
            break;
        }
    }

    Ok(())
}

#[cfg(unix)]
async fn handle_signals(token: CancellationToken, shutdown: Arc<AtomicBool>) -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sighup = signal(SignalKind::hangup())?;

    select! {
        () = token.cancelled() => {
            trace!("cancellation token triggered, shutting down signal handler");
        }
        _ = sigint.recv() => {
            info!("received SIGINT, shutting down bartoc");
            shutdown.store(true, Ordering::SeqCst);
            token.cancel();
        }
        _ = sigterm.recv() => {
            info!("received SIGTERM, shutting down bartoc");
            shutdown.store(true, Ordering::SeqCst);
            token.cancel();
        }
        _ = sighup.recv() => {
            info!("received SIGHUP, reloading configuration");
        }
    }
    Ok(())
}

#[cfg(not(unix))]
async fn handle_signals(token: CancellationToken, shutdown: Arc<AtomicBool>) -> Result<()> {
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
                shutdown.store(true, Ordering::SeqCst);
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

async fn handle_shutdown(
    shutdown: &AtomicBool,
    sd_r: Arc<AtomicBool>,
    retry_count: &mut u8,
    error_count: &mut u32,
) -> bool {
    let mut should_break = false;
    let sd = shutdown.load(Ordering::SeqCst);
    trace!("is bartoc shutting down? {sd}");
    if sd {
        should_break = true;
    }
    let retry_token = CancellationToken::new();
    let rt_c = retry_token.clone();
    let sighan_handle = spawn(async move { handle_signals(rt_c, sd_r).await });
    info!("retrying in {} seconds...", 2u64.pow(*error_count));

    select! {
        () = retry_token.cancelled() => {
            should_break = true;
        }
        () = sleep(Duration::from_secs(2u64.pow(*error_count))) => {
            trace!("retrying now");
            *retry_count -= 1;
            *error_count += 1;
            sighan_handle.abort();
        }
    }

    should_break
}
