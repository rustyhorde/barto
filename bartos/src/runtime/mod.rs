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
    fs,
    io::{Write, stdout},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::mpsc as std_mpsc,
    time::Duration,
};

use actix_web::{
    App, HttpServer,
    dev::Server,
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
use rustls::{ServerConfig, crypto::ring::default_provider};
use sqlx::MySqlPool;
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{
    select, spawn,
    sync::{Mutex, RwLock, broadcast, mpsc, oneshot},
    task::JoinHandle,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};

use crate::{
    common::{Clients, WorkerSignal},
    config::Config,
    endpoints::insecure::insecure_config,
    error::Error,
};

use self::cli::Cli;

struct WebAppData {
    token: Data<CancellationToken>,
    config: Data<Config>,
    pool: Data<MySqlPool>,
    clients: Data<Mutex<Clients>>,
    live_schedules: Data<RwLock<BTreeMap<String, Schedules>>>,
    worker_bcast: Data<broadcast::Sender<WorkerSignal>>,
}

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗ ███████╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗██╔════╝
██████╔╝███████║██████╔╝   ██║   ██║   ██║███████╗
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║╚════██║
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝███████║
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚══════╝";

pub(crate) async fn run<I, T>(args: Option<I>) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = if let Some(args) = args {
        Cli::try_parse_from(args)?
    } else {
        Cli::try_parse()?
    };
    let config = load::<Cli, Config, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;
    init_tracing(&config, config.tracing().file(), &cli, None)
        .with_context(|| Error::TracingInit)?;
    trace!("configuration loaded");
    trace!("tracing initialized");
    display_startup_info(&config)?;

    let workers = usize::from(*config.actix().workers());
    let tls_opt = resolve_tls_config(&config)?;
    let bartos_port = *config.actix().port();
    let bartos_host = config.actix().ip().clone();

    let token = CancellationToken::new();
    let server_token = token.clone();

    let url = config.mariadb().connection_string();
    info!(
        "connecting to database at: {}",
        config.mariadb().disp_connection_string()
    );
    let (worker_bcast_tx, _) = broadcast::channel::<WorkerSignal>(16);
    let (reload_trigger_tx, reload_trigger_rx) = mpsc::channel::<()>(4);
    let live_schedules_data: Data<RwLock<BTreeMap<String, Schedules>>> =
        Data::new(RwLock::new(config.schedules().clone()));

    let _reload_handle = spawn_reload_task(
        cli.clone(),
        live_schedules_data.clone(),
        reload_trigger_rx,
        worker_bcast_tx.clone(),
    );

    let config_path = resolve_config_path(&cli).with_context(|| Error::ConfigLoad)?;
    let _watcher_handle =
        setup_file_watcher(config_path, server_token.clone(), reload_trigger_tx.clone()).await?;

    let reload_trigger_tx_sig = reload_trigger_tx.clone();
    let sighan_handle = spawn(async move { handle_signals(token, reload_trigger_tx_sig).await });

    let web_app_data = WebAppData {
        token: Data::new(server_token.clone()),
        config: Data::new(config),
        pool: Data::new(MySqlPool::connect(&url).await?),
        clients: Data::new(Mutex::new(Clients::builder().build())),
        live_schedules: live_schedules_data,
        worker_bcast: Data::new(worker_bcast_tx),
    };
    let server = build_http_server(web_app_data, workers, &bartos_host, bartos_port, tls_opt)?;

    select! {
        () = server_token.cancelled() => {
            trace!("cancellation token triggered, shutting down bartos");
            // sleep to allow existing connections to send close messages
            sleep(Duration::from_secs(1)).await;
        }
        _ = server => {}
    }

    let _res = sighan_handle.await?;
    Ok(())
}

fn display_startup_info(config: &Config) -> Result<()> {
    let writer: Option<&mut dyn Write> = if config.enable_std_output() {
        Some(&mut stdout())
    } else {
        None
    };
    header::<Config, dyn Write>(config, HEADER_PREFIX, writer)?;
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
    match default_provider().install_default() {
        Ok(()) => trace!("crypto provider initialized"),
        Err(_e) => warn!("crypto provider already initialized"),
    }
    Ok(())
}

