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

/// A record of data from a bartoc client
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Data {
    /// An output record
    Output(Output),
    /// A status record
    Status(Status),
}

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

impl From<OutputKind> for &'static str {
    fn from(kind: OutputKind) -> Self {
        match kind {
            OutputKind::Stdout => "stdout",
            OutputKind::Stderr => "stderr",
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
    /// The id of the bartoc that produced the output
    #[get_copy = "pub"]
    bartoc_uuid: UuidWrapper,
    /// The name of the bartoc that produced the output
    #[get = "pub"]
    bartoc_name: String,
    /// The timestamp of the output
    #[get_copy = "pub"]
    timestamp: OffsetDataTimeWrapper,
    /// The UUID of the bartoc command that produced the output
    #[get_copy = "pub"]
    cmd_uuid: UuidWrapper,
    /// The name of the command that produced the output
    #[get = "pub"]
    cmd_name: String,
    /// The kind of output (stdout or stderr)
    #[get_copy = "pub"]
    kind: OutputKind,
    /// The output data
    #[get = "pub"]
    data: String,
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {}) => {}",
            self.bartoc_uuid, self.cmd_uuid, self.kind, self.data,
        )
    }
}

/// An output record from a bartoc client
#[derive(
    Builder,
    Clone,
    Copy,
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
pub struct Status {
    /// The command `Uuid` of the bartoc command that produced the status
    #[get_copy = "pub"]
    cmd_uuid: UuidWrapper,
    /// The timestamp of the status
    #[get_copy = "pub"]
    timestamp: OffsetDataTimeWrapper,
    /// The exit code of the command
    #[get_copy = "pub"]
    #[builder(required)]
    exit_code: Option<i32>,
    /// The success status of the command
    #[get_copy = "pub"]
    success: bool,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.exit_code {
            write!(
                f,
                "({} exit_code={} success={})",
                self.cmd_uuid, code, self.success,
            )
        } else {
            write!(
                f,
                "({} exit_code=None success={})",
                self.cmd_uuid, self.success,
            )
        }
    }
}
