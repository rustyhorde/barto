// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use clap::{ArgAction, Parser};
use config::{ConfigError, Map, Source, Value, ValueKind};
use getset::Getters;
use libbarto::PathDefaults;

use crate::config::PathDefaultsExt;

#[derive(Clone, Debug, Getters, Parser)]
#[command(author, version, about, long_about = None)]
#[getset(get = "pub(crate)")]
pub(crate) struct Cli {
    /// Set logging verbosity.  More v's, more verbose.
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Turn up logging verbosity (multiple will turn it up more)",
        conflicts_with = "quiet",
    )]
    verbose: u8,
    /// Set logging quietness.  More q's, more quiet.
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Turn down logging verbosity (multiple will turn it down more)",
        conflicts_with = "verbose",
    )]
    quiet: u8,
    /// Enable logging to stdout/stderr in additions to the tracing output file
    /// * NOTE * - This should not be used when running as a daemon/service
    #[clap(short, long, help = "Enable logging to stdout/stderr")]
    enable_std_output: bool,
    /// The absolute path to a non-standard config file
    #[clap(short, long, help = "Specify the absolute path to the config file")]
    config_absolute_path: Option<String>,
    /// The absolute path to a non-standard tracing output file
    #[clap(
        short,
        long,
        help = "Specify the absolute path to the tracing output file"
    )]
    tracing_absolute_path: Option<String>,
    /// The absolute path to a non-standard redb database file
    #[clap(
        short,
        long,
        help = "Specify the absolute path to the redb database file"
    )]
    redb_absolute_path: Option<String>,
}

impl Source for Cli {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut map = Map::new();
        let origin = String::from("command line");
        let _old = map.insert(
            "verbose".to_string(),
            Value::new(Some(&origin), ValueKind::U64(u8::into(self.verbose))),
        );
        let _old = map.insert(
            "quiet".to_string(),
            Value::new(Some(&origin), ValueKind::U64(u8::into(self.quiet))),
        );
        let _old = map.insert(
            "enable_std_output".to_string(),
            Value::new(Some(&origin), ValueKind::Boolean(self.enable_std_output)),
        );
        if let Some(config_path) = &self.config_absolute_path {
            let _old = map.insert(
                "config_path".to_string(),
                Value::new(Some(&origin), ValueKind::String(config_path.clone())),
            );
        }
        if let Some(tracing_path) = &self.tracing_absolute_path {
            let _old = map.insert(
                "tracing_path".to_string(),
                Value::new(Some(&origin), ValueKind::String(tracing_path.clone())),
            );
        }
        if let Some(redb_path) = &self.redb_absolute_path {
            let _old = map.insert(
                "redb_path".to_string(),
                Value::new(Some(&origin), ValueKind::String(redb_path.clone())),
            );
        }
        Ok(map)
    }
}

impl PathDefaults for Cli {
    fn env_prefix(&self) -> String {
        env!("CARGO_PKG_NAME").to_ascii_uppercase()
    }

    fn config_absolute_path(&self) -> Option<String> {
        self.config_absolute_path.clone()
    }

    fn default_file_path(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn default_file_name(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn tracing_absolute_path(&self) -> Option<String> {
        self.tracing_absolute_path.clone()
    }

    fn default_tracing_path(&self) -> String {
        format!("{}/logs", env!("CARGO_PKG_NAME"))
    }

    fn default_tracing_file_name(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }
}

impl PathDefaultsExt for Cli {
    fn redb_absolute_path(&self) -> Option<String> {
        self.redb_absolute_path.clone()
    }

    fn default_redb_path(&self) -> String {
        format!("{}/db", env!("CARGO_PKG_NAME"))
    }

    fn default_redb_file_name(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use libbarto::PathDefaults;

    use super::{Cli, PathDefaultsExt};

    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("bartoc").chain(args.iter().copied()))
    }

    #[test]
    fn defaults_are_zero() {
        let cli = parse(&[]);
        assert_eq!(*cli.verbose(), 0);
        assert_eq!(*cli.quiet(), 0);
        assert!(!*cli.enable_std_output());
        assert!(cli.config_absolute_path().is_none());
        assert!(cli.tracing_absolute_path().is_none());
        assert!(PathDefaultsExt::redb_absolute_path(&cli).is_none());
    }

    #[test]
    fn verbose_flag_increments() {
        let cli = parse(&["-v", "-v"]);
        assert_eq!(*cli.verbose(), 2);
    }

    #[test]
    fn quiet_flag_increments() {
        let cli = parse(&["-q", "-q", "-q"]);
        assert_eq!(*cli.quiet(), 3);
    }

    #[test]
    fn enable_std_output_flag() {
        let cli = parse(&["-e"]);
        assert!(*cli.enable_std_output());
    }

    #[test]
    fn redb_absolute_path_set() {
        let cli = parse(&["-r", "/tmp/test.redb"]);
        assert_eq!(cli.redb_absolute_path().as_deref(), Some("/tmp/test.redb"));
    }

    #[test]
    fn config_absolute_path_set() {
        let cli = parse(&["-c", "/etc/bartoc.toml"]);
        assert_eq!(
            cli.config_absolute_path().as_deref(),
            Some("/etc/bartoc.toml")
        );
    }

    #[test]
    fn tracing_absolute_path_set() {
        let cli = parse(&["-t", "/var/log/bartoc.log"]);
        assert_eq!(
            cli.tracing_absolute_path().as_deref(),
            Some("/var/log/bartoc.log")
        );
    }

    #[test]
    fn path_defaults_env_prefix() {
        let cli = parse(&[]);
        assert_eq!(cli.env_prefix(), "BARTOC");
    }

    #[test]
    fn path_defaults_default_file_path() {
        let cli = parse(&[]);
        assert_eq!(cli.default_file_path(), "bartoc");
    }

    #[test]
    fn path_defaults_default_tracing_path() {
        let cli = parse(&[]);
        assert_eq!(cli.default_tracing_path(), "bartoc/logs");
    }

    #[test]
    fn path_defaults_ext_default_redb_path() {
        let cli = parse(&[]);
        assert_eq!(cli.default_redb_path(), "bartoc/db");
    }

    #[test]
    fn path_defaults_ext_default_redb_file_name() {
        let cli = parse(&[]);
        assert_eq!(cli.default_redb_file_name(), "bartoc");
    }
}
