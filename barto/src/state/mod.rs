// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use bon::Builder;
use getset::{Getters, MutGetters};
use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    padding::{bottom, top},
    widget::{PickList, button, column, container, horizontal_space, row, text},
    window,
};
use sqlx::MySqlPool;
use tracing::info;

use crate::{
    config::Config,
    message::Message,
    screen::main::{MainScreen, MainScreenMessage},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Screen {
    Main(MainScreen),
}

#[derive(Builder, Debug, Getters, MutGetters)]
#[getset(get = "pub(crate)", get_mut = "pub(crate)")]
pub(crate) struct State {
    #[builder(default = Theme::CatppuccinMocha)]
    theme: Theme,
    config: Config,
    current_screen: Option<Screen>,
    db: Option<MySqlPool>,
}

impl State {
    pub(crate) fn title(&self) -> String {
        self.config.title().clone()
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let controls =
            row![horizontal_space(), button("Close").on_press(Message::Close)].spacing(5);
        let status_bar = row![
            PickList::new(Theme::ALL, Some(self.theme()), Message::ThemeChanged),
            horizontal_space(),
        ]
        .padding(5);

        let curr_screen = match self.current_screen() {
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

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized => Task::perform(
                setup_database_opt(self.config.clone()),
                Message::DatabaseInitialized,
            ),
            Message::DatabaseInitialized(db_init_opt) => {
                if let Some(pool) = db_init_opt {
                    self.db = Some(pool);
                    Task::done(Message::MainScreen(MainScreenMessage::Load))
                } else {
                    Task::done(Message::Error("Failed to initialize database".to_string()))
                }
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::Close => window::get_latest().and_then(window::close),
            Message::Error(error) => {
                eprintln!("An error occurred: {error:?}");
                Task::done(Message::Close)
            }
            Message::MainScreen(main_screen_message) => {
                let pool_c = self.db.clone();
                if let Some(Screen::Main(main_screen)) = &mut self.current_screen {
                    main_screen.handle_message(pool_c, main_screen_message)
                } else {
                    MainScreen::handle_init_message(pool_c, main_screen_message)
                }
            }
            Message::LoadScreen(screen) => {
                self.current_screen = Some(screen);
                Task::none()
            }
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
