// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

// Header

use crate::TracingConfigExt;
use anyhow::Result;
use console::Style;
use rand::Rng;
use std::io::Write;
use tracing::Level;
use vergen_pretty::{Prefix, Pretty, vergen_pretty_env};

fn from_u8(val: u8) -> Style {
    let style = Style::new();
    match val {
        0 => style.green(),
        1 => style.yellow(),
        2 => style.blue(),
        3 => style.magenta(),
        4 => style.cyan(),
        5 => style.white(),
        _ => style.red(),
    }
}

/// Generate a pretty header
///
/// # Errors
///
pub fn header<T, U>(config: &T, prefix: &'static str, writer: Option<&mut U>) -> Result<()>
where
    T: TracingConfigExt,
    U: Write + ?Sized,
{
    let mut rng = rand::rng();
    let app_style = from_u8(rng.random_range(0..7));
    if let Some(writer) = writer {
        output_to_writer(writer, app_style.clone(), prefix)?;
    }
    if config.level() >= Level::INFO {
        trace(app_style, prefix);
    }
    Ok(())
}

fn output_to_writer<T>(writer: &mut T, app_style: Style, prefix: &'static str) -> Result<()>
where
    T: Write + ?Sized,
{
    let prefix = Prefix::builder()
        .lines(prefix.lines().map(str::to_string).collect())
        .style(app_style)
        .build();
    Pretty::builder()
        .env(vergen_pretty_env!())
        .prefix(prefix)
        .build()
        .display(writer)?;
    Ok(())
}

fn trace(app_style: Style, prefix: &'static str) {
    let prefix = Prefix::builder()
        .lines(prefix.lines().map(str::to_string).collect())
        .style(app_style)
        .build();
    Pretty::builder()
        .env(vergen_pretty_env!())
        .prefix(prefix)
        .build()
        .trace();
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use super::{from_u8, header};
    use crate::{TracingConfigExt, utils::test::TestConfig};
    use console::Style;
    use regex::Regex;
    use tracing::Level;
    use tracing_subscriber_init::TracingConfig;

    const HEADER_PREFIX: &str = r"██████╗ ██╗   ██╗██████╗ ██╗    ██╗
██╔══██╗██║   ██║██╔══██╗██║    ██║
██████╔╝██║   ██║██║  ██║██║ █╗ ██║
██╔═══╝ ██║   ██║██║  ██║██║███╗██║
██║     ╚██████╔╝██████╔╝╚███╔███╔╝
╚═╝      ╚═════╝ ╚═════╝  ╚══╝╚══╝ 

4a61736f6e204f7a696173
";

    static BUILD_TIMESTAMP: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Timestamp \(  build\)").unwrap());
    static BUILD_SEMVER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Semver \(  rustc\)").unwrap());
    static GIT_BRANCH: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Branch \(    git\)").unwrap());

    #[test]
    fn from_u8_works() {
        assert_eq!(from_u8(0), Style::new().green());
        assert_eq!(from_u8(1), Style::new().yellow());
        assert_eq!(from_u8(2), Style::new().blue());
        assert_eq!(from_u8(3), Style::new().magenta());
        assert_eq!(from_u8(4), Style::new().cyan());
        assert_eq!(from_u8(5), Style::new().white());
        assert_eq!(from_u8(6), Style::new().red());
        assert_eq!(from_u8(7), Style::new().red());
    }

    #[test]
    #[cfg(debug_assertions)]
    fn header_writes() {
        let mut buf = vec![];
        let config = TestConfig::default();
        assert!(config.quiet() == 0);
        assert!(config.verbose() == 3);
        assert!(config.level() == Level::INFO);
        assert!(config.enable_stdout());
        assert!(config.directives().is_none());
        assert!(header(&config, HEADER_PREFIX, Some(&mut buf)).is_ok());
        assert!(!buf.is_empty());
        let header_str = String::from_utf8_lossy(&buf);
        assert!(BUILD_TIMESTAMP.is_match(&header_str));
        assert!(BUILD_SEMVER.is_match(&header_str));
        assert!(GIT_BRANCH.is_match(&header_str));
    }

    #[test]
    fn none_writer_skips_header() {
        let config = TestConfig::default();
        assert!(config.quiet() == 0);
        assert!(config.verbose() == 3);
        assert!(config.level() == Level::INFO);
        assert!(config.enable_stdout());
        assert!(config.directives().is_none());
        assert!(header(&config, HEADER_PREFIX, None::<&mut Vec<u8>>).is_ok());
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn header_writes() {
        let mut buf = vec![];
        let config = TestConfig::default();
        assert!(config.quiet() == 0);
        assert!(config.verbose() == 3);
        assert!(config.level() == Level::INFO);
        assert!(config.enable_stdout());
        assert!(config.directives().is_none());
        assert!(header(&config, HEADER_PREFIX, Some(&mut buf)).is_ok());
        assert!(!buf.is_empty());
        let header_str = String::from_utf8_lossy(&buf);
        assert!(BUILD_TIMESTAMP.is_match(&header_str));
        assert!(BUILD_SEMVER.is_match(&header_str));
        assert!(GIT_BRANCH.is_match(&header_str));
    }
}
