// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::{Error, Result};
use rand::Rng as _;

use crate::{
    schedule::{All, parse_time_chunk},
    utils::as_two_digit,
};

const MIN_HOUR_IN_DAY: u8 = 0;
const MAX_HOUR_IN_DAY: u8 = 23;
pub(crate) const HOURS_PER_DAY: u8 = 23;

/// The hour for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Hour {
    /// Every hour
    All,
    /// Specific hours
    Hours(Vec<u8>),
}

impl Hour {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Hour::All => true,
            Hour::Hours(hours) => hours.contains(&given),
        }
    }

    pub(crate) fn midnight() -> Self {
        Hour::Hours(vec![0])
    }
}

impl Default for Hour {
    fn default() -> Self {
        Self::Hours(vec![0])
    }
}

impl All for Hour {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(MIN_HOUR_IN_DAY..=MAX_HOUR_IN_DAY);
        Hour::Hours(vec![rand_in_range])
    }
}

impl TryFrom<Vec<u8>> for Hour {
    type Error = Error;

    fn try_from(values: Vec<u8>) -> Result<Self> {
        for &value in &values {
            if value > MAX_HOUR_IN_DAY {
                return Err(Error::msg(format!("Invalid hour value: {value}")));
            }
        }
        Ok(Hour::Hours(values))
    }
}

impl TryFrom<u8> for Hour {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Hour::try_from(vec![value])
    }
}

impl TryFrom<&str> for Hour {
    type Error = Error;

    fn try_from(hourish: &str) -> Result<Self> {
        parse_time_chunk::<Hour>(hourish, HOURS_PER_DAY, false)
    }
}

impl FromStr for Hour {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Hour::try_from(s)
    }
}

impl Display for Hour {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Hour::All => write!(f, "*"),
            Hour::Hours(hours) => {
                write!(f, "{}", as_two_digit(hours))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::schedule::{RANGE_RE, REP_RE};

    use super::{HOURS_PER_DAY, Hour};

    use std::sync::LazyLock;

    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use regex::Regex;

    static VALID_HOUR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([0-9]|[1][0-9]|2[0-3])$").unwrap());

    prop_compose! {
        pub fn hour_strategy()(num in any::<u8>()) -> (String, u8) {
            let hour = (num % HOURS_PER_DAY) + 1;
            (hour.to_string(), hour)
        }
    }

    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_HOUR_RE.is_match(s.as_str()));
            prop_assume!(!RANGE_RE.is_match(s.as_str()));
            prop_assume!(!REP_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Hour::try_from(s.as_str()).is_err());
            assert!(s.parse::<Hour>().is_err());
        }

        #[test]
        fn test_hour_try_from_valid(value in hour_strategy()) {
            let (hour_str, _) = value;
            let hour_res = Hour::try_from(hour_str.as_str());
            assert!(hour_res.is_ok());
            let hour_res = hour_str.parse::<Hour>();
            assert!(hour_res.is_ok());
        }
    }
}
