// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::path::PathBuf;

use anyhow::{Context, Result};
use bincode_next::{Decode, Encode};
#[cfg(test)]
use bon::Builder;
use config::{Config, Environment, File, FileFormat, Source};
use dirs2::config_dir;
use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber_init::{TracingConfig, get_effective_level};

#[cfg(test)]
use crate::utils::Mock;
use crate::{TlsConfig, TracingConfigExt, error::Error, utils::to_path_buf};

/// Trait to allow default paths to be supplied to [`load`]
pub trait PathDefaults {
    /// Environment variable prefix
    fn env_prefix(&self) -> String;
    /// The absolute path to use for the config file
    fn config_absolute_path(&self) -> Option<String>;
    /// The default file path to use
    fn default_file_path(&self) -> String;
    /// The default file name to use
    fn default_file_name(&self) -> String;
    /// The abolute path to use for tracing output
    fn tracing_absolute_path(&self) -> Option<String>;
    /// The default logging path to use
    fn default_tracing_path(&self) -> String;
    /// The default log file name to use
    fn default_tracing_file_name(&self) -> String;
}

/// Tracing configuration
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Tracing {
    /// stdout layer configuration
    #[getset(get = "pub")]
    stdout: Layer,
    /// file layer configuration
    #[getset(get = "pub")]
    file: FileLayer,
}

/// Tracing configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Setters)]
pub struct FileLayer {
    /// quiet level
    quiet: u8,
    /// verbose level
    verbose: u8,
    /// layer configuration
    layer: Layer,
}

impl TracingConfig for FileLayer {
    fn quiet(&self) -> u8 {
        self.quiet
    }

    fn verbose(&self) -> u8 {
        self.verbose
    }

    fn with_ansi(&self) -> bool {
        false
    }

    fn with_target(&self) -> bool {
        self.layer.with_target
    }

    fn with_thread_ids(&self) -> bool {
        self.layer.with_thread_ids
    }

    fn with_thread_names(&self) -> bool {
        self.layer.with_thread_names
    }

    fn with_line_number(&self) -> bool {
        self.layer.with_line_number
    }

    fn with_level(&self) -> bool {
        self.layer.with_level
    }
}

impl TracingConfigExt for FileLayer {
    fn enable_stdout(&self) -> bool {
        false
    }

    fn directives(&self) -> Option<&String> {
        self.layer.directives.as_ref()
    }

    fn level(&self) -> Level {
        get_effective_level(self.quiet(), self.verbose())
    }
}

/// Tracing configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Layer {
    /// Should we trace the event target
    #[getset(get_copy = "pub")]
    with_target: bool,
    /// Should we trace the thread id
    #[getset(get_copy = "pub")]
    with_thread_ids: bool,
    /// Should we trace the thread names
    #[getset(get_copy = "pub")]
    with_thread_names: bool,
    /// Should we trace the line numbers
    #[getset(get_copy = "pub")]
    with_line_number: bool,
    /// Should we trace the level
    #[getset(get_copy = "pub")]
    with_level: bool,
    /// Additional tracing directives
    #[getset(get = "pub")]
    directives: Option<String>,
}

/// A command to run on a worker
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
#[getset(get = "pub")]
pub struct Command {
    /// The command to run
    cmd: String,
}

/// Configuration for the Actix web server
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
#[getset(get = "pub")]
pub struct Actix {
    /// The number of workers to start
    workers: u8,
    /// The IP address to listen on
    ip: String,
    /// The port to listen on
    port: u16,
    /// The optional TLS configuration
    tls: Option<Tls>,
}

/// TLS configuration for the Actix web server
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Tls {
    /// The IP address to listen on
    #[getset(get = "pub")]
    ip: String,
    /// The port to listen on
    #[getset(get_copy = "pub")]
    port: u16,
    /// The path to the certificate file (PEM format)
    #[getset(get = "pub")]
    cert_file_path: String,
    /// The path to the key file (PEM format)
    #[getset(get = "pub")]
    key_file_path: String,
    /// Optional path to a CA certificate for mutual TLS client authentication.
    /// When set, clients must present a certificate signed by this CA.
    #[getset(get = "pub")]
    #[serde(default)]
    client_ca_cert: Option<PathBuf>,
}

impl TlsConfig for Tls {
    fn cert_file_path(&self) -> &str {
        &self.cert_file_path
    }

    fn key_file_path(&self) -> &str {
        &self.key_file_path
    }

    fn client_ca_cert_path(&self) -> Option<&std::path::Path> {
        self.client_ca_cert.as_deref()
    }
}

