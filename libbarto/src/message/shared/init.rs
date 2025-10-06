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

use crate::{Schedules, UuidWrapper};

/// An initialization message from bartos to a named bartoc client.
#[derive(Builder, Clone, CopyGetters, Debug, Decode, Encode, Getters)]
pub struct Initialize {
    /// The unique identifier for the bartoc client
    #[get_copy = "pub"]
    id: UuidWrapper,
    /// The schedules to initialize the bartoc client with
    #[get = "pub"]
    schedules: Schedules,
}
