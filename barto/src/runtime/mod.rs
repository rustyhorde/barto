// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    ffi::OsString,
    io::{Write, stdout},
};

use anyhow::{Context as _, Result};
use clap::Parser as _;
use iced::{Task, Theme};
use libbarto::{header, init_tracing, load};
use tracing::{info, trace};

use crate::{config::Config, error::Error, message::Message, runtime::cli::Cli, state::State};

use iced_fonts::REQUIRED_FONT_BYTES;

mod cli;

const HEADER_PREFIX: &str = r"██████╗  █████╗ ██████╗ ████████╗ ██████╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗
██████╔╝███████║██████╔╝   ██║   ██║   ██║
██╔══██╗██╔══██║██╔══██╗   ██║   ██║   ██║
██████╔╝██║  ██║██║  ██║   ██║   ╚██████╔╝
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ";

pub(crate) fn run<I, T>(args: Option<I>) -> Result<()>
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

    iced::application(State::title, State::update, State::view)
        .theme(theme)
        .font(REQUIRED_FONT_BYTES)
        .exit_on_close_request(false)
        .run_with(|| {
            (
                State::builder().config(config).build(),
                Task::done(Message::Initialized),
            )
        })?;
    Ok(())
}

fn theme(state: &State) -> Theme {
    state.theme().clone()
}
