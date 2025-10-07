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
use getset::CopyGetters;
use libbarto::{Status, UuidWrapper};

#[derive(
    Builder, Clone, Copy, CopyGetters, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[get_copy = "pub(crate)"]
pub(crate) struct StatusKey {
    cmd_uuid: UuidWrapper,
}

impl Display for StatusKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cmd_uuid.0)
    }
}

impl From<&Status> for StatusKey {
    fn from(status: &Status) -> Self {
        StatusKey {
            cmd_uuid: status.cmd_uuid(),
        }
    }
}

#[derive(
    Builder, Clone, CopyGetters, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[get_copy = "pub(crate)"]
pub(crate) struct StatusValue {
    exit_code: Option<i32>,
    success: bool,
}

impl Display for StatusValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.exit_code {
            Some(code) => write!(f, "exit code: {code}, success: {}", self.success),
            None => write!(f, "exit code: None, success: {}", self.success),
        }
    }
}

impl From<&Status> for StatusValue {
    fn from(status: &Status) -> Self {
        StatusValue {
            exit_code: status.exit_code(),
            success: status.success(),
        }
    }
}
