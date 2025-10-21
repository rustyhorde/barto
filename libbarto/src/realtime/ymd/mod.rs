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

use std::str::FromStr;

use anyhow::{Error, Result};

use crate::{
    error::Error::InvalidDate,
    realtime::ymd::{month::Month, year::Year},
};

pub(crate) type YearMonthDayTuple = (Year, Month, Option<Vec<u8>>);

pub(crate) struct YearMonthDay(
    pub(crate) Year,
    pub(crate) Month,
    pub(crate) Option<Vec<u8>>,
);

impl YearMonthDay {
    pub(crate) fn take(self) -> YearMonthDayTuple {
        (self.0, self.1, self.2)
    }
}

impl TryFrom<&str> for YearMonthDay {
    type Error = Error;

    fn try_from(ymdish: &str) -> Result<Self> {
        let date_parts: Vec<&str> = ymdish.split('-').collect();
        if date_parts.len() == 3 {
            let year = date_parts[0].parse::<Year>()?;
            let month = date_parts[1].parse::<Month>()?;
            let day = None;
            Ok(YearMonthDay(year, month, day))
        } else {
            Err(InvalidDate(ymdish.to_string()).into())
        }
    }
}

impl FromStr for YearMonthDay {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        YearMonthDay::try_from(s)
    }
}
