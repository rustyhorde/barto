// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod day;
pub(crate) mod month;
pub(crate) mod year;

use std::fmt::{Display, Formatter};

use anyhow::{Error, Result};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::{
    error::Error::InvalidDate,
    schedule::ymd::{day::Day, month::Month, year::Year},
};

// TODO: Fix this

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
            Err(InvalidDate(ymdish.to_string()).into())
        }
    }
}

impl Display for YearMonthDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod test {
    use crate::{Day, Month, Year};

    use super::YearMonthDay;

    use anyhow::Result;

    #[test]
    fn display_works() -> Result<()> {
        let ymd = YearMonthDay::builder()
            .year(Year::try_from("2025")?)
            .month(Month::first())
            .day(Day::try_from(15)?)
            .build();
        assert_eq!(ymd.to_string(), "2025 01 15");
        Ok(())
    }
}
