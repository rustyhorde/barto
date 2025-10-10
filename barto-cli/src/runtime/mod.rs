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

use anyhow::{Context as _, Result};
use bincode::{config::standard, encode_to_vec};
use clap::Parser as _;
use futures_util::{SinkExt as _, StreamExt as _};
use libbarto::{BartoCli, header, init_tracing, load};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::trace;

use crate::{config::Config, error::Error};

use self::cli::Cli;

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗        ██████╗██╗     ██╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗      ██╔════╝██║     ██║
██████╔╝███████║██████╔╝   ██║   ██║   ██║█████╗██║     ██║     ██║
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║╚════╝██║     ██║     ██║
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝      ╚██████╗███████╗██║
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝        ╚═════╝╚══════╝╚═╝";

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
    header::<Config, dyn Write>(&config, HEADER_PREFIX, Some(&mut stdout()))?;

    let url = format!(
        "{}://{}:{}/v1/ws/cli?name={}",
        config.bartos().prefix(),
        config.bartos().host(),
        config.bartos().port(),
        config.name()
    );
    trace!("connecting to bartos at {url}");
    let (ws_stream, _) = connect_async(&url).await?;
    trace!("websocket connected");
    let (mut sink, mut _stream) = ws_stream.split();

    let info = encode_to_vec(BartoCli::Info, standard())?;
    sink.send(Message::Binary(info.into())).await?;
    trace!("info message sent");
    sleep(Duration::from_secs(5)).await;
    sink.send(Message::Close(None)).await?;
    trace!("connection closed");
    Ok(())
}
