// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter};

use bincode::{Decode, Encode};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::message::shared::{odt::OffsetDataTimeWrapper, uuid::UuidWrapper};

/// The kind of output (stdout or stderr)
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutputKind {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}

impl Display for OutputKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputKind::Stdout => write!(f, "stdout"),
            OutputKind::Stderr => write!(f, "stderr"),
        }
    }
}

/// An output record from a bartoc client
#[derive(
    Builder,
    Clone,
    CopyGetters,
    Debug,
    Decode,
    Encode,
    Eq,
    Getters,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct Output {
    /// The timestamp of the output
    #[get_copy = "pub"]
    timestamp: OffsetDataTimeWrapper,
    /// The UUID of the bartoc command that produced the output
    #[get_copy = "pub"]
    uuid: UuidWrapper,
    /// The kind of output (stdout or stderr)
    #[get_copy = "pub"]
    kind: OutputKind,
    /// The output data
    #[get = "pub"]
    data: String,
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {}) => {}", self.uuid, self.kind, self.data,)
    }
}
