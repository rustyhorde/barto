// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter};

use bincode_next::{Decode, Encode};
use bon::Builder;
use getset::CopyGetters;
use libbarto::{OffsetDataTimeWrapper, Status, UuidWrapper};

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
    timestamp: OffsetDataTimeWrapper,
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
            timestamp: status.timestamp(),
            exit_code: status.exit_code(),
            success: status.success(),
        }
    }
}

#[cfg(test)]
mod tests {
    use libbarto::{OffsetDataTimeWrapper, Status, UuidWrapper};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{StatusKey, StatusValue};

    fn make_status(exit_code: Option<i32>, success: bool) -> Status {
        Status::builder()
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(exit_code)
            .success(success)
            .build()
    }

    #[test]
    fn status_key_from_preserves_cmd_uuid() {
        let status = make_status(Some(0), true);
        let key = StatusKey::from(&status);
        assert_eq!(key.cmd_uuid(), status.cmd_uuid());
    }

    #[test]
    fn status_key_display_is_uuid_string() {
        let status = make_status(Some(0), true);
        let key = StatusKey::from(&status);
        assert_eq!(key.to_string(), status.cmd_uuid().0.to_string());
    }

    #[test]
    fn status_value_from_with_exit_code() {
        let status = make_status(Some(42), false);
        let value = StatusValue::from(&status);
        assert_eq!(value.exit_code(), Some(42));
        assert!(!value.success());
    }

    #[test]
    fn status_value_from_success() {
        let status = make_status(Some(0), true);
        let value = StatusValue::from(&status);
        assert_eq!(value.exit_code(), Some(0));
        assert!(value.success());
    }

    #[test]
    fn status_value_from_no_exit_code() {
        let status = make_status(None, false);
        let value = StatusValue::from(&status);
        assert_eq!(value.exit_code(), None);
        assert!(!value.success());
    }

    #[test]
    fn status_value_display_with_code() {
        let status = make_status(Some(1), false);
        let value = StatusValue::from(&status);
        let s = value.to_string();
        assert!(s.contains("exit code: 1"));
        assert!(s.contains("success: false"));
    }

    #[test]
    fn status_value_display_no_code() {
        let status = make_status(None, true);
        let value = StatusValue::from(&status);
        let s = value.to_string();
        assert!(s.contains("exit code: None"));
        assert!(s.contains("success: true"));
    }

    #[test]
    fn status_value_preserves_timestamp() {
        let status = make_status(Some(0), true);
        let value = StatusValue::from(&status);
        assert_eq!(value.timestamp(), status.timestamp());
    }
}
