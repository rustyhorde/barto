// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::OffsetDataTimeWrapper;
#[cfg(test)]
use crate::utils::Mock;

/// The output of a `Failed` request
#[derive(Builder, Clone, CopyGetters, Debug, Getters, PartialEq)]
pub struct FailedOutput {
    /// The timestamp of when the output was generated
    #[getset(get = "pub")]
    timestamp: Option<OffsetDataTimeWrapper>,
    /// The name of the bartoc client
    #[getset(get = "pub")]
    bartoc_name: Option<String>,
    /// The name of the command that failed
    #[getset(get = "pub")]
    cmd_name: Option<String>,
    /// The data returned from the command
    #[getset(get = "pub")]
    data: Option<String>,
    /// The exit code of the command
    #[getset(get_copy = "pub")]
    exit_code: u8,
    /// Whether the command was successful
    #[getset(get_copy = "pub")]
    success: i8,
}

#[cfg(test)]
impl Mock for FailedOutput {
    fn mock() -> Self {
        Self {
            timestamp: Some(OffsetDataTimeWrapper::mock()),
            bartoc_name: Some("mock_bartoc".to_string()),
            cmd_name: Some("mock_cmd".to_string()),
            data: Some("mock_data".to_string()),
            exit_code: 1,
            success: 0,
        }
    }
}

impl<Context> Decode<Context> for FailedOutput {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            timestamp: Decode::decode(decoder)?,
            bartoc_name: Decode::decode(decoder)?,
            cmd_name: Decode::decode(decoder)?,
            data: Decode::decode(decoder)?,
            exit_code: Decode::decode(decoder)?,
            success: Decode::decode(decoder)?,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for FailedOutput {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            timestamp: BorrowDecode::borrow_decode(decoder)?,
            bartoc_name: BorrowDecode::borrow_decode(decoder)?,
            cmd_name: BorrowDecode::borrow_decode(decoder)?,
            data: BorrowDecode::borrow_decode(decoder)?,
            exit_code: BorrowDecode::borrow_decode(decoder)?,
            success: BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl Encode for FailedOutput {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.timestamp, encoder)?;
        Encode::encode(&self.bartoc_name, encoder)?;
        Encode::encode(&self.cmd_name, encoder)?;
        Encode::encode(&self.data, encoder)?;
        Encode::encode(&self.exit_code, encoder)?;
        Encode::encode(&self.success, encoder)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::FailedOutput;

    use crate::utils::Mock;
    use anyhow::Result;
    use bincode::{config::standard, decode_from_slice, encode_to_vec};

    #[test]
    fn test_failed_output_encode_decode() -> Result<()> {
        let original = FailedOutput::mock();

        // Encode
        let encoded = encode_to_vec(&original, standard())?;
        let (decoded, _): (FailedOutput, usize) = decode_from_slice(&encoded, standard())?;
        let (borrow_decoded, _): (FailedOutput, usize) =
            bincode::borrow_decode_from_slice(&encoded, standard())?;

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
        Ok(())
    }
}
