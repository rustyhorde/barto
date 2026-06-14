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

#[cfg(test)]
mod tests {
    use clap::Parser;
    use config::Source;
    use libbarto::PathDefaults;

    use super::Cli;

    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("bartos").chain(args.iter().copied()))
    }

    #[test]
    fn defaults_are_zero() {
        let cli = parse(&[]);
        assert_eq!(*cli.verbose(), 0);
        assert_eq!(*cli.quiet(), 0);
        assert!(!*cli.enable_std_output());
        assert!(cli.config_absolute_path().is_none());
        assert!(cli.tracing_absolute_path().is_none());
    }

    #[test]
    fn verbose_flag_increments() {
        assert_eq!(*parse(&["-v", "-v"]).verbose(), 2);
    }

    #[test]
    fn quiet_flag_increments() {
        assert_eq!(*parse(&["-q", "-q", "-q"]).quiet(), 3);
    }

    #[test]
    fn enable_std_output_flag() {
        assert!(*parse(&["-e"]).enable_std_output());
    }

    #[test]
    fn config_absolute_path_set() {
        let cli = parse(&["-c", "/etc/bartos.toml"]);
        assert_eq!(
            cli.config_absolute_path().as_deref(),
            Some("/etc/bartos.toml")
        );
    }

    #[test]
    fn tracing_absolute_path_set() {
        let cli = parse(&["-t", "/var/log/bartos.log"]);
        assert_eq!(
            cli.tracing_absolute_path().as_deref(),
            Some("/var/log/bartos.log")
        );
    }

    #[test]
    fn source_collect_basic_keys() {
        let map = parse(&["-v"]).collect().expect("collect");
        assert!(map.contains_key("verbose"));
        assert!(map.contains_key("quiet"));
        assert!(map.contains_key("enable_std_output"));
        assert!(!map.contains_key("config_path"));
        assert!(!map.contains_key("tracing_path"));
    }

    #[test]
    fn source_collect_includes_paths_when_set() {
        let map = parse(&["-c", "/a.toml", "-t", "/b.log"])
            .collect()
            .expect("collect");
        assert!(map.contains_key("config_path"));
        assert!(map.contains_key("tracing_path"));
    }

    #[test]
    fn path_defaults_env_prefix() {
        assert_eq!(parse(&[]).env_prefix(), "BARTOS");
    }

    #[test]
    fn path_defaults_file_names() {
        let cli = parse(&[]);
        assert_eq!(cli.default_file_path(), "bartos");
        assert_eq!(cli.default_file_name(), "bartos");
        assert_eq!(cli.default_tracing_file_name(), "bartos");
    }

    #[test]
    fn path_defaults_tracing_path() {
        assert_eq!(parse(&[]).default_tracing_path(), "bartos/logs");
    }

    #[test]
    fn path_defaults_round_trip_paths() {
        let cli = parse(&["-c", "/a.toml", "-t", "/b.log"]);
        assert_eq!(
            PathDefaults::config_absolute_path(&cli).as_deref(),
            Some("/a.toml")
        );
        assert_eq!(
            PathDefaults::tracing_absolute_path(&cli).as_deref(),
            Some("/b.log")
        );
    }
}
