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
    error::Error::InvalidMonth,
    schedule::{All, parse_time_chunk},
    utils::as_two_digit,
};

const MONTHS_PER_YEAR: u8 = 12;

/// The month for a realtime schedule
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Month {
    /// Every month
    #[default]
    All,
    /// Specific months
    Months(Vec<u8>),
}

impl Month {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Month::All => true,
            Month::Months(months) => months.contains(&given),
        }
    }

    pub(crate) fn quarterly() -> Self {
        Month::Months(vec![1, 4, 7, 10])
    }

    pub(crate) fn biannually() -> Self {
        Month::Months(vec![1, 7])
    }

    pub(crate) fn first() -> Self {
        Month::Months(vec![1])
    }
}

impl All for Month {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(1..=MONTHS_PER_YEAR);
        Month::Months(vec![rand_in_range])
    }
}

impl TryFrom<Vec<u8>> for Month {
    type Error = Error;

    fn try_from(values: Vec<u8>) -> Result<Self> {
        for &value in &values {
            if value == 0 || value > MONTHS_PER_YEAR {
                return Err(InvalidMonth(value.to_string()).into());
            }
        }
        Ok(Month::Months(values))
    }
}

impl TryFrom<u8> for Month {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Month::try_from(vec![value])
    }
}

impl TryFrom<&str> for Month {
    type Error = Error;

    fn try_from(monthish: &str) -> Result<Self> {
        parse_time_chunk::<Month>(monthish, MONTHS_PER_YEAR, true)
    }
}

impl FromStr for Month {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Month::try_from(s)
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Month::All => write!(f, "*"),
            Month::Months(months) => {
                write!(f, "{}", as_two_digit(months))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, sync::LazyLock};

    use crate::schedule::{RANGE_RE, REP_RE};

    use super::Month;
    use anyhow::Result;
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};
    use regex::Regex;

    static VALID_MONTH_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(1[0-2]|[1-9])$").unwrap());

    prop_compose! {
        pub fn month_strategy()(num in any::<u8>()) -> (String, u8) {
            let month = (num % 12) + 1;
            (month.to_string(), month)
        }
    }

    prop_compose! {
        pub fn invalid_month_strategy()(num in any::<u8>()) -> String {
            let month = if num > 0 && num <= 12 {
                num + 12
            } else {
                num
            };
            month.to_string()
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in month_strategy(), second in month_strategy()) -> (String, u8, u8) {
            let (first_str, first_val) = first;
            let (second_str, second_val) = second;
            if first_val <= second_val {
                (format!("{first_str}..{second_str}"), first_val, second_val)
            } else {
                (format!("{second_str}..{first_str}"), second_val, first_val)
            }
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in month_strategy(), second in month_strategy()) -> (String, u8, u8) {
            let (_, mut first_val) = first;
            let (_, second_val) = second;
            if first_val == second_val {
                first_val += 1;
            }
            match first_val.cmp(&second_val) {
                Ordering::Less | Ordering::Equal => (format!("{second_val}..{first_val}"), first_val, second_val),
                Ordering::Greater => (format!("{first_val}..{second_val}"), second_val, first_val),
            }
        }
    }

    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_MONTH_RE.is_match(s.as_str()));
            prop_assume!(!RANGE_RE.is_match(s.as_str()));
            prop_assume!(!REP_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Month::try_from(s.as_str()).is_err());
            assert!(s.parse::<Month>().is_err());
        }

        #[test]
        fn test_month_try_from_valid(value in month_strategy()) {
            let (month_str, _) = value;
            let month_res = Month::try_from(month_str.as_str());
            assert!(month_res.is_ok());
            let month_res = month_str.parse::<Month>();
            assert!(month_res.is_ok());
        }

        #[test]
        fn test_month_try_from_valid_u8(value in month_strategy()) {
            let (_, month_val) = value;
            let month_res = Month::try_from(month_val);
            assert!(month_res.is_ok());
        }

        #[test]
        fn invalid_month_errors(s in invalid_month_strategy()) {
            let month_res = Month::try_from(s.as_str());
            assert!(month_res.is_err());
            let month_res = s.parse::<Month>();
            assert!(month_res.is_err());
        }

        #[test]
        fn any_valid_range_str_works(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Month::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Month>().is_ok());
        }

        #[test]
        fn any_invalid_range_str_errors(s in arb_invalid_range()) {
            let (s, _, _) = s;
            assert!(Month::try_from(s.as_str()).is_err());
            assert!(s.parse::<Month>().is_err());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            match Month::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(month_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    let below = rng().random_range(u8::MIN..min);
                    let above = rng().random_range((max + 1)..=u8::MAX);
                    assert!(month_range.matches(in_range), "day {in_range} should match range '{range_str}'");
                    assert!(!month_range.matches(below), "day {below} should not match range '{range_str}'");
                    assert!(!month_range.matches(above), "day {above} should not match range '{range_str}'");
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Month::try_from("").is_err());
        assert!("".parse::<Month>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Month::All, Month::try_from("*")?);
        assert_eq!(Month::All, "*".parse::<Month>()?);
        Ok(())
    }

    #[test]
    fn rand() {
        assert!(Month::try_from("R").is_ok());
        assert!("R".parse::<Month>().is_ok());
    }

    #[test]
    fn all_display_works() {
        assert_eq!(Month::All.to_string(), "*");
        assert_eq!(Month::Months(vec![1, 2, 3, 4]).to_string(), "01,02,03,04");
    }
}