/// Used in bartoc configuration to define the bartos instance to connect to
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Bartos {
    /// The websocket prefix (ws or wss)
    #[getset(get = "pub")]
    prefix: String,
    /// The bartos hostname or IP
    #[getset(get = "pub")]
    host: String,
    /// The bartos port
    #[getset(get_copy = "pub")]
    port: u16,
    /// Optional path to a PEM CA certificate to pin for the bartos TLS connection.
    /// When set, only this CA is trusted instead of the system/Mozilla root CA store.
    #[getset(get = "pub")]
    #[serde(default)]
    ca_cert: Option<PathBuf>,
    /// Optional path to a client certificate PEM file for mutual TLS authentication.
    /// Both `client_cert` and `client_key` must be set together.
    #[getset(get = "pub")]
    #[serde(default)]
    client_cert: Option<PathBuf>,
    /// Optional path to a client private key PEM file for mutual TLS authentication.
    /// Both `client_cert` and `client_key` must be set together.
    #[getset(get = "pub")]
    #[serde(default)]
    client_key: Option<PathBuf>,
    /// Optional pre-shared token for Bearer authentication on the WebSocket upgrade.
    /// When set, the client sends `Authorization: Bearer <api_key>` on the WS upgrade request.
    /// Must match `api_key` in bartos.toml.
    #[getset(get = "pub")]
    #[serde(default)]
    api_key: Option<String>,
}

/// The `MariaDB` configuration
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Mariadb {
    /// The host or IP for the database
    host: String,
    /// The port for the database
    port: Option<u16>,
    /// The username for the database
    username: String,
    /// The password for the database
    password: String,
    /// The database name
    database: String,
    /// The options string
    options: Option<String>,
    /// The output table name, used for testing
    #[doc(hidden)]
    #[getset(get_copy = "pub")]
    #[serde(default = "OutputTableName::default")]
    output_table: OutputTableName,
    /// The status table name, used for testing
    #[doc(hidden)]
    #[getset(get_copy = "pub")]
    #[serde(default = "StatusTableName::default")]
    status_table: StatusTableName,
}

impl Mariadb {
    /// Generate the `MariaDB` connection string
    #[must_use]
    pub fn connection_string(&self) -> String {
        let mut url = format!(
            "mariadb://{}:{}@{}:{}/{}",
            self.username,
            self.password,
            self.host,
            self.port.unwrap_or(3306),
            self.database
        );
        if let Some(options) = self.options.as_ref() {
            url.push('?');
            url.push_str(options);
        }
        url
    }

    /// Generate a displayable `MariaDB` connection string
    #[must_use]
    pub fn disp_connection_string(&self) -> String {
        let mut url = format!(
            "mariadb://{}:****@{}:{}/{}",
            self.username,
            self.host,
            self.port.unwrap_or(3306),
            self.database
        );
        if let Some(options) = self.options.as_ref() {
            url.push('?');
            url.push_str(options);
        }
        url
    }
}
/// The output table name
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum OutputTableName {
    /// Use the default table name `output`
    #[default]
    Output,
    /// Use the test table name `output_test`
    OutputTest,
}

impl From<OutputTableName> for &'static str {
    fn from(value: OutputTableName) -> Self {
        match value {
            OutputTableName::Output => "output",
            OutputTableName::OutputTest => "output_test",
        }
    }
}

/// The status table name
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum StatusTableName {
    /// Use the default table name `status`
    #[default]
    Status,
    /// Use the test table name `status_test`
    StatusTest,
}

impl From<StatusTableName> for &'static str {
    fn from(value: StatusTableName) -> Self {
        match value {
            StatusTableName::Status => "status",
            StatusTableName::StatusTest => "status_test",
        }
    }
}

/// The output table name
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum MissedTick {
    /// `MissedTickBehavior::Burst`
    #[default]
    Burst,
    /// `MissedTickBehavior::Delay`
    Delay,
    /// `MissedTickBehavior::Skip`
    Skip,
}

/// The schedule to run commands on a given worker client
#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, Getters, PartialEq, Serialize)]
#[cfg_attr(test, derive(Builder))]
#[getset(get = "pub")]
pub struct Schedules {
    /// All of the schedules for a worker client
    schedules: Vec<Schedule>,
}

#[cfg(test)]
impl Mock for Schedules {
    fn mock() -> Self {
        Self::builder()
            .schedules(vec![Schedule::mock(), Schedule::mock()])
            .build()
    }
}

/// A schedule
#[derive(Clone, Debug, Decode, Default, Deserialize, Encode, Eq, Getters, PartialEq, Serialize)]
#[cfg_attr(test, derive(Builder))]
#[getset(get = "pub")]
pub struct Schedule {
    /// The name of the schedule
    name: String,
    /// A calendar string similar to cron format
    on_calendar: String,
    /// The commands to run
    cmds: Vec<String>,
}

#[cfg(test)]
impl Mock for Schedule {
    fn mock() -> Self {
        Self::builder()
            .name("mock_schedule".to_string())
            .on_calendar("* * * * *".to_string())
            .cmds(vec!["echo 'Hello, World!'".to_string()])
            .build()
    }
}

/// Load the configuration
///
/// # Errors
/// - [`Error::ConfigDir`] - No valid config directory could be found
/// - [`Error::ConfigBuild`] - Unable to build a valid configuration
/// - [`Error::ConfigDeserialize`] - Unable to deserialize configuration
/// - Any other error encountered while trying to read the config file
///
pub fn load<'a, S, T, D>(cli: &S, defaults: &D) -> Result<T>
where
    T: Deserialize<'a>,
    S: Source + Clone + Send + Sync + 'static,
    D: PathDefaults,
{
    let config_file_path = config_file_path(defaults)?;
    let config = Config::builder()
        .add_source(
            Environment::with_prefix(&defaults.env_prefix())
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        )
        .add_source(cli.clone())
        .add_source(File::from(config_file_path).format(FileFormat::Toml))
        .build()
        .with_context(|| Error::ConfigBuild)?;
    config
        .try_deserialize::<T>()
        .with_context(|| Error::ConfigDeserialize)
}

