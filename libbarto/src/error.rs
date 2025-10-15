// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use clap::error::ErrorKind;

/// Error types for the barto library
#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// No valid config directory could be found
    #[error("There is no valid config directory")]
    ConfigDir,
    /// Unable to build a valid configuration
    #[error("Unable to build a valid configuration")]
    ConfigBuild,
    /// Unable to deserialize configuration
    #[error("Unable to deserialize config")]
    ConfigDeserialize,
    /// No valid data directory could be found
    #[error("There is no valid data directory")]
    DataDir,
    /// Unable to read the certificate file
    #[error("Unable to read the certificate file")]
    CertRead,
    /// Unable to read the private key file
    #[error("Unable to read the private key file")]
    KeyRead,
    /// No valid private keys found in the key file
    #[error("No valid private keys found in the key file")]
    NoPrivateKeys,
    /// No valid captures when parsing a realtime schedule
    #[error("no valid captures")]
    NoValidCaptures,
    /// An invalid range was specified when parsing a realtime schedule
    #[error("invalid range: '{}'", range)]
    InvalidRange {
        /// The invalid range
        range: String,
    },
    /// An invalid first capture when parsing a realtime schedule
    #[error("invalid first capture")]
    InvalidFirstCapture,
    /// An invalid second capture when parsing a realtime schedule
    #[error("invalid second capture")]
    InvalidSecondCapture,
    /// An invalid time string was specified when parsing a realtime schedule
    #[error("invalid time string: '{}'", time)]
    InvalidTime {
        /// The invalid time string
        time: String,
    },
    /// An invalid date string was specified when parsing a realtime schedule
    #[error("invalid date string: '{}'", date)]
    InvalidDate {
        /// The invalid date string
        date: String,
    },
    /// An invalid calendar string was specified when parsing a realtime schedule
    #[error("invalid calendar string: '{}'", calendar)]
    InvalidCalendar {
        /// The invalid calendar string
        calendar: String,
    },
    /// An invalid query type was specified
    #[error("invalid query type")]
    InvalidQueryType,
}

/// Converts an `anyhow::Error` into a suitable exit code or clap message for a CLI application.
#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn clap_or_error(err: anyhow::Error) -> i32 {
    let disp_err = || {
        eprintln!("{err:?}");
        1
    };
    match err.downcast_ref::<clap::Error>() {
        Some(e) => match e.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                println!("{e}");
                0
            }
            ErrorKind::InvalidValue
            | ErrorKind::UnknownArgument
            | ErrorKind::InvalidSubcommand
            | ErrorKind::NoEquals
            | ErrorKind::ValueValidation
            | ErrorKind::TooManyValues
            | ErrorKind::TooFewValues
            | ErrorKind::WrongNumberOfValues
            | ErrorKind::ArgumentConflict
            | ErrorKind::MissingRequiredArgument
            | ErrorKind::MissingSubcommand
            | ErrorKind::InvalidUtf8
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | ErrorKind::Io
            | ErrorKind::Format => disp_err(),
            _ => {
                eprintln!("Unknown ErrorKind");
                disp_err()
            }
        },
        None => disp_err(),
    }
}

/// Indicates successful execution of a function, returning exit code 0.
#[must_use]
pub fn success((): ()) -> i32 {
    0
}
