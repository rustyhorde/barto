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
use time::{OffsetDateTime, error::Format, format_description::well_known::Rfc3339};

#[cfg(test)]
use crate::utils::Mock;

/// An `OffsetDateTime` wrapper that implements `bincode::Encode` and `bincode::Decode`
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OffsetDataTimeWrapper(pub OffsetDateTime);

#[cfg(test)]
impl Mock for OffsetDataTimeWrapper {
    fn mock() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}

impl OffsetDataTimeWrapper {
    /// Get the inner `OffsetDateTime`
    #[cfg_attr(coverage_nightly, coverage(off))] // this can't be tested directly
    #[allow(clippy::needless_pass_by_value)] // Used in map
    fn format_err(err: Format) -> EncodeError {
        EncodeError::OtherString(format!("failed to format OffsetDateTime to string: {err}"))
    }
}

impl<Context> Decode<Context> for OffsetDataTimeWrapper {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let odt = OffsetDateTime::parse(&s, &Rfc3339).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse OffsetDateTime from string: {e}"))
        })?;
        Ok(OffsetDataTimeWrapper(odt))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for OffsetDataTimeWrapper {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let odt = OffsetDateTime::parse(&s, &Rfc3339).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse OffsetDateTime from string: {e}"))
        })?;
        Ok(OffsetDataTimeWrapper(odt))
    }
}

impl Encode for OffsetDataTimeWrapper {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let s = self.0.format(&Rfc3339).map_err(Self::format_err)?;
        s.encode(encoder)
    }
}

impl Display for OffsetDataTimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = self.0.format(&Rfc3339).map_err(|_| std::fmt::Error)?;
        write!(f, "{formatted}")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::OffsetDataTimeWrapper;

    use anyhow::Result;
    use bincode::{
        borrow_decode_from_slice, config::standard, decode_from_slice, encode_into_slice,
        encode_to_vec,
    };
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    #[test]
    fn bad_encode_fails_decode() -> Result<()> {
        let not_valid = b"not a valid datetime".to_vec();
        let bad_encoded = encode_to_vec(not_valid, standard())?;
        let result: Result<(OffsetDataTimeWrapper, _), _> =
            decode_from_slice(&bad_encoded, standard());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn bad_encode_fails_borrow_decode() -> Result<()> {
        let mut slice = [0u8; 100];
        let not_valid = "not a valid datetime";
        let length = encode_into_slice(not_valid, &mut slice, standard())?;
        let bad_slice = &slice[..length];
        let result: Result<(OffsetDataTimeWrapper, _), _> =
            borrow_decode_from_slice(bad_slice, standard());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_odt_wrapper_encode_decode() -> Result<()> {
        let original_odt = OffsetDateTime::now_utc();
        let wrapper = OffsetDataTimeWrapper(original_odt);

        // Encode the wrapper
        let encoded = encode_to_vec(wrapper, standard())?;

        // Decode back to wrapper
        let (decoded_wrapper, _): (OffsetDataTimeWrapper, _) =
            decode_from_slice(&encoded, standard())?;

        // Verify that the original and decoded OffsetDateTime are the same
        assert_eq!(wrapper.0, decoded_wrapper.0);
        Ok(())
    }

    #[test]
    fn test_odt_wrapper_encode_borrow_decode() -> Result<()> {
        let original_odt = OffsetDateTime::now_utc();
        let wrapper = OffsetDataTimeWrapper(original_odt);

        // Encode the wrapper
        let encoded = encode_to_vec(wrapper, standard())?;

        // Decode back to wrapper using borrow decode
        let (decoded_wrapper, _): (OffsetDataTimeWrapper, _) =
            borrow_decode_from_slice(&encoded, standard())?;

        // Verify that the original and decoded OffsetDateTime are the same
        assert_eq!(wrapper.0, decoded_wrapper.0);
        Ok(())
    }

    #[test]
    fn test_odt_wrapper_display() {
        let odt = OffsetDateTime::parse("2025-01-01T12:00:00+00:00", &Rfc3339).unwrap();
        let wrapper = OffsetDataTimeWrapper(odt);
        let display_str = format!("{wrapper}");
        assert_eq!(display_str, "2025-01-01T12:00:00Z");
    }
}
