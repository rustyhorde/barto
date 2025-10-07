// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter};

use anyhow::{Error, Result, anyhow};
use bon::Builder;
use getset::{CopyGetters, Getters};
use rand::Rng;

use crate::{
    error::Error::InvalidDate,
    schedule::{All, RANGE_RE, parse_time_chunk},
    utils::as_two_digit,
};

const MONTHS_PER_YEAR: u8 = 12;
// TODO: Fix this
const DAYS_PER_MONTH: u8 = 31;

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
                *start..=9999
            }
            .step_by(usize::from(*rep))
            .any(|x| x == given),
            Year::Year(year) => *year == given,
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
                    write!(f, "{start}/{rep}..{end}")
                } else {
                    write!(f, "{start}/{rep}")
                }
            }
            Year::Year(year) => write!(f, "{year:04}"),
        }
    }
}

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
}

impl All for Month {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(1..13);
        Month::Months(vec![rand_in_range])
    }
}

impl From<Vec<u8>> for Month {
    fn from(value: Vec<u8>) -> Self {
        Month::Months(value)
    }
}

impl From<u8> for Month {
    fn from(value: u8) -> Self {
        Month::Months(vec![value])
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
}

impl All for Day {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(1..29);
        Day::Days(vec![rand_in_range])
    }
}

impl From<Vec<u8>> for Day {
    fn from(value: Vec<u8>) -> Self {
        Day::Days(value)
    }
}

impl From<u8> for Day {
    fn from(value: u8) -> Self {
        Day::Days(vec![value])
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

/// A year month day combinations
#[derive(Builder, Clone, CopyGetters, Debug, Default, Eq, Getters, Hash, PartialEq)]
pub struct YearMonthDay {
    /// The year(s) to run
    #[builder(default = Year::All, into)]
    #[getset(get_copy = "pub")]
    year: Year,
    /// The month(s) to run
    #[builder(default = Month::All, into)]
    #[getset(get = "pub")]
    month: Month,
    /// The day(s) to run
    #[builder(default = Day::All, into)]
    #[getset(get = "pub")]
    day: Day,
}

impl YearMonthDay {
    /// A monthly schedule at the first day of the month
    #[must_use]
    pub fn monthly() -> Self {
        YearMonthDay::builder().day(1).build()
    }

    /// A quarterly schedule at the first day of the 1st, 4th, 7th, and 10th month
    #[must_use]
    pub fn quarterly() -> Self {
        YearMonthDay::builder()
            .month(vec![1, 4, 7, 10])
            .day(1)
            .build()
    }

    /// A semiannual schedule at the first day of the 1st and 7th month
    #[must_use]
    pub fn semiannually() -> Self {
        YearMonthDay::builder().month(vec![1, 7]).day(1).build()
    }

    /// A yearly schedule at the first day of the first month
    #[must_use]
    pub fn yearly() -> Self {
        YearMonthDay::builder().month(1).day(1).build()
    }
}

impl TryFrom<&str> for YearMonthDay {
    type Error = Error;

    fn try_from(ymdish: &str) -> Result<Self> {
        let date_parts: Vec<&str> = ymdish.split('-').collect();
        if date_parts.len() == 3 {
            let year = parse_year(date_parts[0])?;
            let month = parse_time_chunk::<Month>(date_parts[1], MONTHS_PER_YEAR, true)?;
            let day = parse_time_chunk::<Day>(date_parts[2], DAYS_PER_MONTH, true)?;
            Ok(YearMonthDay { year, month, day })
        } else {
            Err(InvalidDate {
                date: ymdish.to_string(),
            }
            .into())
        }
    }
}

impl Display for YearMonthDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.year, self.month, self.day)
    }
}

fn parse_year(yearish: &str) -> Result<Year> {
    Ok(if yearish == "*" {
        Year::All
    } else if RANGE_RE.is_match(yearish) {
        let caps = RANGE_RE.captures(yearish).ok_or_else(|| anyhow!(""))?;
        let first = caps
            .get(1)
            .ok_or_else(|| anyhow!(""))?
            .as_str()
            .parse::<i32>()?;
        let second = caps
            .get(2)
            .ok_or_else(|| anyhow!(""))?
            .as_str()
            .parse::<i32>()?;
        Year::Range(first, second)
    } else {
        Year::Year(yearish.parse::<i32>()?)
    })
}
