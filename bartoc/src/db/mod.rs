// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    any::type_name,
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
};

use bincode::{
    BorrowDecode, Decode, Encode,
    config::standard,
    de::{BorrowDecoder, Decoder},
    decode_from_slice,
    enc::Encoder,
    encode_to_vec,
    error::{DecodeError, EncodeError},
};
use bon::Builder;
use redb::{Key, TypeName, Value};
use time::{OffsetDateTime, format_description::well_known};

use crate::handler::OutputKind;

#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OutKey {
    timestamp: OdtWrapper,
}

impl Display for OutKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.timestamp.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OdtWrapper(pub OffsetDateTime);

impl<Context> Decode<Context> for OdtWrapper {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let odt = OffsetDateTime::parse(&s, &well_known::Rfc3339).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse OffsetDateTime from string: {e}"))
        })?;
        Ok(OdtWrapper(odt))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for OdtWrapper {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        let odt = OffsetDateTime::parse(&s, &well_known::Rfc3339).map_err(|e| {
            DecodeError::OtherString(format!("failed to parse OffsetDateTime from string: {e}"))
        })?;
        Ok(OdtWrapper(odt))
    }
}

impl Encode for OdtWrapper {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let s = self.0.format(&well_known::Rfc3339).map_err(|e| {
            EncodeError::OtherString(format!("failed to format OffsetDateTime to string: {e}"))
        })?;
        s.encode(encoder)
    }
}

#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OutValue {
    data: (OutputKind, Vec<u8>),
}

impl Display for OutValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.data.0,
            String::from_utf8_lossy(&self.data.1)
        )
    }
}

/// Wrapper type to handle keys and values using bincode serialization
#[derive(Debug)]
pub(crate) struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
where
    T: Debug + Encode + Decode<()>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice(data, standard()).unwrap().0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        encode_to_vec(value, standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}

impl<T> Key for Bincode<T>
where
    T: Debug + Decode<()> + Encode + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
