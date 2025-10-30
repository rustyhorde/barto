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
use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    padding::{bottom, top},
    widget::{PickList, button, column, container, horizontal_space, row, text},
    window,
};
use libbarto::{header, init_tracing, load};
use sqlx::MySqlPool;
use tracing::{info, trace};

use crate::{
    config::Config,
    error::Error,
    message::Message,
    runtime::cli::Cli,
    screen::main::{MainScreen, MainScreenMessage},
    state::{Screen, State},
};

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

    iced::application("A cool application", update, view)
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

fn view(state: &State) -> Element<'_, Message> {
    let controls = row![horizontal_space(), button("Close").on_press(Message::Close)].spacing(5);
    let status_bar = row![
        PickList::new(Theme::ALL, Some(state.theme()), Message::ThemeChanged),
        horizontal_space(),
    ]
    .padding(5);

    let curr_screen = match state.current_screen() {
        Some(Screen::Main(main_screen)) => main_screen.render().map(Message::from),
        _ => container(text("No screen")).into(),
    };

    container(
        column![
            controls.padding(bottom(10)),
            curr_screen,
            status_bar.padding(top(10))
        ]
        .height(Fill),
    )
    .padding(10)
    .height(Fill)
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Initialized => Task::perform(
            setup_database_opt(state.config().clone()),
            Message::DatabaseInitialized,
        ),
        Message::DatabaseInitialized(db_init_opt) => {
            if let Some(pool) = db_init_opt {
                *state.db_mut() = Some(pool);
                Task::done(Message::MainScreen(MainScreenMessage::Load))
            } else {
                Task::done(Message::Error("Failed to initialize database".to_string()))
            }
        }
        Message::ThemeChanged(theme) => {
            *state.theme_mut() = theme;
            Task::none()
        }
        Message::Close => window::get_latest().and_then(window::close),
        Message::Error(error) => {
            eprintln!("An error occurred: {error:?}");
            Task::done(Message::Close)
        }
        Message::MainScreen(main_screen_message) => {
            let pool_c = state.db().clone();
            if let Some(Screen::Main(main_screen)) = &mut state.current_screen_mut() {
                main_screen.handle_message(pool_c, main_screen_message)
            } else {
                MainScreen::handle_init_message(pool_c, main_screen_message)
            }
        }
        Message::LoadScreen(screen) => {
            *state.current_screen_mut() = Some(screen);
            Task::none()
        }
    }
}

async fn setup_database_opt(config: Config) -> Option<MySqlPool> {
    setup_database(config).await.ok()
}

async fn setup_database(config: Config) -> Result<MySqlPool> {
    // Setup the database pool
    let url = config.mariadb().connection_string();
    info!(
        "connecting to database at: {}",
        config.mariadb().disp_connection_string()
    );
    MySqlPool::connect(&url).await.map_err(Into::into)
}
