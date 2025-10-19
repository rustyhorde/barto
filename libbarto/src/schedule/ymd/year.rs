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
    sync::LazyLock,
};

use anyhow::{Error, Result};
use regex::Regex;

use crate::error::Error::InvalidYear;

static YEAR_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(-?\d+)\.\.(-?\d+)").expect("invalid year range regex"));
static YEAR_REP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(-?\d+)(\.\.(-?\d+))?/(\d+)").expect("invalid repetition regex"));

/// The year for a realtime schedule
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Year {
    /// Every year
    #[default]
    All,
    /// A range of years
    Range(i32, i32),
    /// A repetition of years
    ///
    /// This is a sequence of years: start, start + rep, start + 2*rep
    /// up to the optional end year.
    Repetition {
        /// The year to start
        start: i32,
        /// An optional end year
        end: Option<i32>,
        /// The repetition value
        rep: u8,
    },
    /// Specific years
    Year(i32),
}

impl Year {
    pub(crate) fn matches(&self, given: i32) -> bool {
        match self {
            Year::All => true,
            Year::Range(lo, hi) => *lo <= given && given <= *hi,
            Year::Repetition { start, end, rep } => if let Some(end) = end {
                *start..=*end
            } else {
                *start..=i32::MAX
            }
            .step_by(usize::from(*rep))
            .any(|x| x == given),
            Year::Year(year) => *year == given,
        }
    }

    fn invalid_year(year: &str) -> Error {
        InvalidYear(year.to_string()).into()
    }

    fn parse_years_range(yearish: &str) -> Result<Self> {
        if let Some(caps) = YEAR_RANGE_RE.captures(yearish) {
            let first = caps[1]
                .parse::<i32>()
                .map_err(|_| Self::invalid_year(yearish))?;
            let second = caps[2]
                .parse::<i32>()
                .map_err(|_| Self::invalid_year(yearish))?;
            if second < first {
                Err(Self::invalid_year(yearish))
            } else {
                Ok(Year::Range(first, second))
            }
        } else {
            Err(Self::invalid_year(yearish))
        }
    }

    fn parse_years_repetition(yearish: &str) -> Result<Self> {
        if let Some(caps) = YEAR_REP_RE.captures(yearish) {
            let start = caps[1]
                .parse::<i32>()
                .map_err(|_| Self::invalid_year(yearish))?;
            let end = if let Some(end_match) = caps.get(3) {
                Some(
                    end_match
                        .as_str()
                        .parse::<i32>()
                        .map_err(|_| Self::invalid_year(yearish))?,
                )
            } else {
                None
            };
            let rep = caps[4]
                .parse::<u8>()
                .map_err(|_| Self::invalid_year(yearish))?;
            if rep == 0 {
                Err(Self::invalid_year(yearish))
            } else if let Some(end_val) = end {
                if end_val < start {
                    Err(Self::invalid_year(yearish))
                } else {
                    Ok(Year::Repetition { start, end, rep })
                }
            } else {
                Ok(Year::Repetition { start, end, rep })
            }
        } else {
            Err(Self::invalid_year(yearish))
        }
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Year::All => write!(f, "*"),
            Year::Range(lo, hi) => write!(f, "{lo}..{hi}"),
            Year::Repetition { start, end, rep } => {
                if let Some(end) = end {
                    write!(f, "{start}..{end}/{rep}")
                } else {
                    write!(f, "{start}/{rep}")
                }
            }
            Year::Year(year) => write!(f, "{year}"),
        }
    }
}

impl TryFrom<&str> for Year {
    type Error = Error;

    fn try_from(yearish: &str) -> Result<Self> {
        if yearish.is_empty() {
            Err(Self::invalid_year(yearish))
        } else if yearish == "*" {
            Ok(Year::All)
        } else if YEAR_REP_RE.is_match(yearish) {
            Self::parse_years_repetition(yearish)
        } else if YEAR_RANGE_RE.is_match(yearish) {
            Self::parse_years_range(yearish)
        } else {
            Ok(Year::Year(yearish.parse::<i32>()?))
        }
    }
}

