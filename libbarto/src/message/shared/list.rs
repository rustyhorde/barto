// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{Decode, Encode};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::OffsetDataTimeWrapper;

/// The output of a `List` request, containing the names of all registered bartoc clients
#[derive(Builder, Clone, CopyGetters, Debug, Decode, Encode, Getters, PartialEq)]
pub struct ListOutput {
    /// The timestamp of when the output was generated
    #[getset(get = "pub")]
    timestamp: Option<OffsetDataTimeWrapper>,
    /// The data returned from the command
    #[getset(get = "pub")]
    data: Option<String>,
    /// The exit code of the command
    #[getset(get_copy = "pub")]
    exit_code: u8,
    /// Whether the command was successful
    #[getset(get_copy = "pub")]
    success: i8,
}
