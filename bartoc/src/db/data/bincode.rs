// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{any::type_name, cmp::Ordering, fmt::Debug};

use bincode::{Decode, Encode, config::standard, decode_from_slice, encode_to_vec};
use redb::{Key, TypeName, Value};

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