fn config_file_path<D>(defaults: &D) -> Result<PathBuf>
where
    D: PathDefaults,
{
    let default_fn = || -> Result<PathBuf> { default_config_file_path(defaults) };
    defaults
        .config_absolute_path()
        .as_ref()
        .map_or_else(default_fn, to_path_buf)
}

fn default_config_file_path<D>(defaults: &D) -> Result<PathBuf>
where
    D: PathDefaults,
{
    let mut config_file_path = config_dir().ok_or(Error::ConfigDir)?;
    config_file_path.push(defaults.default_file_path());
    config_file_path.push(defaults.default_file_name());
    let _ = config_file_path.set_extension("toml");
    Ok(config_file_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::TlsConfig;

    use super::{Bartos, PathDefaults, Tls, load};

    #[test]
    fn test_bartos_client_cert_key_default_none() {
        let bartos = Bartos::default();
        assert!(bartos.client_cert().is_none());
        assert!(bartos.client_key().is_none());
        assert!(bartos.ca_cert().is_none());
        assert!(bartos.api_key().is_none());
    }

    #[test]
    fn test_tls_client_ca_cert_default_none() {
        let tls = Tls::default();
        assert!(tls.client_ca_cert().is_none());
        assert!(tls.client_ca_cert_path().is_none());
    }

    #[test]
    fn test_tls_client_ca_cert_path_returns_some() {
        let tls = Tls {
            ip: "0.0.0.0".to_string(),
            port: 8443,
            cert_file_path: "cert.pem".to_string(),
            key_file_path: "key.pem".to_string(),
            client_ca_cert: Some(PathBuf::from("/etc/bartos/client-ca.pem")),
        };
        assert_eq!(
            tls.client_ca_cert_path(),
            Some(std::path::Path::new("/etc/bartos/client-ca.pem"))
        );
    }

    #[test]
    fn test_bartos_client_cert_key_returns_some() {
        let bartos = Bartos {
            prefix: "wss".to_string(),
            host: "localhost".to_string(),
            port: 8443,
            ca_cert: None,
            client_cert: Some(PathBuf::from("/etc/bartoc/client.pem")),
            client_key: Some(PathBuf::from("/etc/bartoc/client.key")),
            api_key: None,
        };
        assert_eq!(
            bartos.client_cert().as_deref(),
            Some(std::path::Path::new("/etc/bartoc/client.pem"))
        );
        assert_eq!(
            bartos.client_key().as_deref(),
            Some(std::path::Path::new("/etc/bartoc/client.key"))
        );
    }

    // Verify that the env var separator fix works: a flat field whose name contains
    // an underscore (e.g. `my_field`) must be populated from `PREFIX_MY_FIELD`, not
    // treated as a nested path `my.field` (which was the old behaviour).
    #[test]
    #[cfg_attr(nightly, allow(unsafe_code))]
    fn test_load_flat_env_var_with_underscore() {
        use std::env;

        use config::{ConfigError, Map, Source, Value};
        use serde::Deserialize;
        use tempfile::NamedTempFile;

        #[derive(Debug, Deserialize, PartialEq)]
        struct TestCfg {
            my_field: Option<String>,
        }

        #[derive(Debug, Clone)]
        struct NoOpSource;
        impl Source for NoOpSource {
            fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
                Box::new(self.clone())
            }
            fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
                Ok(Map::new())
            }
        }

        struct TestDefaults {
            path: String,
        }
        impl PathDefaults for TestDefaults {
            fn env_prefix(&self) -> String {
                "LBCFGTEST".to_string()
            }
            fn config_absolute_path(&self) -> Option<String> {
                Some(self.path.clone())
            }
            fn default_file_path(&self) -> String {
                String::new()
            }
            fn default_file_name(&self) -> String {
                String::new()
            }
            fn tracing_absolute_path(&self) -> Option<String> {
                None
            }
            fn default_tracing_path(&self) -> String {
                String::new()
            }
            fn default_tracing_file_name(&self) -> String {
                String::new()
            }
        }

        let toml = NamedTempFile::new().unwrap();
        std::fs::write(toml.path(), b"").unwrap();
        let defaults = TestDefaults {
            path: toml.path().to_str().unwrap().to_string(),
        };

        // SAFETY: single-threaded test, no other thread reads this env var
        unsafe { env::set_var("LBCFGTEST_MY_FIELD", "flatval") };
        let cfg: TestCfg = load(&NoOpSource, &defaults).unwrap();
        // SAFETY: same as above
        unsafe { env::remove_var("LBCFGTEST_MY_FIELD") };

        assert_eq!(cfg.my_field.as_deref(), Some("flatval"));
    }
}
