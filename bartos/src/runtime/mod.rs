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
    net::{IpAddr, SocketAddr},
};

use actix_web::{
    App, HttpServer,
    middleware::Compress,
    web::{Data, scope},
};
use anyhow::{Context, Result};
use clap::Parser;
use libbarto::{init_tracing, load, load_tls_config};
use rustls::crypto::aws_lc_rs::default_provider;
use tracing::{info, trace, warn};

use crate::{config::Config, endpoints::insecure::insecure_config, error::Error};

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

    // Startup the server
    trace!("Starting {} on {socket_addr:?}", env!("CARGO_PKG_NAME"));
    info!("{} configured!", env!("CARGO_PKG_NAME"));
    info!("{} starting!", env!("CARGO_PKG_NAME"));

    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .wrap(Compress::default())
            .service(scope("/v1").configure(insecure_config))
    })
    .workers(workers)
    .bind_rustls_0_23(socket_addr, server_config)?
    .run()
    .await?;
    Ok(())
}
