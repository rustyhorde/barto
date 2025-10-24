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

/// The output of a `List` request, containing the names of all registered bartoc clients
#[derive(Builder, Clone, CopyGetters, Debug, Getters, PartialEq)]
pub struct ListOutput {
    /// The timestamp of when the output was generated
    #[getset(get = "pub")]
    timestamp: Option<OffsetDataTimeWrapper>,
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

impl<Context> Decode<Context> for ListOutput {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            timestamp: Decode::decode(decoder)?,
            data: Decode::decode(decoder)?,
            exit_code: Decode::decode(decoder)?,
            success: Decode::decode(decoder)?,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for ListOutput {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            timestamp: BorrowDecode::borrow_decode(decoder)?,
            data: BorrowDecode::borrow_decode(decoder)?,
            exit_code: BorrowDecode::borrow_decode(decoder)?,
            success: BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl Encode for ListOutput {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.timestamp, encoder)?;
        Encode::encode(&self.data, encoder)?;
        Encode::encode(&self.exit_code, encoder)?;
        Encode::encode(&self.success, encoder)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::OffsetDataTimeWrapper;

    use super::ListOutput;

    use anyhow::Result;
    use bincode::{borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec};
    use time::OffsetDateTime;

    #[test]
    fn test_initialize_encode_decode() -> Result<()> {
        let odtw = OffsetDataTimeWrapper(OffsetDateTime::now_utc());
        let list_output = ListOutput::builder()
            .timestamp(odtw)
            .data("client1\nclient2\n".to_string())
            .exit_code(0)
            .success(1)
            .build();

        let encoded = encode_to_vec(list_output.clone(), standard())?;
        let (decoded, _): (ListOutput, _) = decode_from_slice(&encoded, standard())?;
        let (borrow_decoded, _): (ListOutput, _) = borrow_decode_from_slice(&encoded, standard())?;

        assert_eq!(list_output, decoded);
        assert_eq!(borrow_decoded, decoded);
        assert_eq!(list_output.timestamp(), decoded.timestamp());
        assert_eq!(list_output.data(), decoded.data());
        assert_eq!(list_output.exit_code(), decoded.exit_code());
        assert_eq!(list_output.success(), decoded.success());
        assert!(!format!("{list_output:?}").is_empty());
        Ok(())
    }
}
