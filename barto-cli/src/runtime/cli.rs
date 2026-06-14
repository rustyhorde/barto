// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{io::Cursor, sync::LazyLock};

use clap::{ArgAction, Parser, Subcommand};
use config::{ConfigError, Map, Source, Value, ValueKind};
use getset::{CopyGetters, Getters};
use libbarto::PathDefaults;
use vergen_pretty::{Pretty, vergen_pretty_env};

static LONG_VERSION: LazyLock<String> = LazyLock::new(|| {
    let pretty = Pretty::builder().env(vergen_pretty_env!()).build();
    let mut cursor = Cursor::new(vec![]);
    let mut output = env!("CARGO_PKG_VERSION").to_string();
    output.push_str("\n\n");
    pretty.display(&mut cursor).unwrap();
    output += &String::from_utf8_lossy(cursor.get_ref());
    output
});

#[derive(Clone, CopyGetters, Debug, Getters, Parser)]
#[command(author, version, about, long_version = LONG_VERSION.as_str(), long_about = None)]
pub(crate) struct Cli {
    /// Set logging verbosity.  More v's, more verbose.
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Turn up logging verbosity (multiple will turn it up more)",
        conflicts_with = "quiet",
    )]
    #[getset(get_copy = "pub(crate)")]
    verbose: u8,
    /// Set logging quietness.  More q's, more quiet.
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Turn down logging verbosity (multiple will turn it down more)",
        conflicts_with = "verbose",
    )]
    #[getset(get_copy = "pub(crate)")]
    quiet: u8,
    /// Enable logging to stdout/stderr in additions to the tracing output file
    /// * NOTE * - This should not be used when running as a daemon/service
    #[clap(short, long, help = "Enable logging to stdout/stderr")]
    enable_std_output: bool,
    /// The absolute path to a non-standard config file
    #[clap(short, long, help = "Specify the absolute path to the config file")]
    #[getset(get = "pub(crate)")]
    config_absolute_path: Option<String>,
    /// The absolute path to a non-standard tracing output file
    #[clap(
        short,
        long,
        help = "Specify the absolute path to the tracing output file"
    )]
    #[getset(get = "pub(crate)")]
    tracing_absolute_path: Option<String>,
    #[command(subcommand)]
    #[getset(get = "pub(crate)")]
    command: Commands,
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
        env!("CARGO_PKG_NAME")
            .replace('-', "_")
            .to_ascii_uppercase()
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

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Commands {
    #[clap(about = "Manage barto secrets in the platform keychain (no bartos connection needed)")]
    Secrets(SecretsArgs),
    #[clap(about = "Display the bartos version information")]
    Info {
        /// Output the information in JSON format
        #[clap(
            short,
            long,
            help = "Output the information in JSON format",
            default_value_t = false
        )]
        json: bool,
    },
    #[clap(about = "Check for recent updates on a batoc client")]
    Updates {
        /// The name of the batoc client to check for recent updates
        #[clap(
            short,
            long,
            help = "The name of the batoc client to check for recent updates"
        )]
        name: String,
        /// The kind of updates to check for
        #[clap(short, long, help = "Check for updates of the given kind")]
        update_kind: String,
    },
    #[clap(about = "Perform cleanup of old database entries")]
    Cleanup,
    #[clap(about = "List the currently connected clients")]
    Clients {
        /// Show the bartoc binary version for each connected client
        #[clap(long, help = "Show the bartoc version for each client")]
        versions: bool,
    },
    #[clap(about = "Run a query on bartos")]
    Query {
        /// The query to run on bartos
        #[clap(short, long, help = "The query to run on bartos")]
        query: String,
    },
    #[clap(about = "List the output for the given command")]
    List {
        /// The name of the batoc client to check for recent updates
        #[clap(
            short,
            long,
            help = "The name of the batoc client to check for recent updates"
        )]
        name: String,
        /// The name of the command to list the output for
        #[clap(short, long, help = "The name of the command to list the output for")]
        cmd_name_opt: Option<String>,
    },
    #[clap(about = "List the jobs that failed")]
    Failed,
    #[clap(about = "Display output for the given command name across all clients")]
    Cmd {
        /// The name of the command to display output for
        #[clap(help = "The name of the command to display output for")]
        cmd_name: String,
    },
}

/// Wrapper so `secrets` appears as a subcommand with its own sub-subcommands.
#[derive(Clone, Debug, clap::Args)]
pub(crate) struct SecretsArgs {
    #[command(subcommand)]
    pub(crate) command: SecretsSubcommand,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum SecretsSubcommand {
    /// Store a secret value in the platform keychain.
    /// Prompts for the value without echoing it.
    Set {
        /// Name of the secret (e.g. `BARTOC_HMAC_KEY`)
        key: String,
    },
    /// Retrieve and print a secret from the platform keychain.
    Get {
        /// Name of the secret to retrieve
        key: String,
    },
    /// List known barto secrets and whether each is currently stored.
    List,
    /// Delete a secret from the platform keychain.
    Delete {
        /// Name of the secret to delete
        key: String,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use config::Source;
    use libbarto::PathDefaults;

    use super::{Cli, Commands, SecretsSubcommand};

    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("barto-cli").chain(args.iter().copied()))
    }

    #[test]
    fn defaults_are_zero() {
        let cli = parse(&["info"]);
        assert_eq!(cli.verbose(), 0);
        assert_eq!(cli.quiet(), 0);
        assert!(cli.config_absolute_path().is_none());
        assert!(cli.tracing_absolute_path().is_none());
    }

    #[test]
    fn verbose_flag_increments() {
        let cli = parse(&["-v", "-v", "info"]);
        assert_eq!(cli.verbose(), 2);
    }

