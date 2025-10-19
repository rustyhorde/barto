// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod month;
pub(crate) mod year;

use std::fmt::{Display, Formatter};

use anyhow::{Error, Result};
use bon::Builder;
use getset::{CopyGetters, Getters};
use rand::Rng;

use crate::{
    error::Error::{InvalidDate, InvalidDay},
    schedule::{
        All, parse_time_chunk,
        ymd::{month::Month, year::Year},
    },
    utils::as_two_digit,
};

// TODO: Fix this
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
        YearMonthDay::builder().day(Day::first()).build()
    }

    /// A quarterly schedule at the first day of the 1st, 4th, 7th, and 10th month
    #[must_use]
    pub fn quarterly() -> Self {
        YearMonthDay::builder()
            .month(Month::quarterly())
            .day(Day::first())
            .build()
    }

    /// A semiannual schedule at the first day of the 1st and 7th month
    #[must_use]
    pub fn semiannually() -> Self {
        YearMonthDay::builder()
            .month(Month::biannually())
            .day(Day::first())
            .build()
    }

    /// A yearly schedule at the first day of the first month
    #[must_use]
    pub fn yearly() -> Self {
        YearMonthDay::builder()
            .month(Month::first())
            .day(Day::first())
            .build()
    }
}

impl TryFrom<&str> for YearMonthDay {
    type Error = Error;

    fn try_from(ymdish: &str) -> Result<Self> {
        let date_parts: Vec<&str> = ymdish.split('-').collect();
        if date_parts.len() == 3 {
            let year = date_parts[0].try_into()?;
            let month = date_parts[1].try_into()?;
            let day = date_parts[2].try_into()?;
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
