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
use libbarto::{header, init_tracing, load, load_tls_config};
use rustls::crypto::aws_lc_rs::default_provider;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::{select, spawn, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{info, trace, warn};
#[cfg(not(unix))]
use {tokio::signal::ctrl_c, tracing::error};

use crate::{config::Config, endpoints::insecure::insecure_config, error::Error};

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

    // Setup the default crypto provider
    match default_provider().install_default() {
        Ok(()) => trace!("crypto provider initialized"),
        Err(_e) => warn!("crypto provider already initialized"),
    }

    // Load the TLS server configuration
    let server_config = load_tls_config(&config)?;

    // Add config to app data
    let config_c = config.clone();
    let config_data = Data::new(config_c);

    let workers = usize::from(*config.actix().workers());
    let ip = config.actix().ip();
    let port = config.actix().port();
    let ip_addr: IpAddr = ip.parse().with_context(|| Error::InvalidIp)?;
    let socket_addr = SocketAddr::from((ip_addr, *port));

    // Setup the signal handling
    let token = CancellationToken::new();
    let server_token = token.clone();
    let app_token = token.clone();
    let app_token_data = Data::new(app_token);
    let sighan_handle = spawn(async move { handle_signals(token).await });

    // Startup the server
    trace!("Starting {} on {socket_addr:?}", env!("CARGO_PKG_NAME"));
    info!("{} configured!", env!("CARGO_PKG_NAME"));
    info!("{} starting!", env!("CARGO_PKG_NAME"));

    select! {
        () = server_token.cancelled() => {
            trace!("cancellation token triggered, shutting down bartos");
            // sleep to allow existing connections to send close messages
            sleep(Duration::from_secs(1)).await;
        }
        _ = HttpServer::new(move || {
                App::new()
                    .app_data(app_token_data.clone())
                    .app_data(config_data.clone())
                    .wrap(Compress::default())
                    .service(scope("/v1").configure(insecure_config))
                })
            .workers(workers)
            .bind_rustls_0_23(socket_addr, server_config)?
            .run() => {
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
