// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::path::PathBuf;

use anyhow::{Context, Result};
use bincode::{Decode, Encode};
use config::{Config, Environment, File, FileFormat, Source};
use dirs2::config_dir;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{error::Error, utils::to_path_buf};

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
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Tracing {
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

/// hosts configuration
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
#[getset(get = "pub")]
pub struct Actix {
    /// The number of workers to start
    workers: u8,
    /// The IP address to listen on
    ip: String,
    /// The port to listen on
    port: u16,
}

/// bartos configuration for clients
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Bartos {
    /// The number of workers to start
    #[getset(get = "pub")]
    prefix: String,
    /// The IP address to listen on
    #[getset(get = "pub")]
    host: String,
    /// The port to listen on
    #[getset(get_copy = "pub")]
    port: u16,
}

/// hosts configuration
#[derive(Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Mariadb {
    /// The username for the database
    #[getset(get = "pub")]
    username: String,
    /// The password for the database
    #[getset(get = "pub")]
    password: String,
    /// The database name
    #[getset(get = "pub")]
    database: String,
    /// The output table name
    #[getset(get_copy = "pub")]
    output_table: OutputTableName,
    /// The status table name
    #[getset(get_copy = "pub")]
    status_table: StatusTableName,
}

/// The output table name
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

/// The schedule to run commands on a given worker client
#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, Getters, PartialEq, Serialize)]
#[getset(get = "pub")]
pub struct Schedules {
    /// All of the schedules for a worker client
    schedules: Vec<Schedule>,
}

/// A schedule
#[derive(Clone, Debug, Decode, Default, Deserialize, Encode, Eq, Getters, PartialEq, Serialize)]
#[getset(get = "pub")]
pub struct Schedule {
    /// A calendar string similar to cron format
    on_calendar: String,
    /// The commands to run
    cmds: Vec<String>,
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
                .separator("_")
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
