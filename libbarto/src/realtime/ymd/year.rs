// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{str::FromStr, sync::LazyLock};

use anyhow::{Error, Result};
use regex::Regex;

use crate::{
    error::Error::InvalidYear,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

static YEAR_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(-?\d+)\.\.(-?\d+)$").expect("invalid year range regex"));
static YEAR_REPETITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(-?\d+)(\.\.(-?\d+))?/(\d+)$").expect("invalid repetition regex")
});

pub(crate) type Year = ConstrainedValue<i32>;

impl TryFrom<&str> for Year {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Year::parse(s)
    }
}

impl FromStr for Year {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Year::try_from(s)
    }
}

impl ConstrainedValueParser<'_, i32> for Year {
    fn invalid(s: &str) -> Error {
        InvalidYear(s.to_string()).into()
    }

    fn all() -> Self {
        Year::All
    }

    fn repetition_regex() -> Regex {
        YEAR_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        YEAR_RANGE_RE.clone()
    }

    fn rep(start: i32, end: Option<i32>, rep: u8) -> Self {
        Year::Repetition { start, end, rep }
    }

    fn range(first: i32, second: i32) -> Self {
        Year::Range(first, second)
    }

    fn specific(values: Vec<i32>) -> Self {
        Year::Specific(values)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::{cmp::Ordering, fmt::Write as _, sync::LazyLock};

    use anyhow::Result;
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};
    use regex::Regex;

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::Year;

    pub(crate) static VALID_I32_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"-?\d+").expect("invalid at least 4 digits regex"));

    // Valid strategies
    prop_compose! {
        pub(crate) fn arb_year() (year in any::<i32>()) -> (String, i32) {
            (year.to_string(), year)
        }
    }

    prop_compose! {
        pub(crate) fn arb_valid_year_range()(first in any::<i32>(), second in any::<i32>()) -> (String, i32, i32) {
            if first <= second {
                (format!("{first}..{second}"), first, second)
            } else {
                (format!("{second}..{first}"), second, first)
            }
        }
    }

    prop_compose! {
        fn arb_valid_repetition()(s in arb_valid_year_range(), rep in any::<u8>()) -> (String, i32, i32, u8) {
            let (mut prefix, min, max) = s;
            let rep = if rep == 0 { 1 } else { rep };
            write!(prefix, "/{rep}").unwrap();
            (prefix, min, max, rep)
        }
    }

    prop_compose! {
        fn arb_valid_repetition_no_end()(first in any::<i32>(), rep in any::<u8>()) -> String {
            let mut prefix = format!("{first}");
            let rep = if rep == 0 { 1 } else { rep };
            write!(prefix, "/{rep}").unwrap();
            prefix
        }
    }

    // Invalid strategies
    prop_compose! {
        fn arb_invalid_range()(mut first in any::<i32>(), second in any::<i32>()) -> String {
            if first == second {
                first += 1;
            }
            match first.cmp(&second) {
                Ordering::Less | Ordering::Equal => format!("{second}..{first}"),
                Ordering::Greater => format!("{first}..{second}"),
            }
        }
    }

    prop_compose! {
        fn arb_invalid_repetition()(s in arb_invalid_range(), rep in any::<u8>()) -> String {
            let mut prefix = s;
            write!(prefix, "/{rep}").unwrap();
            prefix
        }
    }

    prop_compose! {
        fn arb_invalid_repetition_zero_rep()(s in arb_valid_year_range()) -> String {
            let (mut prefix, _, _) = s;
            write!(prefix, "/0").unwrap();
            prefix
        }
    }

    // Valid inputs
    proptest! {
        #[test]
        fn arb_year_works(s in arb_year()) {
            let (year, _) = s;
            assert!(Year::try_from(year.as_str()).is_ok());
            assert!(year.parse::<Year>().is_ok());
        }

        #[test]
        fn arb_valid_year_range_works(s in arb_valid_year_range()) {
            let (s, _, _) = s;
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
        }

        #[test]
        fn arb_valid_year_repetition_works(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Year::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Year>().is_ok());
        }

        #[test]
        fn arb_valid_year_repetition_no_end_works(s in arb_valid_repetition_no_end()) {
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
        }
    }

    // Invalid inputs
    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_I32_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn arb_invalid_year_range_errors(s in arb_invalid_range()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn arb_invalid_year_repetition_errors(s in arb_invalid_repetition()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn arb_invalid_year_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_year_range()) {
            let (range_str, min, max) = s;
            match Year::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(year_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    assert!(year_range.matches(in_range), "day {in_range} should match range '{range_str}'");
                    if min > i32::MIN {
                        let below = rng().random_range(i32::MIN..min);
                        assert!(!year_range.matches(below), "day {below} should not match range '{range_str}'");
                    }
                    if max + 1 < i32::MAX {
                        let above = rng().random_range((max + 1)..=i32::MAX);
                        assert!(!year_range.matches(above), "day {above} should not match range '{range_str}'");
                    }
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Year::try_from("").is_err());
        assert!("".parse::<Year>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Year::All, Year::try_from("*")?);
        assert_eq!(Year::All, "*".parse::<Year>()?);
        Ok(())
    }
}
