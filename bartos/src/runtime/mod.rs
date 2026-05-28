// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use std::{
    env,
    ffi::OsString,
    io::{Write, stdout},
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use actix_web::{
    App, HttpServer,
    middleware::Compress,
    web::{Data, scope},
};
use anyhow::{Context, Result};
use clap::Parser;
use libbarto::{header, init_tracing, key_fingerprint, load, load_tls_config, parse_signing_key};
use rustls::crypto::ring::default_provider;
use sqlx::MySqlPool;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{select, spawn, sync::Mutex, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{info, trace, warn};
#[cfg(not(unix))]
use {tokio::signal::ctrl_c, tracing::error};

use crate::{common::Clients, config::Config, endpoints::insecure::insecure_config, error::Error};

use self::cli::Cli;

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

    // Display the bartoc header
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
    let sighan_handle = spawn(async move { handle_signals(token).await });

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
