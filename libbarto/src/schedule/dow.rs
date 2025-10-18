// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use regex::Regex;
use time::Weekday;

use crate::{
    error::Error::{InvalidDayOfWeek, InvalidRange},
    utils::until_err,
};

static DOW_RANGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-zA-Z]{3,})\.\.([a-zA-Z]{3,})").expect("invalid day of week range regex")
});

/// The day of the week for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DayOfWeek {
    /// Every day of the week
    All,
    /// Specific days of the week
    Days(Vec<u8>),
}

impl DayOfWeek {
    pub(crate) fn matches(&self, given: Weekday) -> bool {
        match self {
            DayOfWeek::All => true,
            DayOfWeek::Days(days) => {
                let given_u = match given {
                    Weekday::Sunday => 0,
                    Weekday::Monday => 1,
                    Weekday::Tuesday => 2,
                    Weekday::Wednesday => 3,
                    Weekday::Thursday => 4,
                    Weekday::Friday => 5,
                    Weekday::Saturday => 6,
                };
                days.contains(&given_u)
            }
        }
    }

    fn parse_dowish(dowish: &str) -> Result<Vec<u8>> {
        if DOW_RANGE_RE.is_match(dowish) {
            Self::parse_dow_range(dowish)
        } else {
            Self::parse_dow_v(dowish)
        }
    }

    fn parse_dow_range(dow_range: &str) -> Result<Vec<u8>> {
        if let Some(caps) = DOW_RANGE_RE.captures(dow_range) {
            let first = Self::parse_dow(&caps[1])?;
            let second = Self::parse_dow(&caps[2])?;
            if second < first {
                Err(InvalidRange(dow_range.to_string()).into())
            } else {
                Ok((first..=second).collect())
            }
        } else {
            Err(InvalidRange(dow_range.to_string()).into())
        }
    }

    fn parse_dow_v(dow: &str) -> Result<Vec<u8>> {
        Self::parse_dow(dow).map(|x| vec![x])
    }

    fn parse_dow(dow: &str) -> Result<u8> {
        if dow.len() > 9 {
            Err(Self::invalid_dow(dow))
        } else {
            let res = if dow == "Sun" || dow == "Sunday" {
                0
            } else if dow == "Mon" || dow == "Monday" {
                1
            } else if dow == "Tue" || dow == "Tuesday" {
                2
            } else if dow == "Wed" || dow == "Wednesday" {
                3
            } else if dow == "Thu" || dow == "Thursday" {
                4
            } else if dow == "Fri" || dow == "Friday" {
                5
            } else if dow == "Sat" || dow == "Saturday" {
                6
            } else {
                return Err(Self::invalid_dow(dow));
            };
            Ok(res)
        }
    }

    fn invalid_dow(dow: &str) -> Error {
        InvalidDayOfWeek(dow.to_string()).into()
    }
}

impl From<u8> for DayOfWeek {
    fn from(value: u8) -> Self {
        DayOfWeek::Days(vec![value])
    }
}

impl From<Vec<u8>> for DayOfWeek {
    fn from(value: Vec<u8>) -> Self {
        DayOfWeek::Days(value)
    }
}

impl TryFrom<String> for DayOfWeek {
    type Error = Error;

    fn try_from(dowish: String) -> Result<Self> {
        Self::try_from(dowish.as_str())
    }
}

impl TryFrom<&str> for DayOfWeek {
    type Error = Error;

    fn try_from(dowish: &str) -> Result<Self> {
        if dowish.is_empty() {
            Err(Self::invalid_dow(dowish))
        } else if dowish == "*" {
            Ok(DayOfWeek::All)
        } else {
            let mut err = Ok(());
            let mut dows: Vec<u8> = dowish
                .split(',')
                .map(Self::parse_dowish)
                .scan(&mut err, until_err)
                .flatten()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            err?;
            dows.sort_unstable();
            Ok(DayOfWeek::Days(dows))
        }
    }
}

impl FromStr for DayOfWeek {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        DayOfWeek::try_from(s)
    }
}

