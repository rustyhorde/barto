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

use crate::{config::Config, error::Error, handler::Handler, runtime::cli::Commands};

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
    let mut config = load::<Cli, Config, Cli>(&cli, &cli).with_context(|| Error::ConfigLoad)?;

    // Display the bartoc header
    let writer: Option<&mut dyn Write> = if config.enable_std_output() {
        Some(&mut stdout())
    } else {
        None
    };
    header::<Config, dyn Write>(&config, HEADER_PREFIX, writer)?;

    // Initialize tracing
    let _ = config.set_enable_std_output(true);
    init_tracing(&config, &cli, None).with_context(|| Error::TracingInit)?;

    trace!("configuration loaded");
    trace!("tracing initialized");

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
    let (mut sink, stream) = ws_stream.split();
    let mut handler = Handler::builder().stream(stream).build();

    let message = match cli.command() {
        Commands::Info => {
            let info = encode_to_vec(BartoCli::Info, standard())?;
            Message::Binary(info.into())
        }
        Commands::Updates { name } => {
            let update = encode_to_vec(BartoCli::Updates { name: name.clone() }, standard())?;
            Message::Binary(update.into())
        }
        Commands::Cleanup => {
            let cleanup = encode_to_vec(BartoCli::Cleanup, standard())?;
            Message::Binary(cleanup.into())
        }
        Commands::Clients => {
            let clients = encode_to_vec(BartoCli::Clients, standard())?;
            Message::Binary(clients.into())
        }
    };
    sink.send(message).await?;
    trace!("message sent");

    handler.handle().await?;

    sleep(Duration::from_secs(1)).await;
    sink.send(Message::Close(None)).await?;
    trace!("connection closed");
    Ok(())
}
