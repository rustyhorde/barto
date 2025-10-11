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
    sync::LazyLock,
    time::Duration,
};

use anyhow::{Context as _, Result};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use clap::Parser as _;
use console::Style;
use futures_util::{SinkExt as _, StreamExt as _};
use libbarto::{BartoCli, BartosToBartoCli, header, init_tracing, load};
use tokio::{select, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, trace};
use vergen_pretty::PrettyExt;

use crate::{config::Config, error::Error};

use self::cli::Cli;

pub(crate) static BOLD_BLUE: LazyLock<Style> = LazyLock::new(|| Style::new().bold().blue());
pub(crate) static BOLD_GREEN: LazyLock<Style> = LazyLock::new(|| Style::new().bold().green());

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
    let (mut sink, mut stream) = ws_stream.split();

    let info = encode_to_vec(BartoCli::Info, standard())?;
    sink.send(Message::Binary(info.into())).await?;
    trace!("info message sent");

    select! {
        () = sleep(Duration::from_secs(5)) => {},
        msg_opt_res = stream.next() => {
            if let Some(msg_res) = msg_opt_res {
                let msg = msg_res?;
                if let Message::Binary(bytes) = &msg {
                    match decode_from_slice(bytes, standard()) {
                        Err(e) => trace!("unable to decode binary message: {e}"),
                        Ok((msg, _)) => match msg {
                            BartosToBartoCli::Info(pretty_ext) => {
                                let (max_category, max_label) = maxes(&pretty_ext);
                                for (category, label, value) in pretty_ext.vars() {
                                    let blah = format!("{label:>max_label$} ({category:>max_category$})");
                                    let key = BOLD_BLUE.apply_to(&blah);
                                    let value = BOLD_GREEN.apply_to(value);
                                    info!("{key}: {value}");
                                }
                            },
                        }
                    }
                }
            }
        },
    }
    sleep(Duration::from_secs(1)).await;
    sink.send(Message::Close(None)).await?;
    trace!("connection closed");
    Ok(())
}

fn maxes(pretty_ext: &PrettyExt) -> (usize, usize) {
    let mut max_category = 0;
    let mut max_label = 0;
    for (category, label, _) in pretty_ext.vars() {
        if category.len() > max_category {
            max_category = category.len();
        }
        if label.len() > max_label {
            max_label = label.len();
        }
    }
    (max_category, max_label)
}
