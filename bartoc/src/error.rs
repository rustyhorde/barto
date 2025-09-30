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
}
