// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter};

use anyhow::{Error, Result};
use rand::Rng as _;

use crate::{error::Error::InvalidTime, schedule::All, utils::as_two_digit};

pub(crate) const SECONDS_PER_MINUTE: u8 = 60;

/// The seconds for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Second {
    /// Every second
    All,
    /// Specific seconds
    Seconds(Vec<u8>),
}

impl Second {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Second::All => true,
            Second::Seconds(seconds) => seconds.contains(&given),
        }
    }

    pub(crate) fn top_of_minute() -> Self {
        Second::Seconds(vec![0])
    }
}

impl Default for Second {
    fn default() -> Self {
        Self::Seconds(vec![0])
    }
}

impl All for Second {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(0..60);
        Second::Seconds(vec![rand_in_range])
    }
}

impl TryFrom<Vec<u8>> for Second {
    type Error = Error;

    fn try_from(values: Vec<u8>) -> Result<Self> {
        for &value in &values {
            if value >= SECONDS_PER_MINUTE {
                return Err(InvalidTime(value.to_string()).into());
            }
        }
        Ok(Second::Seconds(values))
    }
}

impl TryFrom<u8> for Second {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Second::try_from(vec![value])
    }
}

impl Display for Second {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Second::All => write!(f, "*"),
            Second::Seconds(seconds) => {
                write!(f, "{}", as_two_digit(seconds))
            }
        }
    }
}
