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
    #[get = "pub(crate)"]
    name: String,
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
            name: output.cmd_name().clone(),
            kind: output.kind(),
            data: output.data().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use libbarto::{OffsetDataTimeWrapper, Output, OutputKind, UuidWrapper};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{OutputKey, OutputValue};

    fn make_output(kind: OutputKind) -> Output {
        Output::builder()
            .bartoc_uuid(UuidWrapper(Uuid::new_v4()))
            .bartoc_name("test-bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .cmd_name("test-cmd".to_string())
            .kind(kind)
            .data("hello".to_string())
            .build()
    }

    #[test]
    fn output_key_from_preserves_timestamp() {
        let output = make_output(OutputKind::Stdout);
        let key = OutputKey::from(&output);
        assert_eq!(key.timestamp(), output.timestamp());
    }

    #[test]
    fn output_key_from_preserves_bartoc_id() {
        let output = make_output(OutputKind::Stdout);
        let key = OutputKey::from(&output);
        assert_eq!(key.bartoc_id(), output.bartoc_uuid());
    }

    #[test]
    fn output_key_from_preserves_cmd_uuid() {
        let output = make_output(OutputKind::Stdout);
        let key = OutputKey::from(&output);
        assert_eq!(key.cmd_uuid(), output.cmd_uuid());
    }

    #[test]
    fn output_key_display_contains_cmd_uuid() {
        let output = make_output(OutputKind::Stdout);
        let key = OutputKey::from(&output);
        assert!(key.to_string().contains(&output.cmd_uuid().0.to_string()));
    }

    #[test]
    fn output_key_display_contains_timestamp() {
        let output = make_output(OutputKind::Stdout);
        let key = OutputKey::from(&output);
        assert!(key.to_string().contains('T'));
    }

    #[test]
    fn output_value_from_stdout() {
        let output = make_output(OutputKind::Stdout);
        let value = OutputValue::from(&output);
        assert_eq!(value.name(), output.cmd_name());
        assert_eq!(value.kind(), OutputKind::Stdout);
        assert_eq!(value.data(), output.data());
    }

    #[test]
    fn output_value_from_stderr() {
        let output = make_output(OutputKind::Stderr);
        let value = OutputValue::from(&output);
        assert_eq!(value.kind(), OutputKind::Stderr);
    }

    #[test]
    fn output_value_display_stdout() {
        let output = make_output(OutputKind::Stdout);
        let value = OutputValue::from(&output);
        let s = value.to_string();
        assert!(s.contains("stdout"));
        assert!(s.contains("hello"));
    }

    #[test]
    fn output_value_display_stderr() {
        let output = make_output(OutputKind::Stderr);
        let value = OutputValue::from(&output);
        assert!(value.to_string().contains("stderr"));
    }
}
