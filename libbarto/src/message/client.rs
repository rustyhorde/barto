// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{Decode, Encode};

use crate::Output;

/// A message from a worker client to a worker session
#[derive(Clone, Debug, Decode, Encode)]
pub enum BartocWs {
    /// A close message from bartoc
    Close(Option<(u16, String)>),
    /// A ping message from bartoc
    Ping(Vec<u8>),
    /// A pong message from bartos
    Pong(Vec<u8>),
}

/// A message from a worker client to a worker session
#[derive(Clone, Debug, Decode, Encode)]
pub enum Bartoc {
    /// A close message from bartoc
    Record(Output),
}
