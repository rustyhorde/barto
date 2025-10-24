// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter};

use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::message::shared::{odt::OffsetDataTimeWrapper, uuid::UuidWrapper};
#[cfg(test)]
use crate::utils::Mock;

/// A record of data from a bartoc client
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Data {
    /// An output record
    Output(Output),
    /// A status record
    Status(Status),
}

impl<Context> Decode<Context> for Data {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u8 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let output = Output::decode(decoder)?;
                Ok(Data::Output(output))
            }
            1 => {
                let status = Status::decode(decoder)?;
                Ok(Data::Status(status))
            }
            _ => Err(DecodeError::Other("Invalid variant for Data enum")),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Data {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u8 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let output = Output::borrow_decode(decoder)?;
                Ok(Data::Output(output))
            }
            1 => {
                let status = Status::borrow_decode(decoder)?;
                Ok(Data::Status(status))
            }
            _ => Err(DecodeError::Other("Invalid variant for Data enum")),
        }
    }
}

impl Encode for Data {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Data::Output(output) => {
                Encode::encode(&0u8, encoder)?;
                Encode::encode(output, encoder)?;
            }
            Data::Status(status) => {
                Encode::encode(&1u8, encoder)?;
                Encode::encode(status, encoder)?;
            }
        }
        Ok(())
    }
}

/// The kind of output (stdout or stderr)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutputKind {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}

impl<Context> Decode<Context> for OutputKind {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u8 = Decode::decode(decoder)?;
        match variant {
            0 => Ok(OutputKind::Stdout),
            1 => Ok(OutputKind::Stderr),
            _ => Err(DecodeError::Other("Invalid variant for Data enum")),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for OutputKind {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u8 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => Ok(OutputKind::Stdout),
            1 => Ok(OutputKind::Stderr),
            _ => Err(DecodeError::Other("Invalid variant for Data enum")),
        }
    }
}

impl Encode for OutputKind {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            OutputKind::Stdout => {
                Encode::encode(&0u8, encoder)?;
            }
            OutputKind::Stderr => {
                Encode::encode(&1u8, encoder)?;
            }
        }
        Ok(())
    }
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
#[derive(Builder, Clone, CopyGetters, Debug, Eq, Getters, Hash, Ord, PartialEq, PartialOrd)]
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

#[cfg(test)]
impl Mock for Output {
    fn mock() -> Self {
        Self::builder()
            .bartoc_uuid(UuidWrapper::mock())
            .bartoc_name("mock_bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper::mock())
            .cmd_uuid(UuidWrapper::mock())
            .cmd_name("mock_command".to_string())
            .kind(OutputKind::Stdout)
            .data("mock output data".to_string())
            .build()
    }
}

impl<Context> Decode<Context> for Output {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bartoc_uuid = UuidWrapper::decode(decoder)?;
        let bartoc_name = String::decode(decoder)?;
        let timestamp = OffsetDataTimeWrapper::decode(decoder)?;
        let cmd_uuid = UuidWrapper::decode(decoder)?;
        let cmd_name = String::decode(decoder)?;
        let kind = OutputKind::decode(decoder)?;
        let data = String::decode(decoder)?;

        Ok(Output {
            bartoc_uuid,
            bartoc_name,
            timestamp,
            cmd_uuid,
            cmd_name,
            kind,
            data,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Output {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let bartoc_uuid = UuidWrapper::borrow_decode(decoder)?;
        let bartoc_name = String::borrow_decode(decoder)?;
        let timestamp = OffsetDataTimeWrapper::borrow_decode(decoder)?;
        let cmd_uuid = UuidWrapper::borrow_decode(decoder)?;
        let cmd_name = String::borrow_decode(decoder)?;
        let kind = OutputKind::borrow_decode(decoder)?;
        let data = String::borrow_decode(decoder)?;

        Ok(Output {
            bartoc_uuid,
            bartoc_name,
            timestamp,
            cmd_uuid,
            cmd_name,
            kind,
            data,
        })
    }
}

impl Encode for Output {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.bartoc_uuid, encoder)?;
        Encode::encode(&self.bartoc_name, encoder)?;
        Encode::encode(&self.timestamp, encoder)?;
        Encode::encode(&self.cmd_uuid, encoder)?;
        Encode::encode(&self.cmd_name, encoder)?;
        Encode::encode(&self.kind, encoder)?;
        Encode::encode(&self.data, encoder)?;
        Ok(())
    }
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
    Builder, Clone, Copy, CopyGetters, Debug, Eq, Getters, Hash, Ord, PartialEq, PartialOrd,
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

impl<Context> Decode<Context> for Status {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let cmd_uuid = UuidWrapper::decode(decoder)?;
        let timestamp = OffsetDataTimeWrapper::decode(decoder)?;
        let exit_code = Option::<i32>::decode(decoder)?;
        let success = bool::decode(decoder)?;

