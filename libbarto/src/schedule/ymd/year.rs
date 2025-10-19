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
}

impl Display for Year {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Year::All => write!(f, "*"),
            Year::Range(lo, hi) => write!(f, "{lo}..{hi}"),
            Year::Repetition { start, end, rep } => {
                if let Some(end) = end {
                    write!(f, "{start}/{rep}..{end}")
                } else {
                    write!(f, "{start}/{rep}")
                }
            }
            Year::Year(year) => write!(f, "{year:04}"),
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
        } else if YEAR_RANGE_RE.is_match(yearish) {
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
        } else if YEAR_REP_RE.is_match(yearish) {
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
    use std::sync::LazyLock;

    use super::Year;

    use anyhow::Result;
    use proptest::prelude::*;
    use regex::Regex;

    static VALID_I32_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"-?\d+").expect("invalid at least 4 digits regex"));

    prop_compose! {
        fn arb_year() (year in any::<i32>()) -> String {
            year.to_string()
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in any::<i32>(), second in any::<i32>()) -> String {
            if first <= second {
                format!("{first}..{second}")
            } else {
                format!("{second}..{first}")
            }
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in any::<i32>(), second in any::<i32>()) -> String {
            if first < second {
                format!("{second}..{first}")
            } else if first == second {
                let new_first = first + 1;
                format!("{new_first}..{second}")
            } else {
                format!("{first}..{second}")
            }
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
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
        }

        #[test]
        fn any_invalid_range_str_errors(s in arb_invalid_range()) {
            assert!(Year::try_from(s.as_str()).is_err());
            assert!(s.parse::<Year>().is_err());
        }

        #[test]
        fn any_valid_range_str_works(s in arb_valid_range()) {
            assert!(Year::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Year>().is_ok());
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
