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
use awc::{Client, error::WsClientError, http::Version};
use clap::Parser;
use futures_util::StreamExt as _;
use libbarto::{init_tracing, load};
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

    let awc = Client::builder()
        .max_http_version(Version::HTTP_11)
        .finish();

    match awc
        .ws("wss://localhost.ozias.net:21526/v1/ws/worker")
        .connect()
        .await
    {
        Ok((response, framed)) => {
            trace!("connected to server: {}", response.status());
            let (_sink, _stream) = framed.split();
        }
        Err(e) => handle_ws_client_error(e),
    }
    Ok(())
}

fn handle_ws_client_error(e: WsClientError) {
    match e {
        WsClientError::InvalidResponseStatus(status_code) => {
            error!("invalid response status code: {status_code}");
        }
        WsClientError::InvalidUpgradeHeader => {
            error!("invalid upgrade header");
        }
        WsClientError::InvalidConnectionHeader(header_value) => {
            error!("invalid connection header: {header_value:?}");
        }
        WsClientError::MissingConnectionHeader => {
            error!("missing connection header");
        }
        WsClientError::MissingWebSocketAcceptHeader => {
            error!("missing websocket accept header");
        }
        WsClientError::InvalidChallengeResponse(_, header_value) => {
            error!("invalid challenge response: {header_value:?}");
        }
        WsClientError::Protocol(protocol_error) => {
            error!("protocol error: {protocol_error}");
        }
        WsClientError::SendRequest(send_request_error) => {
            error!("send request error: {send_request_error}");
        }
    }
}
