// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

/// Error types for bartoc
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unable to load a valid configuration")]
    ConfigLoad,
    #[error("Unable to initialize tracing")]
    TracingInit,
    #[error("bartoc is shutting down")]
    Shutdown,
    #[error("Invalid bartoc message for bartos")]
    InvalidBartocMessage,
    #[error("Unable to get stdout handle")]
    StdoutHandle,
    #[error("Unable to get stderr handle")]
    StderrHandle,
    #[error("There is no valid data directory")]
    DataDir,
    #[error("No redb database path specified")]
    NoRedbPath,
    #[cfg(unix)]
    #[error("No shell found in environment variables")]
    NoShell,
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn config_load_display() {
        assert_eq!(
            Error::ConfigLoad.to_string(),
            "Unable to load a valid configuration"
        );
    }

    #[test]
    fn tracing_init_display() {
        assert_eq!(
            Error::TracingInit.to_string(),
            "Unable to initialize tracing"
        );
    }

    #[test]
    fn shutdown_display() {
        assert_eq!(Error::Shutdown.to_string(), "bartoc is shutting down");
    }

    #[test]
    fn invalid_bartoc_message_display() {
        assert_eq!(
            Error::InvalidBartocMessage.to_string(),
            "Invalid bartoc message for bartos"
        );
    }

    #[test]
    fn stdout_handle_display() {
        assert_eq!(
            Error::StdoutHandle.to_string(),
            "Unable to get stdout handle"
        );
    }

    #[test]
    fn stderr_handle_display() {
        assert_eq!(
            Error::StderrHandle.to_string(),
            "Unable to get stderr handle"
        );
    }

    #[test]
    fn data_dir_display() {
        assert_eq!(
            Error::DataDir.to_string(),
            "There is no valid data directory"
        );
    }

    #[test]
    fn no_redb_path_display() {
        assert_eq!(
            Error::NoRedbPath.to_string(),
            "No redb database path specified"
        );
    }

    #[cfg(unix)]
    #[test]
    fn no_shell_display() {
        assert_eq!(
            Error::NoShell.to_string(),
            "No shell found in environment variables"
        );
    }
}
