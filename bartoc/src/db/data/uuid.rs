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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UuidWrapper(pub Uuid);

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
