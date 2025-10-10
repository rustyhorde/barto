// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{Decode, Encode};

/// A message from a worker client to a worker session
#[derive(Clone, Copy, Debug, Decode, Encode)]
pub enum BartoCli {
    /// An info request from barto-cli
    Info,
}
