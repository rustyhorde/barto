// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::path::PathBuf;

use anyhow::Result;
use dirs2::data_dir;
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt::time::UtcTime};
use tracing_subscriber_init::{Iso8601, TracingConfig, compact, try_init};

use crate::{Error, PathDefaults, utils::to_path_buf};

/// Extension trait for `TracingConfig` to add additional configuration options
pub trait TracingConfigExt: TracingConfig {
    /// Should we enable stdout logging
    fn enable_stdout(&self) -> bool;
    /// Additional tracing directives
    fn directives(&self) -> Option<&String>;
    /// Current tracing level
    fn level(&self) -> Level;
}

/// Initialize tracing
///
/// # Errors
/// * If the path to the tracing file cannot be created or found
///
pub fn init_tracing<T, U, V>(
    stdout: &T,
    file: &V,
    defaults: &U,
    layers_opt: Option<Vec<Box<dyn Layer<Registry> + Send + Sync>>>,
) -> Result<()>
where
    T: TracingConfigExt,
    U: PathDefaults,
    V: TracingConfigExt,
{
    let mut layers = layers_opt.unwrap_or_default();

    // Setup the stdout tracing layer if enabled
    if stdout.enable_stdout() {
        let (layer, level_filter) = compact(stdout);
        let mut directives = directives(stdout, level_filter);
        directives.push_str(",vergen_pretty=error");
        let filter = EnvFilter::builder()
            .with_default_directive(level_filter.into())
            .parse_lossy(directives);
        let stdout_layer = layer
            .with_timer(UtcTime::new(Iso8601::DEFAULT))
            .with_filter(filter);
        layers.push(stdout_layer.boxed());
    }

    // Setup the tracing file layer
    let (directory, logfile) = tracing_absolute_path(defaults)?;
    let tracing_file = RollingFileAppender::new(Rotation::DAILY, directory, logfile);
    let (layer, _level_filter) = compact(file);
    let level_filter = LevelFilter::from(file.level());
    let directives = directives(file, level_filter);
    let filter = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .parse_lossy(directives);
    let file_layer = layer
        .with_timer(UtcTime::new(Iso8601::DEFAULT))
        .with_writer(tracing_file)
        .with_filter(filter);
    layers.push(file_layer.boxed());

    try_init(layers)?;
    Ok(())
}

fn directives<T>(config: &T, level_filter: LevelFilter) -> String
where
    T: TracingConfigExt,
{
    let directives_base = match level_filter.into_level() {
        Some(level) => match level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        },
        None => "info",
    };

    if let Some(directives) = config.directives() {
        format!("{directives_base},{directives}")
    } else {
        directives_base.to_string()
    }
}

fn tracing_absolute_path<D>(defaults: &D) -> Result<(PathBuf, PathBuf)>
where
    D: PathDefaults,
{
    let default_fn = || -> Result<PathBuf> { default_tracing_absolute_path(defaults) };
    let mut full_path = defaults
        .tracing_absolute_path()
        .as_ref()
        .map_or_else(default_fn, to_path_buf)?;

    let file_path = PathBuf::from(full_path.file_name().ok_or(Error::DataDir)?);
    let _ = full_path.pop();
    Ok((full_path, file_path))
}

#[allow(clippy::unnecessary_wraps)]
fn default_tracing_absolute_path<D>(defaults: &D) -> Result<PathBuf>
where
    D: PathDefaults,
{
    let mut config_file_path = data_dir().ok_or(Error::DataDir)?;
    config_file_path.push(defaults.default_tracing_path());
    config_file_path.push(defaults.default_tracing_file_name());
    let _ = config_file_path.set_extension("log");
    Ok(config_file_path)
}
