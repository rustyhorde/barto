// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

/// Error types for bartos
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unable to load a valid configuration")]
    ConfigLoad,
    #[error("Unable to initialize tracing")]
    TracingInit,
    #[error("Invalid message received")]
    InvalidMessage,
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
    fn invalid_message_display() {
        assert_eq!(
            Error::InvalidMessage.to_string(),
            "Invalid message received"
        );
    }
}