    #[test]
    fn quiet_flag_increments() {
        let cli = parse(&["-q", "-q", "-q", "info"]);
        assert_eq!(cli.quiet(), 3);
    }

    #[test]
    fn config_absolute_path_set() {
        let cli = parse(&["-c", "/etc/barto-cli.toml", "info"]);
        assert_eq!(
            cli.config_absolute_path().as_deref(),
            Some("/etc/barto-cli.toml")
        );
    }

    #[test]
    fn tracing_absolute_path_set() {
        let cli = parse(&["-t", "/var/log/barto-cli.log", "info"]);
        assert_eq!(
            cli.tracing_absolute_path().as_deref(),
            Some("/var/log/barto-cli.log")
        );
    }

    #[test]
    fn command_info() {
        assert!(matches!(
            parse(&["info"]).command(),
            Commands::Info { json: false }
        ));
        assert!(matches!(
            parse(&["info", "--json"]).command(),
            Commands::Info { json: true }
        ));
    }

    #[test]
    fn command_updates() {
        match parse(&["updates", "-n", "host1", "-u", "garuda"]).command() {
            Commands::Updates { name, update_kind } => {
                assert_eq!(name, "host1");
                assert_eq!(update_kind, "garuda");
            }
            other => panic!("expected Updates, got {other:?}"),
        }
    }

    #[test]
    fn command_cleanup() {
        assert!(matches!(parse(&["cleanup"]).command(), Commands::Cleanup));
    }

    #[test]
    fn command_clients() {
        assert!(matches!(
            parse(&["clients"]).command(),
            Commands::Clients { versions: false }
        ));
        assert!(matches!(
            parse(&["clients", "--versions"]).command(),
            Commands::Clients { versions: true }
        ));
    }

    #[test]
    fn command_query() {
        match parse(&["query", "-q", "select 1"]).command() {
            Commands::Query { query } => assert_eq!(query, "select 1"),
            other => panic!("expected Query, got {other:?}"),
        }
    }

    #[test]
    fn command_list() {
        match parse(&["list", "-n", "host1"]).command() {
            Commands::List { name, cmd_name_opt } => {
                assert_eq!(name, "host1");
                assert!(cmd_name_opt.is_none());
            }
            other => panic!("expected List, got {other:?}"),
        }
        match parse(&["list", "-n", "host1", "-c", "backup"]).command() {
            Commands::List { name, cmd_name_opt } => {
                assert_eq!(name, "host1");
                assert_eq!(cmd_name_opt.as_deref(), Some("backup"));
            }
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn command_failed() {
        assert!(matches!(parse(&["failed"]).command(), Commands::Failed));
    }

    #[test]
    fn command_cmd() {
        match parse(&["cmd", "backup"]).command() {
            Commands::Cmd { cmd_name } => assert_eq!(cmd_name, "backup"),
            other => panic!("expected Cmd, got {other:?}"),
        }
    }

    fn secrets_subcommand(args: &[&str]) -> SecretsSubcommand {
        match parse(args).command() {
            Commands::Secrets(secrets) => secrets.command.clone(),
            other => panic!("expected Secrets, got {other:?}"),
        }
    }

    #[test]
    fn command_secrets_set() {
        match secrets_subcommand(&["secrets", "set", "K"]) {
            SecretsSubcommand::Set { key } => assert_eq!(key, "K"),
            other => panic!("expected Set, got {other:?}"),
        }
    }

    #[test]
    fn command_secrets_get() {
        match secrets_subcommand(&["secrets", "get", "K"]) {
            SecretsSubcommand::Get { key } => assert_eq!(key, "K"),
            other => panic!("expected Get, got {other:?}"),
        }
    }

    #[test]
    fn command_secrets_list() {
        assert!(matches!(
            secrets_subcommand(&["secrets", "list"]),
            SecretsSubcommand::List
        ));
    }

    #[test]
    fn command_secrets_delete() {
        match secrets_subcommand(&["secrets", "delete", "K"]) {
            SecretsSubcommand::Delete { key } => assert_eq!(key, "K"),
            other => panic!("expected Delete, got {other:?}"),
        }
    }

    #[test]
    fn source_collect_basic_keys() {
        let cli = parse(&["-v", "info"]);
        let map = cli.collect().expect("collect");
        assert!(map.contains_key("verbose"));
        assert!(map.contains_key("quiet"));
        assert!(map.contains_key("enable_std_output"));
        // No path options provided, so these must be absent.
        assert!(!map.contains_key("config_path"));
        assert!(!map.contains_key("tracing_path"));
    }

    #[test]
    fn source_collect_includes_paths_when_set() {
        let cli = parse(&["-c", "/a.toml", "-t", "/b.log", "info"]);
        let map = cli.collect().expect("collect");
        assert!(map.contains_key("config_path"));
        assert!(map.contains_key("tracing_path"));
    }

    #[test]
    fn path_defaults_env_prefix() {
        assert_eq!(parse(&["info"]).env_prefix(), "BARTO_CLI");
    }

    #[test]
    fn path_defaults_default_file_path_and_name() {
        let cli = parse(&["info"]);
        assert_eq!(cli.default_file_path(), "barto-cli");
        assert_eq!(cli.default_file_name(), "barto-cli");
    }

    #[test]
    fn path_defaults_tracing_defaults() {
        let cli = parse(&["info"]);
        assert_eq!(cli.default_tracing_path(), "barto-cli/logs");
        assert_eq!(cli.default_tracing_file_name(), "barto-cli");
    }

    #[test]
    fn path_defaults_round_trip_paths() {
        let cli = parse(&["-c", "/a.toml", "-t", "/b.log", "info"]);
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
