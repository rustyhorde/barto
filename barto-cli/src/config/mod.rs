// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use getset::{CopyGetters, Getters, Setters};
use libbarto::{Bartos, Tracing, TracingConfigExt};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber_init::{TracingConfig, get_effective_level};

#[derive(
    Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize, Setters,
)]
pub(crate) struct Config {
    #[getset(get_copy = "pub(crate)")]
    verbose: u8,
    #[getset(get_copy = "pub(crate)")]
    quiet: u8,
    #[getset(get_copy = "pub(crate)")]
    #[getset(set = "pub(crate)")]
    enable_std_output: bool,
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
        self.tracing().stdout().with_target()
    }

    fn with_thread_ids(&self) -> bool {
        self.tracing().stdout().with_thread_ids()
    }

    fn with_thread_names(&self) -> bool {
        self.tracing().stdout().with_thread_names()
    }

    fn with_line_number(&self) -> bool {
        self.tracing().stdout().with_line_number()
    }

    fn with_level(&self) -> bool {
        self.tracing().stdout().with_level()
    }
}

impl TracingConfigExt for Config {
    fn enable_stdout(&self) -> bool {
        self.enable_std_output
    }

    fn directives(&self) -> Option<&String> {
        self.tracing().stdout().directives().as_ref()
    }

    fn level(&self) -> Level {
        get_effective_level(self.quiet(), self.verbose())
    }
}

#[cfg(test)]
mod tests {
    use tracing_subscriber_init::{TracingConfig, get_effective_level};

    use super::Config;
    use libbarto::TracingConfigExt;

    #[test]
    fn defaults() {
        let config = Config::default();
        assert_eq!(config.verbose(), 0);
        assert_eq!(config.quiet(), 0);
        assert!(!config.enable_std_output());
        assert!(config.name().is_empty());
    }

    #[test]
    fn set_enable_std_output_flips() {
        let mut config = Config::default();
        assert!(!config.enable_std_output());
        let _ = config.set_enable_std_output(true);
        assert!(config.enable_std_output());
    }

    #[test]
    fn tracing_config_methods_match_stdout() {
        let config = Config::default();
        let stdout = config.tracing().stdout();
        assert_eq!(TracingConfig::with_target(&config), stdout.with_target());
        assert_eq!(
            TracingConfig::with_thread_ids(&config),
            stdout.with_thread_ids()
        );
        assert_eq!(
            TracingConfig::with_thread_names(&config),
            stdout.with_thread_names()
        );
        assert_eq!(
            TracingConfig::with_line_number(&config),
            stdout.with_line_number()
        );
        assert_eq!(TracingConfig::with_level(&config), stdout.with_level());
        assert_eq!(TracingConfig::quiet(&config), config.quiet());
        assert_eq!(TracingConfig::verbose(&config), config.verbose());
    }

    #[test]
    fn tracing_config_ext_methods() {
        let mut config = Config::default();
        assert!(!config.enable_stdout());
        let _ = config.set_enable_std_output(true);
        assert!(config.enable_stdout());
        assert_eq!(
            config.directives(),
            config.tracing().stdout().directives().as_ref()
        );
        assert_eq!(config.level(), get_effective_level(0, 0));
    }
}