impl FromStr for Year {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Year::try_from(s)
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, fmt::Write as _, sync::LazyLock};

    use crate::schedule::ymd::year::YEAR_RANGE_RE;

    use super::Year;

    use anyhow::Result;
    use proptest::prelude::*;
    use rand::{rng, seq::IndexedRandom};
    use regex::Regex;

    static VALID_I32_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"-?\d+").expect("invalid at least 4 digits regex"));

    prop_compose! {
        fn arb_year() (year in any::<i32>()) -> (String, i32) {
            (year.to_string(), year)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in any::<i32>(), second in any::<i32>()) -> (String, i32, i32) {
            if first <= second {
                (format!("{first}..{second}"), first, second)
            } else {
                (format!("{second}..{first}"), second, first)
            }
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in any::<i32>(), second in any::<i32>()) -> String {
            match first.cmp(&second) {
                Ordering::Less | Ordering::Equal => format!("{second}..{first}"),
                Ordering::Greater => format!("{first}..{second}"),
            }
        }
    }

    prop_compose! {
        fn arb_valid_repetition()(s in arb_valid_range(), rep in any::<u8>()) -> (String, i32, i32, u8) {
            let (mut prefix, min, max) = s;
            let rep = if rep == 0 { 1 } else { rep };
            write!(prefix, "/{rep}").unwrap();
            (prefix, min, max, rep)
        }
    }

    prop_compose! {
        fn arb_invalid_repetition_zero_rep()(s in arb_valid_range()) -> String {
            let (mut prefix, _, _) = s;
            write!(prefix, "/0").unwrap();
            prefix
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
        fn arb_valid_repetition_no_end()(first in any::<i32>(), rep in any::<u8>()) -> String {
            let mut prefix = format!("{first}");
            let rep = if rep == 0 { 1 } else { rep };
            write!(prefix, "/{rep}").unwrap();
            prefix
        }
    }

    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_I32_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_valid_i32_str_works(s in arb_year()) {
            let (year, _) = s;
            assert!(Year::try_from(year.as_str()).is_ok());
            assert!(year.parse::<Year>().is_ok());
        }

        #[test]
        fn any_invalid_range_str_errors(s in arb_invalid_range()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_valid_range_str_works(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
        }

        #[test]
        fn invalid_range_errors(s in "\\PC*") {
            prop_assume!(!YEAR_RANGE_RE.is_match(s.as_str()));
            assert!(Year::parse_years_range(s.as_str()).is_err());
        }

        #[test]
        fn any_valid_repetition_str_works(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Year::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Year>().is_ok());
        }

        #[test]
        fn any_valid_repetition_no_end_str_works(s in arb_valid_repetition_no_end()) {
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
        }

        #[test]
        fn invalid_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_invalid_repetition_str_errors(s in arb_invalid_repetition()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_valid_i32_matches(s in arb_year()) {
            let (year, year_val) = s;
            let year = Year::try_from(year.as_str()).expect("valid year failed to parse");
            assert!(year.matches(year_val));
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            match Year::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(year_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    let below = rng().random_range(i32::MIN..min);
                    let above = rng().random_range(max..=i32::MAX);
                    assert!(year_range.matches(in_range), "day {in_range} should match range '{range_str}'");
                    assert!(!year_range.matches(below), "day {below} should not match range '{range_str}'");
                    assert!(!year_range.matches(above), "day {above} should not match range '{range_str}'");
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

    #[test]
    fn all_display_works() {
        assert_eq!(Year::All.to_string(), "*");
        assert_eq!(Year::Year(2025).to_string(), "2025");
        assert_eq!(Year::Range(2020, 2025).to_string(), "2020..2025");
        assert_eq!(
            Year::Repetition {
                start: 2020,
                end: Some(2030),
                rep: 2
            }
            .to_string(),
            "2020..2030/2"
        );
        assert_eq!(
            Year::Repetition {
                start: 2020,
                end: None,
                rep: 2
            }
            .to_string(),
            "2020/2"
        );
    }

    #[test]
    fn all_debug_works() {
        assert_eq!(format!("{:?}", Year::All), "All");
        assert_eq!(format!("{:?}", Year::Year(2025)), "Year(2025)");
        assert_eq!(
            format!("{:?}", Year::Range(2020, 2025)),
            "Range(2020, 2025)"
        );
        assert_eq!(
            format!(
                "{:?}",
                Year::Repetition {
                    start: 2020,
                    end: Some(2030),
                    rep: 2
                }
            ),
            "Repetition { start: 2020, end: Some(2030), rep: 2 }"
        );
        assert_eq!(
            format!(
                "{:?}",
                Year::Repetition {
                    start: 2020,
                    end: None,
                    rep: 2
                }
            ),
            "Repetition { start: 2020, end: None, rep: 2 }"
        );
    }

    #[test]
    fn invalid_caps() {
        assert!(Year::parse_years_range("invalid").is_err());
        assert!(Year::parse_years_repetition("invalid").is_err());
    }

    #[test]
    fn valid_repetition_matches() {
        let min = 2000;
        let max = 4000;
        let rep = 2u8;
        let range_str = format!("{min}..{max}/{rep}");
        let all_range = (2000..=4000)
            .step_by(usize::from(rep))
            .collect::<Vec<i32>>();
        let res = Year::try_from(range_str.as_str());
        assert!(res.is_ok());
        let year_range = res.unwrap();
        for _ in 0..256 {
            let in_range = all_range.choose(&mut rng()).unwrap();
            let below = rng().random_range(i32::MIN..min);
            let above = rng().random_range(max..=i32::MAX);
            assert!(year_range.matches(*in_range));
            assert!(!year_range.matches(below));
            assert!(!year_range.matches(above));
        }
    }

    #[test]
    fn valid_repetition_no_end_matches() {
        let min = i32::MAX - 4000;
        let rep = 2u8;
        let range_str = format!("{min}/{rep}");
        let all_range = (min..=i32::MAX)
            .step_by(usize::from(rep))
            .collect::<Vec<i32>>();
        let res = Year::try_from(range_str.as_str());
        assert!(res.is_ok());
        let year_range = res.unwrap();
        for _ in 0..256 {
            let in_range = all_range.choose(&mut rng()).unwrap();
            let below = rng().random_range(i32::MIN..min);
            assert!(year_range.matches(*in_range));
            assert!(!year_range.matches(below));
        }
    }

    #[test]
    fn eq_works() {
        assert!(Year::All == Year::default());
        assert!(Year::All == Year::All);
        assert!(Year::Year(2025) == Year::Year(2025));
        assert!(Year::Range(2020, 2025) == Year::Range(2020, 2025));
        assert!(
            Year::Repetition {
                start: 2020,
                end: Some(2030),
                rep: 2
            } == Year::Repetition {
                start: 2020,
                end: Some(2030),
                rep: 2
            }
        );
    }

    #[test]
    fn hash_works() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        assert_eq!(calculate_hash(&Year::All), calculate_hash(&Year::default()));
        assert_eq!(
            calculate_hash(&Year::Year(2025)),
            calculate_hash(&Year::Year(2025))
        );
        assert_eq!(
            calculate_hash(&Year::Range(2020, 2025)),
            calculate_hash(&Year::Range(2020, 2025))
        );
        assert_eq!(
            calculate_hash(&Year::Repetition {
                start: 2020,
                end: Some(2030),
                rep: 2
            }),
            calculate_hash(&Year::Repetition {
                start: 2020,
                end: Some(2030),
                rep: 2
            })
        );
    }

    #[test]
    #[allow(clippy::clone_on_copy)] // for testing purposes
    fn clone_works() {
        let year = Year::Repetition {
            start: 2020,
            end: Some(2030),
            rep: 2,
        };
        let cloned_year = year.clone();
        assert_eq!(year, cloned_year);
    }

    #[test]
    fn copy_works() {
        let year = Year::Year(2025);
        let copied_year = year;
        assert_eq!(year, copied_year);
    }
}
