// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    io::{Write, stdout},
    net::{IpAddr, SocketAddr},
    sync::mpsc as std_mpsc,
    time::Duration,
};

use actix_web::{
    App, HttpServer,
    middleware::Compress,
    web::{Data, scope},
};
use anyhow::{Context, Result};
use clap::Parser;
use libbarto::{
    Realtime, Schedules, header, init_tracing, key_fingerprint, load, load_tls_config,
    parse_signing_key, resolve_config_path,
};
use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use rustls::crypto::ring::default_provider;
use sqlx::MySqlPool;
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{
    select, spawn,
    sync::{Mutex, RwLock, broadcast, mpsc, oneshot},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};

use crate::{common::Clients, config::Config, endpoints::insecure::insecure_config, error::Error};

use self::cli::Cli;

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗ ███████╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗██╔════╝
██████╔╝███████║██████╔╝   ██║   ██║   ██║███████╗
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║╚════██║
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝███████║
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚══════╝";

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
    let config = load::<Cli, Config, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;
    // Initialize tracing
    init_tracing(&config, config.tracing().file(), &cli, None)
        .with_context(|| Error::TracingInit)?;

    trace!("configuration loaded");
    trace!("tracing initialized");

    // Display the bartos header
    let writer: Option<&mut dyn Write> = if config.enable_std_output() {
        Some(&mut stdout())
    } else {
        None
    };
    header::<Config, dyn Write>(&config, HEADER_PREFIX, writer)?;
    info!("{} configured!", env!("CARGO_PKG_NAME"));
    if let Some(sk_b64) = config.signing_key() {
        match parse_signing_key(sk_b64) {
            Ok(sk) => info!(
                "Ed25519 signing key loaded (public fingerprint: {})",
                key_fingerprint(&sk.verifying_key())
            ),
            Err(e) => warn!("Ed25519 signing key is set but invalid: {e}"),
        }
    } else {
        info!("Ed25519 signing key not configured — messages will be unsigned");
    }

    // Setup the default crypto provider
    match default_provider().install_default() {
        Ok(()) => trace!("crypto provider initialized"),
        Err(_e) => warn!("crypto provider already initialized"),
    }

    // Add config to app data
    let config_c = config.clone();
    let config_data = Data::new(config_c);

    let workers = usize::from(*config.actix().workers());
    let tls_opt = if let Some(actix_tls) = config.actix().tls() {
        // Load the TLS server configuration
        let server_config = load_tls_config(actix_tls)?;
        // Setup the socket address
        let port = actix_tls.port();
        let ip = actix_tls.ip();
        let ip_addr: IpAddr = ip.parse().with_context(|| Error::InvalidIp)?;
        Some((SocketAddr::from((ip_addr, port)), server_config))
    } else {
        None
    };

    let bartos_port = *config.actix().port();
    let bartos_host = config.actix().ip();

    // Setup the signal handling
    let token = CancellationToken::new();
    let server_token = token.clone();
    let app_token = token.clone();
    let app_token_data = Data::new(app_token);

    // Setup the database pool
    let url = config.mariadb().connection_string();
    info!(
        "connecting to database at: {}",
        config.mariadb().disp_connection_string()
    );
    let pool = MySqlPool::connect(&url).await?;
    let pool_data = Data::new(pool);

    // Setup the client data
    let clients = Clients::builder().build();
    let clients_data = Data::new(Mutex::new(clients));

    // Live schedules: updated on config reload, read by worker WS handlers
    let live_schedules_data: Data<RwLock<BTreeMap<String, Schedules>>> =
        Data::new(RwLock::new(config.schedules().clone()));

    // Broadcast channel: notifies all open worker WS connections to re-send Initialize
    let (reload_bcast_tx, _) = broadcast::channel::<()>(16);
    let reload_bcast_data = Data::new(reload_bcast_tx.clone());

    // Internal trigger channel: file watcher and SIGHUP both send here
    let (reload_trigger_tx, mut reload_trigger_rx) = mpsc::channel::<()>(4);

    // Reload task: receives triggers, validates and applies the updated config
    let cli_reload = cli.clone();
    let live_schedules_reload = live_schedules_data.clone();
    let reload_bcast_tx_task = reload_bcast_tx.clone();
    let _reload_handle = spawn(async move {
        while reload_trigger_rx.recv().await.is_some() {
            match load::<Cli, Config, Cli>(&cli_reload, &cli_reload) {
                Err(e) => error!("config reload failed, keeping existing schedules: {e}"),
                Ok(new_config) => {
                    let mut valid = true;
                    for (client, sched_group) in new_config.schedules() {
                        for sched in sched_group.schedules() {
                            if Realtime::try_from(sched.on_calendar().as_str()).is_err() {
                                error!(
                                    "invalid on_calendar '{}' for client '{client}', aborting reload",
                                    sched.on_calendar()
                                );
                                valid = false;
                            }
                        }
                    }
                    if valid {
                        *live_schedules_reload.write().await = new_config.schedules().clone();
                        let _ = reload_bcast_tx_task.send(());
                        info!("config reloaded, schedules pushed to all connected clients");
                    }
                }
            }
        }
    });

    // File watcher: a std bridge thread feeds a tokio channel so we can select! on cancellation.
    // We use a std thread because notify's callback is sync; a tokio mpsc bridges to async code.
    let config_path = resolve_config_path(&cli).with_context(|| Error::ConfigLoad)?;
    let watch_dir = config_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("config path has no parent directory"))?
        .to_owned();
    let (watcher_tokio_tx, mut watcher_tokio_rx) = mpsc::channel::<DebounceEventResult>(4);
    let (watcher_ready_tx, watcher_ready_rx) = oneshot::channel::<Result<(), String>>();
    let _watcher_thread = std::thread::spawn(move || {
        let (std_tx, std_rx) = std_mpsc::channel::<DebounceEventResult>();
        let mut debouncer = match new_debouncer(Duration::from_millis(400), std_tx) {
            Ok(d) => d,
            Err(e) => {
                drop(watcher_ready_tx.send(Err(format!("{e}"))));
                return;
            }
        };
        if let Err(e) = debouncer
            .watcher()
            .watch(&watch_dir, RecursiveMode::NonRecursive)
        {
            drop(watcher_ready_tx.send(Err(format!("{e}"))));
            return;
        }
        drop(watcher_ready_tx.send(Ok(())));
        loop {
            match std_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(result) => {
                    if watcher_tokio_tx.blocking_send(result).is_err() {
                        break; // tokio receiver dropped — server is shutting down
                    }
                }
                Err(std_mpsc::RecvTimeoutError::Timeout) => {
                    if watcher_tokio_tx.is_closed() {
                        break;
                    }
                }
                Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    match watcher_ready_rx.await {
        Ok(Ok(())) => info!("watching config file: {}", config_path.display()),
        Ok(Err(e)) => warn!("config file watcher unavailable, SIGHUP reload still works: {e}"),
        Err(_) => warn!("config file watcher thread failed to start"),
    }

    let reload_trigger_tx_fw = reload_trigger_tx.clone();
    let server_token_fw = server_token.clone();
    let _watcher_handle = spawn(async move {
        loop {
            select! {
                () = server_token_fw.cancelled() => break,
                res = watcher_tokio_rx.recv() => {
                    match res {
                        Some(Ok(events)) => {
                            if events.iter().any(|e| e.path == config_path) {
                                info!("config file changed, triggering reload");
                                let _ = reload_trigger_tx_fw.send(()).await;
                            }
                        }
                        Some(Err(e)) => {
                            warn!("file watcher error: {e}");
                        }
                        None => break,
                    }
                }
            }
        }
        // dropping watcher_tokio_rx here closes the channel → bridge thread exits
    });

    // Signal handler: SIGINT/SIGTERM cancel the server; SIGHUP triggers config reload
    let reload_trigger_tx_sig = reload_trigger_tx.clone();
    let sighan_handle = spawn(async move { handle_signals(token, reload_trigger_tx_sig).await });

    // Startup the server
    info!(
        "Starting {} on {bartos_host}:{bartos_port}",
        env!("CARGO_PKG_NAME")
    );
    if let Some(actix_tls) = config.actix().tls() {
        info!(
            "Starting {} TLS on {}:{}",
            env!("CARGO_PKG_NAME"),
            actix_tls.ip(),
            actix_tls.port()
        );
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_token_data.clone())
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(clients_data.clone())
            .app_data(live_schedules_data.clone())
            .app_data(reload_bcast_data.clone())
            .wrap(Compress::default())
            .service(scope("/v1").configure(insecure_config))
    })
    .workers(workers)
    .bind((bartos_host.as_str(), bartos_port))?;

    let server = if let Some((addr, server_config)) = tls_opt {
        server.bind_rustls_0_23(addr, server_config)?
    } else {
        server
    };

    select! {
        () = server_token.cancelled() => {
            trace!("cancellation token triggered, shutting down bartos");
            // sleep to allow existing connections to send close messages
            sleep(Duration::from_secs(1)).await;
        }
        _ = server.run() => {
        }
    }

    let _res = sighan_handle.await?;
    Ok(())
}

#[cfg(unix)]
async fn handle_signals(token: CancellationToken, reload_tx: mpsc::Sender<()>) -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sighup = signal(SignalKind::hangup())?;

    loop {
        select! {
            () = token.cancelled() => {
                trace!("cancellation token triggered, shutting down signal handler");
                break;
            }
            _ = sigint.recv() => {
                trace!("received SIGINT, shutting down bartos");
                token.cancel();
                break;
            }
            _ = sigterm.recv() => {
                trace!("received SIGTERM, shutting down bartos");
                token.cancel();
                break;
            }
            _ = sighup.recv() => {
                info!("received SIGHUP, triggering config reload");
                let _ = reload_tx.send(()).await;
            }
        }
    }
    Ok(())
}

#[cfg(not(unix))]
async fn handle_signals(token: CancellationToken, _reload_tx: mpsc::Sender<()>) -> Result<()> {
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
                trace!("received CTRL-C, shutting down bartos");
                token.cancel();
                Ok(())
            }
        }
    }
}
