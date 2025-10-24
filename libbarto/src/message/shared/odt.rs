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
use time::{OffsetDateTime, format_description::well_known};

/// An `OffsetDateTime` wrapper that implements `bincode::Encode` and `bincode::Decode`
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OffsetDataTimeWrapper(pub OffsetDateTime);

impl<Context> Decode<Context> for OffsetDataTimeWrapper {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let odt = OffsetDateTime::parse(&s, &well_known::Rfc3339).map_err(|e| {
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
        let odt = OffsetDateTime::parse(&s, &well_known::Rfc3339).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse OffsetDateTime from string: {e}"))
        })?;
        Ok(OffsetDataTimeWrapper(odt))
    }
}

impl Encode for OffsetDataTimeWrapper {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let s = self.0.format(&well_known::Rfc3339).map_err(|e| {
            EncodeError::OtherString(format!("failed to format OffsetDateTime to string: {e}"))
        })?;
        s.encode(encoder)
    }
}

impl Display for OffsetDataTimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .format(&well_known::Rfc3339)
                .map_err(|_| std::fmt::Error)?
        )
    }
}
