// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bon::Builder;
use getset::{Getters, MutGetters};
use iced::Theme;
use sqlx::MySqlPool;

use crate::{config::Config, screen::main::MainScreen};

#[derive(Builder, Debug, Getters, MutGetters)]
#[getset(get = "pub(crate)", get_mut = "pub(crate)")]
pub(crate) struct State {
    #[builder(default = Theme::CatppuccinMocha)]
    theme: Theme,
    config: Config,
    current_screen: Option<Screen>,
    db: Option<MySqlPool>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Screen {
    Main(MainScreen),
}