        Ok(Status {
            cmd_uuid,
            timestamp,
            exit_code,
            success,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Status {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let cmd_uuid = UuidWrapper::borrow_decode(decoder)?;
        let timestamp = OffsetDataTimeWrapper::borrow_decode(decoder)?;
        let exit_code = Option::<i32>::borrow_decode(decoder)?;
        let success = bool::borrow_decode(decoder)?;

        Ok(Status {
            cmd_uuid,
            timestamp,
            exit_code,
            success,
        })
    }
}

impl Encode for Status {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.cmd_uuid, encoder)?;
        Encode::encode(&self.timestamp, encoder)?;
        Encode::encode(&self.exit_code, encoder)?;
        Encode::encode(&self.success, encoder)?;
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bincode::{
        borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec,
        error::DecodeError,
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{OffsetDataTimeWrapper, UuidWrapper};

    use super::{Data, Output, OutputKind, Status};

    #[test]
    fn output_kind_display() {
        assert_eq!(OutputKind::Stdout.to_string(), "stdout");
        assert_eq!(OutputKind::Stderr.to_string(), "stderr");
    }

    #[test]
    fn output_kind_into_str() {
        let stdout_str: &str = OutputKind::Stdout.into();
        let stderr_str: &str = OutputKind::Stderr.into();
        assert_eq!(stdout_str, "stdout");
        assert_eq!(stderr_str, "stderr");
    }

    #[test]
    fn data_encode_decode_output() {
        let output = Output::builder()
            .bartoc_uuid(UuidWrapper(Uuid::new_v4()))
            .bartoc_name("test_bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .cmd_name("test_command".to_string())
            .kind(OutputKind::Stdout)
            .data("test output".to_string())
            .build();

        let data = Data::Output(output.clone());
        let encoded = encode_to_vec(&data, standard()).unwrap();
        let decoded: Data = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(data, decoded);
    }

    #[test]
    fn data_encode_decode_status() {
        let status = Status::builder()
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(Some(0))
            .success(true)
            .build();

        let data = Data::Status(status);
        let encoded = encode_to_vec(&data, standard()).unwrap();
        let decoded: Data = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(data, decoded);
    }

    #[test]
    fn output_kind_encode_decode() {
        let stdout = OutputKind::Stdout;
        let stderr = OutputKind::Stderr;

        let encoded_stdout = encode_to_vec(stdout, standard()).unwrap();
        let encoded_stderr = encode_to_vec(stderr, standard()).unwrap();

        let decoded_stdout: OutputKind = decode_from_slice(&encoded_stdout, standard()).unwrap().0;
        let decoded_stderr: OutputKind = decode_from_slice(&encoded_stderr, standard()).unwrap().0;

        assert_eq!(stdout, decoded_stdout);
        assert_eq!(stderr, decoded_stderr);
    }

    #[test]
    fn output_display() {
        let bartoc_uuid = UuidWrapper(Uuid::new_v4());
        let cmd_uuid = UuidWrapper(Uuid::new_v4());

        let output = Output::builder()
            .bartoc_uuid(bartoc_uuid)
            .bartoc_name("test_bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(cmd_uuid)
            .cmd_name("test_command".to_string())
            .kind(OutputKind::Stdout)
            .data("hello world".to_string())
            .build();

        let display_str = output.to_string();
        assert!(display_str.contains(&bartoc_uuid.to_string()));
        assert!(display_str.contains(&cmd_uuid.to_string()));
        assert!(display_str.contains("stdout"));
        assert!(display_str.contains("hello world"));
    }

    #[test]
    fn status_display_with_exit_code() {
        let cmd_uuid = UuidWrapper(Uuid::new_v4());
        let status = Status::builder()
            .cmd_uuid(cmd_uuid)
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(Some(42))
            .success(false)
            .build();

        let display_str = status.to_string();
        assert!(display_str.contains(&cmd_uuid.to_string()));
        assert!(display_str.contains("exit_code=42"));
        assert!(display_str.contains("success=false"));
    }

    #[test]
    fn status_display_without_exit_code() {
        let cmd_uuid = UuidWrapper(Uuid::new_v4());
        let status = Status::builder()
            .cmd_uuid(cmd_uuid)
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(None)
            .success(true)
            .build();

        let display_str = status.to_string();
        assert!(display_str.contains(&cmd_uuid.to_string()));
        assert!(display_str.contains("exit_code=None"));
        assert!(display_str.contains("success=true"));
    }

    #[test]
    fn output_kind_borrow_decode() {
        let stdout = OutputKind::Stdout;
        let stderr = OutputKind::Stderr;

        let encoded_stdout = encode_to_vec(stdout, standard()).unwrap();
        let encoded_stderr = encode_to_vec(stderr, standard()).unwrap();

        let decoded_stdout: OutputKind = borrow_decode_from_slice(&encoded_stdout, standard())
            .unwrap()
            .0;
        let decoded_stderr: OutputKind = borrow_decode_from_slice(&encoded_stderr, standard())
            .unwrap()
            .0;

        assert_eq!(stdout, decoded_stdout);
        assert_eq!(stderr, decoded_stderr);
    }

    #[test]
    fn data_borrow_decode_output() {
        let output = Output::builder()
            .bartoc_uuid(UuidWrapper(Uuid::new_v4()))
            .bartoc_name("test_bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .cmd_name("test_command".to_string())
            .kind(OutputKind::Stdout)
            .data("test output".to_string())
            .build();

        let data = Data::Output(output.clone());
        let encoded = encode_to_vec(&data, standard()).unwrap();
        let decoded: Data = borrow_decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(data, decoded);
    }

    #[test]
    fn data_borrow_decode_status() {
        let status = Status::builder()
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(Some(0))
            .success(true)
            .build();

        let data = Data::Status(status);
        let encoded = encode_to_vec(&data, standard()).unwrap();
        let decoded: Data = borrow_decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(data, decoded);
    }

    #[test]
    fn output_borrow_decode() {
        let output = Output::builder()
            .bartoc_uuid(UuidWrapper(Uuid::new_v4()))
            .bartoc_name("test_bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .cmd_name("test_command".to_string())
            .kind(OutputKind::Stderr)
            .data("error message".to_string())
            .build();

        let encoded = encode_to_vec(&output, standard()).unwrap();
        let decoded: Output = borrow_decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(output, decoded);
    }

    #[test]
    fn status_borrow_decode() -> Result<()> {
        let status = Status::builder()
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(None)
            .success(false)
            .build();

        let encoded = encode_to_vec(status, standard())?;
        let decoded: Status = borrow_decode_from_slice(&encoded, standard())?.0;

        assert_eq!(status, decoded);
        Ok(())
    }

    #[test]
    fn output_kind_bad_decode_variant() -> Result<()> {
        // Manually create encoded data with invalid variant (2)
        let bad_encoded = encode_to_vec(2, standard())?;

        let result: Result<(OutputKind, usize), _> = decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::Other(_)));

        Ok(())
    }

    #[test]
    fn output_kind_bad_borrow_decode_variant() -> Result<()> {
        // Manually create encoded data with invalid variant (2)
        let bad_encoded = encode_to_vec(2u8, standard())?;

        let result: Result<(OutputKind, usize), _> =
            borrow_decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::Other(_)));

        Ok(())
    }

    #[test]
    fn data_bad_decode_variant() -> Result<()> {
        // Manually create encoded data with invalid variant (2)
        let bad_encoded = encode_to_vec(2u8, standard())?;

        let result: Result<(Data, usize), _> = decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::Other(_)));

        Ok(())
    }

    #[test]
    fn data_bad_borrow_decode_variant() -> Result<()> {
        // Manually create encoded data with invalid variant (2)
        let bad_encoded = encode_to_vec(2u8, standard())?;

        let result: Result<(Data, usize), _> = borrow_decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::Other(_)));

        Ok(())
    }
}