fn resolve_tls_config(config: &Config) -> Result<Option<(SocketAddr, ServerConfig)>> {
    if let Some(actix_tls) = config.actix().tls() {
        let server_config = load_tls_config(actix_tls)?;
        let ip_addr: IpAddr = actix_tls.ip().parse().with_context(|| Error::InvalidIp)?;
        Ok(Some((
            SocketAddr::from((ip_addr, actix_tls.port())),
            server_config,
        )))
    } else {
        Ok(None)
    }
}

fn spawn_reload_task(
    cli: Cli,
    live_schedules: Data<RwLock<BTreeMap<String, Schedules>>>,
    mut reload_trigger_rx: mpsc::Receiver<()>,
    worker_bcast_tx: broadcast::Sender<WorkerSignal>,
) -> JoinHandle<()> {
    spawn(async move {
        while reload_trigger_rx.recv().await.is_some() {
            while reload_trigger_rx.try_recv().is_ok() {}
            match load::<Cli, Config, Cli>(&cli, &cli) {
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
                        let new_schedules = new_config.schedules().clone();
                        let mut schedules_guard = live_schedules.write().await;
                        if *schedules_guard == new_schedules {
                            drop(schedules_guard);
                            info!("config reloaded, schedules unchanged, skipping broadcast");
                        } else {
                            *schedules_guard = new_schedules;
                            drop(schedules_guard);
                            let _ = worker_bcast_tx.send(WorkerSignal::Reload);
                            info!("config reloaded, schedules pushed to all connected clients");
                        }
                    }
                }
            }
        }
    })
}

// File watcher: a std bridge thread feeds a tokio channel so we can select! on cancellation.
// We use a std thread because notify's callback is sync; a tokio mpsc bridges to async code.
async fn setup_file_watcher(
    config_path: PathBuf,
    server_token: CancellationToken,
    reload_trigger_tx: mpsc::Sender<()>,
) -> Result<JoinHandle<()>> {
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
    Ok(spawn(async move {
        let mut last_mtime = fs::metadata(&config_path).and_then(|m| m.modified()).ok();
        loop {
            select! {
                () = server_token.cancelled() => break,
                res = watcher_tokio_rx.recv() => {
                    match res {
                        Some(Ok(events)) => {
                            if events.iter().any(|e| e.path == config_path) {
                                let current_mtime = fs::metadata(&config_path)
                                    .and_then(|m| m.modified())
                                    .ok();
                                let changed = match (last_mtime, current_mtime) {
                                    (Some(last), Some(current)) => last != current,
                                    _ => true,
                                };
                                if changed {
                                    last_mtime = current_mtime;
                                    info!("config file changed, triggering reload");
                                    let _ = reload_trigger_tx.send(()).await;
                                } else {
                                    trace!("config file event but mtime unchanged, skipping");
                                }
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
    }))
}

fn build_http_server(
    app_data: WebAppData,
    workers: usize,
    host: &str,
    port: u16,
    tls_opt: Option<(SocketAddr, ServerConfig)>,
) -> Result<Server> {
    info!("Starting {} on {host}:{port}", env!("CARGO_PKG_NAME"));
    if let Some((addr, _)) = &tls_opt {
        info!("Starting {} TLS on {addr}", env!("CARGO_PKG_NAME"));
    }
    let WebAppData {
        token,
        config,
        pool,
        clients,
        live_schedules,
        worker_bcast,
    } = app_data;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(token.clone())
            .app_data(config.clone())
            .app_data(pool.clone())
            .app_data(clients.clone())
            .app_data(live_schedules.clone())
            .app_data(worker_bcast.clone())
            .wrap(Compress::default())
            .service(scope("/v1").configure(insecure_config))
    })
    .workers(workers)
    .bind((host, port))?;
    let server = if let Some((addr, server_config)) = tls_opt {
        server.bind_rustls_0_23(addr, server_config)?
    } else {
        server
    };
    Ok(server.run())
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
