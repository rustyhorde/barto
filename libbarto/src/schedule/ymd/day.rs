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
    error::Error::InvalidDay,
    schedule::{All, parse_time_chunk},
    utils::as_two_digit,
};

const DAYS_PER_MONTH: u8 = 31;
const DAYS_PER_MONTH_RAND: u8 = 28;

/// The date for a realtime schedule
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Day {
    /// Every day
    #[default]
    All,
    /// Specific days
    Days(Vec<u8>),
}

impl Day {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Day::All => true,
            Day::Days(days) => days.contains(&given),
        }
    }

    pub(crate) fn first() -> Self {
        Day::Days(vec![1])
    }
}

impl All for Day {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(1..=DAYS_PER_MONTH_RAND);
        Day::Days(vec![rand_in_range])
    }
}

impl TryFrom<Vec<u8>> for Day {
    type Error = Error;

    fn try_from(values: Vec<u8>) -> Result<Self> {
        for &value in &values {
            if value == 0 || value > DAYS_PER_MONTH {
                return Err(InvalidDay(value.to_string()).into());
            }
        }
        Ok(Day::Days(values))
    }
}

impl TryFrom<u8> for Day {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Day::try_from(vec![value])
    }
}

impl TryFrom<&str> for Day {
    type Error = Error;

    fn try_from(dayish: &str) -> Result<Self> {
        parse_time_chunk::<Day>(dayish, DAYS_PER_MONTH, true)
    }
}

impl FromStr for Day {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Day::try_from(s)
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Day::All => write!(f, "*"),
            Day::Days(days) => {
                write!(f, "{}", as_two_digit(days))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::schedule::{RANGE_RE, REP_RE};

    use super::{DAYS_PER_MONTH, Day};

    use std::{cmp::Ordering, sync::LazyLock};

    use anyhow::Result;
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};
    use regex::Regex;

    static VALID_DAY_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([1-9]|[12][0-9]|3[01])$").unwrap());

    prop_compose! {
        pub fn day_strategy()(num in any::<u8>()) -> (String, u8) {
            let day = (num % DAYS_PER_MONTH) + 1;
            (day.to_string(), day)
        }
    }

    prop_compose! {
        pub fn invalid_day_strategy()(num in any::<u8>()) -> String {
            let day = if num > 0 && num <= DAYS_PER_MONTH {
                num + DAYS_PER_MONTH
            } else {
                num
            };
            day.to_string()
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in day_strategy(), second in day_strategy()) -> (String, u8, u8) {
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
        fn arb_invalid_range()(first in day_strategy(), second in day_strategy()) -> (String, u8, u8) {
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
            prop_assume!(!VALID_DAY_RE.is_match(s.as_str()));
            prop_assume!(!RANGE_RE.is_match(s.as_str()));
            prop_assume!(!REP_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Day::try_from(s.as_str()).is_err());
            assert!(s.parse::<Day>().is_err());
        }

        #[test]
        fn test_day_try_from_valid(value in day_strategy()) {
            let (day_str, _) = value;
            let day_res = Day::try_from(day_str.as_str());
            assert!(day_res.is_ok());
            let day_res = day_str.parse::<Day>();
            assert!(day_res.is_ok());
        }

        #[test]
        fn test_day_try_from_valid_u8(value in day_strategy()) {
            let (_, day_val) = value;
            let day_res = Day::try_from(day_val);
            assert!(day_res.is_ok());
        }

        #[test]
        fn invalid_day_errors(s in invalid_day_strategy()) {
            let day_res = Day::try_from(s.as_str());
            assert!(day_res.is_err());
            let day_res = s.parse::<Day>();
            assert!(day_res.is_err());
        }

        #[test]
        fn any_valid_range_str_works(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Day::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Day>().is_ok());
        }

        #[test]
        fn any_invalid_range_str_errors(s in arb_invalid_range()) {
            let (s, _, _) = s;
            assert!(Day::try_from(s.as_str()).is_err());
            assert!(s.parse::<Day>().is_err());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            match Day::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(day_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    let below = rng().random_range(u8::MIN..min);
                    let above = rng().random_range((max + 1)..=u8::MAX);
                    assert!(day_range.matches(in_range), "day {in_range} should match range '{range_str}'");
                    assert!(!day_range.matches(below), "day {below} should not match range '{range_str}'");
                    assert!(!day_range.matches(above), "day {above} should not match range '{range_str}'");
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Day::try_from("").is_err());
        assert!("".parse::<Day>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Day::All, Day::try_from("*")?);
        assert_eq!(Day::All, "*".parse::<Day>()?);
        Ok(())
    }

    #[test]
    fn rand() {
        assert!(Day::try_from("R").is_ok());
        assert!("R".parse::<Day>().is_ok());
    }

    #[test]
    fn all_display_works() {
        assert_eq!(Day::All.to_string(), "*");
        assert_eq!(Day::Days(vec![1, 2, 3, 4]).to_string(), "01,02,03,04");
    }
}
