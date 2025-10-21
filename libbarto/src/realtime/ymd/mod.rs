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

use std::{str::FromStr, sync::LazyLock};

use anyhow::{Error, Result};
use regex::Regex;

use crate::{
    error::Error::InvalidDate,
    realtime::ymd::{day::Day, month::Month, year::Year},
};

static YMD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(-?\d+)-(\d{1,2})-(\d{1,2})$").expect("invalid YMD regex"));

pub(crate) type YearMonthDayTuple = (Year, Month, Day);

pub(crate) struct YearMonthDay(pub(crate) Year, pub(crate) Month, pub(crate) Day);

impl YearMonthDay {
    pub(crate) fn take(self) -> YearMonthDayTuple {
        (self.0, self.1, self.2)
    }
}

impl TryFrom<&str> for YearMonthDay {
    type Error = Error;

    fn try_from(ymdish: &str) -> Result<Self> {
        if let Some(caps) = YMD_RE.captures(ymdish) {
            let year = caps[1].parse::<Year>()?;
            let month = caps[2].parse::<Month>()?;
            let day = caps[3].parse::<Day>()?;
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

#[cfg(test)]
mod test {
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };

    use crate::realtime::{
        cv::ConstrainedValueParser as _,
        ymd::{
            day::{
                DAY_RANGE_RE, DAY_REPETITION_RE, Day,
                test::{VALID_DAY_RE, day_strategy},
            },
            month::{
                MONTH_RANGE_RE, MONTH_REPETITION_RE, Month,
                tests::{VALID_MONTH_RE, month_strategy},
            },
            year::{Year, test::VALID_I32_RE},
        },
    };

    use super::YearMonthDay;

    // Valid strategies
    prop_compose! {
        fn arb_ymd() (year in any::<i32>(), month in month_strategy(), day in day_strategy()) -> (String, i32, u8, u8) {
            let (month_str, month_val) = month;
            let (day_str, day_val) = day;
            let ymd_str = format!("{}-{}-{}", year, month_str, day_str);
            (ymd_str, year, month_val, day_val)
        }
    }

    // Valid inputs
    proptest! {
        #[test]
        fn arb_ymd_works(s in arb_ymd()) {
            let (ymd_str, _, _, _) = s;
            assert!(YearMonthDay::try_from(ymd_str.as_str()).is_ok());
        }
    }

    // Invalid inputs
    proptest! {
        #[test]
        fn random_input_errors(year in "\\PC*", month in "\\PC*", day in "\\PC*") {
            prop_assume!(!VALID_I32_RE.is_match(year.as_str()));
            prop_assume!(year.as_str() != "*");
            prop_assume!(!VALID_MONTH_RE.is_match(month.as_str()));
            prop_assume!(!MONTH_REPETITION_RE.is_match(month.as_str()));
            prop_assume!(!MONTH_RANGE_RE.is_match(month.as_str()));
            prop_assume!(month.as_str() != "*");
            prop_assume!(month.as_str() != "R");
            prop_assume!(!VALID_DAY_RE.is_match(day.as_str()));
            prop_assume!(!DAY_REPETITION_RE.is_match(day.as_str()));
            prop_assume!(!DAY_RANGE_RE.is_match(day.as_str()));
            prop_assume!(day.as_str() != "*");
            prop_assume!(day.as_str() != "R");
            let ymd = format!("{}-{}-{}", year, month, day);
            assert!(YearMonthDay::try_from(ymd.as_str()).is_err());
            assert!(ymd.as_str().parse::<YearMonthDay>().is_err());
        }
    }

    #[test]
    fn take_works() {
        let ymd = YearMonthDay(Year::all(), Month::all(), Day::all());
        let (year, month, day) = ymd.take();
        assert_eq!(year, Year::all());
        assert_eq!(month, Month::all());
        assert_eq!(day, Day::all());
    }
}
