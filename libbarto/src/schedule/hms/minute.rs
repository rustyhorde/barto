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

pub(crate) const MINUTES_PER_HOUR: u8 = 60;

/// The minute for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Minute {
    /// Every minute
    All,
    /// Specific minutes
    Minutes(Vec<u8>),
}

impl Minute {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Minute::All => true,
            Minute::Minutes(minutes) => minutes.contains(&given),
        }
    }

    pub(crate) fn top_of_hour() -> Self {
        Minute::Minutes(vec![0])
    }
}

impl Default for Minute {
    fn default() -> Self {
        Self::Minutes(vec![0])
    }
}

impl All for Minute {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(0..MINUTES_PER_HOUR);
        Minute::Minutes(vec![rand_in_range])
    }
}

impl TryFrom<Vec<u8>> for Minute {
    type Error = Error;

    fn try_from(values: Vec<u8>) -> Result<Self> {
        for &value in &values {
            if value >= MINUTES_PER_HOUR {
                return Err(InvalidTime(value.to_string()).into());
            }
        }
        Ok(Minute::Minutes(values))
    }
}

impl TryFrom<u8> for Minute {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Minute::try_from(vec![value])
    }
}

impl Display for Minute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Minute::All => write!(f, "*"),
            Minute::Minutes(minutes) => {
                write!(f, "{}", as_two_digit(minutes))
            }
        }
    }
}
