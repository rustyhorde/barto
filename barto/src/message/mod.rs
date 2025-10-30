// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use iced::Theme;
use sqlx::MySqlPool;

use crate::{screen::main::MainScreenMessage, state::Screen};

#[derive(Clone, Debug)]
pub(crate) enum Message {
    // Top-level messages
    Initialized,
    Close,
    ThemeChanged(Theme),
    Error(String),
    DatabaseInitialized(Option<MySqlPool>),
    LoadScreen(Screen),
    // Main Screen messages
    MainScreen(MainScreenMessage),
}

impl From<MainScreenMessage> for Message {
    fn from(value: MainScreenMessage) -> Self {
        Message::MainScreen(value)
    }
}

impl From<Option<MainScreenMessage>> for Message {
    fn from(value: Option<MainScreenMessage>) -> Self {
        if let Some(msg) = value {
            Message::MainScreen(msg)
        } else {
            Message::Error("Received None message".to_string())
        }
    }
}
