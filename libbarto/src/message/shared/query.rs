// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use bincode::{Decode, Encode};

use crate::Error;

/// Supported types for barto-cli queries.
#[derive(Debug, Clone, Copy, Decode, Encode, Eq, PartialEq)]
pub enum QueryTypes {
    /// 64-bit signed integer.
    U64,
    /// `OffsetDateTime`
    ODT,
    /// String
    Str,
    /// UUID
    UUID,
}

impl TryFrom<String> for QueryTypes {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "u64" => Ok(QueryTypes::U64),
            "odt" => Ok(QueryTypes::ODT),
            "str" => Ok(QueryTypes::Str),
            "uuid" => Ok(QueryTypes::UUID),
            _ => Err(Error::InvalidQueryType.into()),
        }
    }
}
