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
use uuid::Uuid;

#[cfg(test)]
use crate::utils::Mock;

/// A `Uuid` wrapper that implements `bincode::Encode` and `bincode::Decode`
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UuidWrapper(pub Uuid);

#[cfg(test)]
impl Mock for UuidWrapper {
    fn mock() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for UuidWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<Context> Decode<Context> for UuidWrapper {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let uuid = Uuid::parse_str(&s).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse Uuid from string: {e}"))
        })?;
        Ok(UuidWrapper(uuid))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for UuidWrapper {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let uuid = Uuid::parse_str(&s).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse Uuid from string: {e}"))
        })?;
        Ok(UuidWrapper(uuid))
    }
}

impl Encode for UuidWrapper {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let s = format!("{}", self.0);
        s.encode(encoder)
    }
}

#[cfg(test)]
mod test {
    use super::UuidWrapper;

    use anyhow::Result;
    use bincode::{
        borrow_decode_from_slice, config::standard, decode_from_slice, encode_into_slice,
        encode_to_vec,
    };
    use uuid::Uuid;

    #[test]
    fn bad_encode_fails_decode() -> Result<()> {
        let not_valid = b"not a valid uuid".to_vec();
        let bad_encoded = encode_to_vec(not_valid, standard())?;
        let result: Result<(UuidWrapper, _), _> = decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn bad_encode_fails_borrow_decode() -> Result<()> {
        let mut slice = [0u8; 100];
        let not_valid = "not a valid uuid";
        let length = encode_into_slice(not_valid, &mut slice, standard())?;
        let bad_slice = &slice[..length];
        let result: Result<(UuidWrapper, _), _> = borrow_decode_from_slice(bad_slice, standard());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_uuid_wrapper_encode_decode() -> Result<()> {
        let original_uuid = Uuid::new_v4();
        let wrapper = UuidWrapper(original_uuid);

        // Encode the wrapper
        let encoded = encode_to_vec(wrapper, standard())?;

        // Decode back to wrapper
        let (decoded_wrapper, _): (UuidWrapper, _) = decode_from_slice(&encoded, standard())?;

        // Verify that the original and decoded Uuid are the same
        assert_eq!(wrapper.0, decoded_wrapper.0);
        Ok(())
    }

    #[test]
    fn test_uuid_wrapper_encode_borrow_decode() -> Result<()> {
        let original_uuid = Uuid::new_v4();
        let wrapper = UuidWrapper(original_uuid);

        // Encode the wrapper
        let encoded = encode_to_vec(wrapper, standard())?;

        // Decode back to wrapper using borrow decode
        let (decoded_wrapper, _): (UuidWrapper, _) =
            borrow_decode_from_slice(&encoded, standard())?;

        // Verify that the original and decoded Uuid are the same
        assert_eq!(wrapper.0, decoded_wrapper.0);
        Ok(())
    }

    #[test]
    fn test_uuid_wrapper_display() {
        let original_uuid = Uuid::new_v4();
        let wrapper = UuidWrapper(original_uuid);
        let display_str = format!("{wrapper}");
        assert_eq!(display_str, format!("{original_uuid}"));
    }
}
