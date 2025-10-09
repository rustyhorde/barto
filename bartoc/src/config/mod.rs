// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::path::PathBuf;

use anyhow::{Context, Result};
use config::Source;
use dirs2::data_dir;
use getset::{CopyGetters, Getters, Setters};
use libbarto::{PathDefaults, Tracing, TracingConfigExt, load, to_path_buf};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber_init::{TracingConfig, get_effective_level};

use crate::error::Error;

pub(crate) trait PathDefaultsExt: PathDefaults {
    fn redb_absolute_path(&self) -> Option<String>;
    fn default_redb_path(&self) -> String;
    fn default_redb_file_name(&self) -> String;
}

#[derive(
    Clone, CopyGetters, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize, Setters,
)]
pub(crate) struct Config {
    #[getset(get_copy = "pub(crate)")]
    verbose: u8,
    #[getset(get_copy = "pub(crate)")]
    quiet: u8,
    #[getset(get_copy = "pub(crate)")]
    enable_std_output: bool,
    #[getset(get = "pub(crate)")]
    tracing: Tracing,
    #[getset(get = "pub(crate)", set)]
    redb_path: Option<PathBuf>,
    #[getset(get = "pub(crate)")]
    name: String,
    #[getset(get = "pub(crate)")]
    bartos_prefix: String,
    #[getset(get = "pub(crate)")]
    bartos_host: String,
    #[getset(get = "pub(crate)")]
    bartos_port: u16,
    #[getset(get = "pub(crate)")]
    retry_count: u8,
    #[getset(get_copy = "pub(crate)")]
    client_timeout: Option<u64>,
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
        self.enable_std_output
    }

    fn directives(&self) -> Option<&String> {
        self.tracing.directives().as_ref()
    }

    fn level(&self) -> Level {
        get_effective_level(self.quiet, self.verbose)
    }
}

pub(crate) fn load_bartoc<S, D>(cli: &S, defaults: &D) -> Result<Config>
where
    S: Source + Clone + Send + Sync + 'static,
    D: PathDefaultsExt,
{
    let mut config: Config = load(cli, defaults).with_context(|| Error::ConfigLoad)?;
    let _ = config.set_redb_path(Some(
        redb_file_path(defaults).with_context(|| Error::ConfigLoad)?,
    ));
    Ok(config)
}

fn redb_file_path<D>(defaults: &D) -> Result<PathBuf>
where
    D: PathDefaultsExt,
{
    let default_fn = || -> Result<PathBuf> { default_redb_file_path(defaults) };
    defaults
        .redb_absolute_path()
        .as_ref()
        .map_or_else(default_fn, to_path_buf)
}

fn default_redb_file_path<D>(defaults: &D) -> Result<PathBuf>
where
    D: PathDefaultsExt,
{
    let mut config_file_path = data_dir().ok_or(Error::DataDir)?;
    config_file_path.push(defaults.default_redb_path());
    config_file_path.push(defaults.default_redb_file_name());
    let _ = config_file_path.set_extension("redb");
    Ok(config_file_path)
}
