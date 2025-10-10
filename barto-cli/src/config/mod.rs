// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use getset::{CopyGetters, Getters};
use libbarto::{Bartos, Tracing, TracingConfigExt};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber_init::{TracingConfig, get_effective_level};

#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub(crate) struct Config {
    #[getset(get_copy = "pub(crate)")]
    verbose: u8,
    #[getset(get_copy = "pub(crate)")]
    quiet: u8,
    #[getset(get = "pub(crate)")]
    name: String,
    #[getset(get = "pub(crate)")]
    tracing: Tracing,
    #[getset(get = "pub(crate)")]
    bartos: Bartos,
}

impl TracingConfig for Config {
    fn quiet(&self) -> u8 {
        self.quiet
    }

    fn verbose(&self) -> u8 {
        self.verbose
    }

    fn with_target(&self) -> bool {
        self.tracing.with_target()
    }

    fn with_thread_ids(&self) -> bool {
        self.tracing.with_thread_ids()
    }

    fn with_thread_names(&self) -> bool {
        self.tracing.with_thread_names()
    }

    fn with_line_number(&self) -> bool {
        self.tracing.with_line_number()
    }

    fn with_level(&self) -> bool {
        self.tracing.with_level()
    }
}

impl TracingConfigExt for Config {
    fn enable_stdout(&self) -> bool {
        true
    }

    fn directives(&self) -> Option<&String> {
        self.tracing.directives().as_ref()
    }

    fn level(&self) -> Level {
        get_effective_level(self.quiet, self.verbose)
    }
}
