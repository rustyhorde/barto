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
use time::format_description::well_known;

use crate::db::data::{odt::OffsetDataTimeWrapper, uuid::UuidWrapper};

#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OutputKey {
    timestamp: OffsetDataTimeWrapper,
    uuid: UuidWrapper,
}

impl Display for OutputKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ts = self
            .timestamp
            .0
            .format(&well_known::Rfc3339)
            .unwrap_or("invalid timestamp".to_string());
        write!(f, "{} {}", ts, self.uuid.0)
    }
}

impl From<&Output> for OutputKey {
    fn from(output: &Output) -> Self {
        OutputKey {
            timestamp: output.timestamp(),
            uuid: output.uuid(),
        }
    }
}

#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OutputValue {
    kind: OutputKind,
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

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum OutputKind {
    Stdout,
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
pub(crate) struct Output {
    #[get_copy = "pub(crate)"]
    timestamp: OffsetDataTimeWrapper,
    #[get_copy = "pub(crate)"]
    uuid: UuidWrapper,
    #[get_copy = "pub(crate)"]
    kind: OutputKind,
    #[get = "pub(crate)"]
    data: String,
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {}) => {}", self.uuid, self.kind, self.data,)
    }
}
