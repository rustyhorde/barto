// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::Result;
use bytes::Bytes;
use strip_ansi_escapes::strip;
use unicode_width::UnicodeWidthStr as _;

/// Convert a string to a `PathBuf`
///
/// # Errors
/// * This function never errors, but is wrapped to use with `map_or_else` and similar
///
#[allow(clippy::unnecessary_wraps)]
pub fn to_path_buf(path: &String) -> Result<PathBuf> {
    Ok(PathBuf::from(path))
}

/// Send a timestamp ping
#[must_use]
pub fn send_ts_ping(origin: Instant) -> [u8; 12] {
    let ts = Instant::now().duration_since(origin);
    let (ts1, ts2) = (ts.as_secs(), ts.subsec_nanos());
    let mut ts = [0; 12];
    ts[0..8].copy_from_slice(&ts1.to_be_bytes());
    ts[8..12].copy_from_slice(&ts2.to_be_bytes());
    ts
}

/// Parse a received timestamp ping
pub fn parse_ts_ping(bytes: &Bytes) -> Option<Duration> {
    if bytes.len() == 12 {
        let secs_bytes = <[u8; 8]>::try_from(&bytes[0..8]).unwrap_or([0; 8]);
        let nanos_bytes = <[u8; 4]>::try_from(&bytes[8..12]).unwrap_or([0; 4]);
        let secs = u64::from_be_bytes(secs_bytes);
        let nanos = u32::from_be_bytes(nanos_bytes);
        Some(Duration::new(secs, nanos))
    } else {
        None
    }
}

#[allow(clippy::mut_mut)]
pub(crate) fn until_err<T>(err: &mut &mut Result<()>, item: Result<T>) -> Option<T> {
    match item {
        Ok(item) => Some(item),
        Err(e) => {
            **err = Err(e);
            None
        }
    }
}

/// Clean an output string by removing tabs, new lines, carriage returns, and ANSI escape codes.
#[must_use]
pub fn clean_output_string(data: &str) -> (String, usize) {
    let data = data.replace('\t', "   ");
    let data = data.replace('\n', " ");
    let data = data.replace('\r', " ");
    let final_data = String::from_utf8_lossy(&strip(data)).to_string();
    let data_uw = final_data.width();
    (final_data, data_uw)
}

#[cfg(test)]
pub(crate) trait Mock {
    fn mock() -> Self;
}

#[cfg(test)]
pub(crate) mod test {
    use std::time::Instant;

    use bytes::Bytes;
    use tracing::Level;
    use tracing_subscriber_init::TracingConfig;
    use unicode_width::UnicodeWidthStr as _;

    use crate::TracingConfigExt;

    use super::{clean_output_string, parse_ts_ping, send_ts_ping, to_path_buf};

    pub(crate) struct TestConfig {
        verbose: u8,
        quiet: u8,
        level: Level,
        directives: Option<String>,
    }

    impl TestConfig {
        pub(crate) fn with_directives() -> Self {
            Self {
                verbose: 3,
                quiet: 0,
                level: Level::INFO,
                directives: Some("actix_web=error".to_string()),
            }
        }
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                verbose: 3,
                quiet: 0,
                level: Level::INFO,
                directives: None,
            }
        }
    }

    impl TracingConfig for TestConfig {
        fn quiet(&self) -> u8 {
            self.quiet
        }

        fn verbose(&self) -> u8 {
            self.verbose
        }
    }

    impl TracingConfigExt for TestConfig {
        fn level(&self) -> Level {
            self.level
        }

        fn enable_stdout(&self) -> bool {
            false
        }

        fn directives(&self) -> Option<&String> {
            self.directives.as_ref()
        }
    }

    #[test]
    fn test_to_path_buf() {
        let path_str = String::from("/some/test/path");
        let path_buf = to_path_buf(&path_str).unwrap();
        assert_eq!(path_buf.to_str().unwrap(), "/some/test/path");
    }

    #[test]
    fn test_clean_output_string() {
        let input = "Hello,\tWorld!\nThis is a test.\r\x1b[31mRed Text\x1b[0m";
        let (cleaned, width) = clean_output_string(input);
        assert_eq!(cleaned, "Hello,   World! This is a test. Red Text");
        assert_eq!(width, cleaned.width()); // Ensure width matches cleaned string
    }

    #[test]
    fn test_send_parse_ts_ping() {
        let origin = Instant::now();
        let ping = send_ts_ping(origin);
        let bytes = Bytes::from(ping.to_vec());
        let duration = parse_ts_ping(&bytes);
        assert!(duration.is_some());
    }

    #[test]
    fn test_parse_ts_ping_invalid() {
        let bytes = Bytes::from(vec![0u8; 10]); // Invalid length
        let duration = parse_ts_ping(&bytes);
        assert!(duration.is_none());
    }
}
