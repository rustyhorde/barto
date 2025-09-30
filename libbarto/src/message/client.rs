// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use actix::Message;
use bincode::{Decode, Encode};

/// A message from a worker client to a worker session
#[derive(Clone, Debug, Decode, Encode, Message)]
#[rtype(result = "()")]
pub enum WorkerClientToWorkerSession {
    /// A text message for a server
    Text(String),
}
