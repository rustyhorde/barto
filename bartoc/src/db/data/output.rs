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
use libbarto::{OffsetDataTimeWrapper, Output, OutputKind, UuidWrapper};
use time::format_description::well_known;

#[derive(
    Builder, Clone, Copy, CopyGetters, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[get_copy = "pub(crate)"]
pub(crate) struct OutputKey {
    timestamp: OffsetDataTimeWrapper,
    bartoc_id: UuidWrapper,
    cmd_uuid: UuidWrapper,
}

impl Display for OutputKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ts = self
            .timestamp
            .0
            .format(&well_known::Rfc3339)
            .unwrap_or("invalid timestamp".to_string());
        write!(f, "{} {}", ts, self.cmd_uuid.0)
    }
}

impl From<&Output> for OutputKey {
    fn from(output: &Output) -> Self {
        OutputKey {
            timestamp: output.timestamp(),
            bartoc_id: output.bartoc_uuid(),
            cmd_uuid: output.cmd_uuid(),
        }
    }
}

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
pub(crate) struct OutputValue {
    #[get_copy = "pub(crate)"]
    kind: OutputKind,
    #[get = "pub(crate)"]
    data: String,
}

impl Display for OutputValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.kind, self.data)
    }
}

impl From<&Output> for OutputValue {
    fn from(output: &Output) -> Self {
        OutputValue {
            kind: output.kind(),
            data: output.data().clone(),
        }
    }
}