impl Display for DayOfWeek {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DayOfWeek::All => {
                write!(f, "*")?;
            }
            DayOfWeek::Days(vals) => {
                let len = vals.len();
                for (idx, val) in vals.iter().enumerate() {
                    match val {
                        0 => write!(f, "Sun")?,
                        1 => write!(f, "Mon")?,
                        2 => write!(f, "Tue")?,
                        3 => write!(f, "Wed")?,
                        4 => write!(f, "Thu")?,
                        5 => write!(f, "Fri")?,
                        6 => write!(f, "Sat")?,
                        _ => write!(f, "Unk")?,
                    }
                    if idx < len - 1 {
                        write!(f, ", ")?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use super::DayOfWeek;
    use anyhow::{Result, anyhow};
    use itertools::Itertools as _;
    use proptest::prelude::*;

    static SHORT_DOWS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    static LONG_DOWS: &[&str] = &[
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    static ALL_DOWS: LazyLock<Vec<&str>> = LazyLock::new(|| {
        SHORT_DOWS
            .iter()
            .chain(LONG_DOWS.iter())
            .copied()
            .collect::<Vec<&str>>()
    });

    // #[allow(dead_code)]
    // fn valid_ranges() -> Vec<String> {
    //     SHORT_DOWS
    //         .iter()
    //         .cloned()
    //         .chain(LONG_DOWS.iter().cloned())
    //         .permutations(2)
    //         .filter_map(|v| {
    //             let vals = v
    //                 .iter()
    //                 .filter_map(|x| parse_dow(x).ok())
    //                 .collect::<Vec<u8>>();
    //             if vals[0] < vals[1] { Some(v) } else { None }
    //         })
    //         .map(|v| format!("{}..{}", v[0], v[1]))
    //         .collect()
    // }

    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!ALL_DOWS.contains(&s.as_str()));
            prop_assume!(s != "*");
            assert!(DayOfWeek::try_from(s.as_str()).is_err());
            assert!(s.parse::<DayOfWeek>().is_err());
        }

        #[test]
        fn input_too_long_errors(s in "[a-zA-Z]{10,}") {
            assert!(DayOfWeek::try_from(s.as_str()).is_err());
            assert!(s.parse::<DayOfWeek>().is_err());
        }

        #[test]
        fn input_invalid_errors(s in "[a-zA-Z]{0,9}") {
            prop_assume!(!ALL_DOWS.contains(&s.as_str()));
            assert!(DayOfWeek::try_from(s.as_str()).is_err());
            assert!(s.parse::<DayOfWeek>().is_err());
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(DayOfWeek::try_from("").is_err());
        assert!("".parse::<DayOfWeek>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(DayOfWeek::All, TryFrom::try_from("*")?);
        Ok(())
    }

    #[test]
    fn valid_single_dow() {
        for dow in ALL_DOWS.iter() {
            assert!(DayOfWeek::try_from(*dow).is_ok());
        }
    }

    #[test]
    fn comma_separated_input() {
        let ps = SHORT_DOWS
            .iter()
            .copied()
            .powerset()
            .filter(|v| !v.is_empty())
            .map(|v| v.join(","))
            .collect::<Vec<String>>();
        for p in ps {
            assert!(
                DayOfWeek::try_from(p.as_str()).is_ok(),
                "Failed on input: {p}",
            );
            assert!(p.parse::<DayOfWeek>().is_ok(), "Failed on input: {p}");
        }
    }

    #[test]
    fn valid_ranges() {
        let valid_ranges = SHORT_DOWS
            .iter()
            .copied()
            .chain(LONG_DOWS.iter().copied())
            .permutations(2)
            .filter_map(|v| {
                let vals = v
                    .iter()
                    .filter_map(|x| DayOfWeek::parse_dow(x).ok())
                    .collect::<Vec<u8>>();
                if vals[0] < vals[1] { Some(v) } else { None }
            })
            .map(|v| format!("{}..{}", v[0], v[1]))
            .collect::<Vec<String>>();
        for range in &valid_ranges {
            assert!(
                DayOfWeek::try_from(range.as_str()).is_ok(),
                "Failed on input: {range}"
            );
            assert!(
                range.parse::<DayOfWeek>().is_ok(),
                "Failed on input: {range}"
            );
        }
    }

    #[test]
    fn day_already_in_range() -> Result<()> {
        assert_eq!(
            DayOfWeek::Days(vec![1, 2, 3, 4, 5]),
            TryFrom::try_from("Mon..Fri,Tue")?
        );
        assert_eq!(
            DayOfWeek::Days(vec![1, 2, 3, 4, 5]),
            TryFrom::try_from("Monday..Friday,Tuesday")?
        );
        Ok(())
    }

    #[test]
    fn one_day_range() -> Result<()> {
        assert_eq!(
            DayOfWeek::Days(vec![1, 5]),
            TryFrom::try_from("Mon..Mon,Fri..Fri")?
        );
        assert_eq!(
            DayOfWeek::Days(vec![1, 5]),
            TryFrom::try_from("Monday..Monday,Friday..Friday")?
        );
        Ok(())
    }

    #[test]
    fn out_of_order() -> Result<()> {
        assert_eq!(
            DayOfWeek::Days(vec![0, 1, 2, 3, 4, 6]),
            TryFrom::try_from("Mon..Thu,Sat,Sun")?
        );
        assert_eq!(
            DayOfWeek::Days(vec![0, 1, 2, 3, 4, 6]),
            TryFrom::try_from("Monday..Thursday,Saturday,Sunday")?
        );
        Ok(())
    }

    #[test]
    fn invalid_range() -> Result<()> {
        match <DayOfWeek>::try_from("Mon..Hogwash,Wed") {
            Ok(_) => Err(anyhow!("this day of week should be invalid")),
            Err(e) => {
                assert_eq!(format!("{e}"), "invalid day of week: 'Hogwash'");
                Ok(())
            }
        }
    }

    #[test]
    fn invalid_range_order() {
        let invalid_ranges = SHORT_DOWS
            .iter()
            .copied()
            .chain(LONG_DOWS.iter().copied())
            .permutations(2)
            .filter_map(|v| {
                let vals = v
                    .iter()
                    .filter_map(|x| DayOfWeek::parse_dow(x).ok())
                    .collect::<Vec<u8>>();
                if vals[0] > vals[1] { Some(v) } else { None }
            })
            .map(|v| format!("{}..{}", v[0], v[1]))
            .collect::<Vec<String>>();
        for range in &invalid_ranges {
            assert!(
                DayOfWeek::try_from(range.as_str()).is_err(),
                "{range} should be invalid"
            );
            assert!(
                range.parse::<DayOfWeek>().is_err(),
                "{range} should be invalid"
            );
        }
    }
}
