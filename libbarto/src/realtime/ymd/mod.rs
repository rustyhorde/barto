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

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::{Error, Result};

use crate::{
    error::Error::InvalidDate,
    realtime::ymd::{day::Day, month::Month, year::Year},
};

pub(crate) type YearMonthDayTuple = (Year, Month, Day);

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct YearMonthDay(pub(crate) Year, pub(crate) Month, pub(crate) Day);

impl YearMonthDay {
    pub(crate) fn take(self) -> YearMonthDayTuple {
        (self.0, self.1, self.2)
    }

    pub(crate) fn monthly() -> Self {
        YearMonthDay(Year::default(), Month::default(), Day::first())
    }

    pub(crate) fn quarterly() -> Self {
        YearMonthDay(Year::default(), Month::quarterly(), Day::first())
    }

    pub(crate) fn semiannually() -> Self {
        YearMonthDay(Year::default(), Month::semiannually(), Day::first())
    }

    pub(crate) fn yearly() -> Self {
        YearMonthDay(Year::default(), Month::first(), Day::first())
    }
}

impl TryFrom<&str> for YearMonthDay {
    type Error = Error;

    fn try_from(ymdish: &str) -> Result<Self> {
        if ymdish == "*" {
            Ok(YearMonthDay(
                Year::default(),
                Month::default(),
                Day::default(),
            ))
        } else {
            let ymd_split = ymdish.split(',').collect::<Vec<&str>>();

            if ymd_split.len() == 3 {
                let year = ymd_split[0].parse::<Year>()?;
                let month = ymd_split[1].parse::<Month>()?;
                let day = ymd_split[2].parse::<Day>()?;
                Ok(YearMonthDay(year, month, day))
            } else {
                Err(InvalidDate(ymdish.to_string()).into())
            }
        }
    }
}

impl FromStr for YearMonthDay {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        YearMonthDay::try_from(s)
    }
}

impl Display for YearMonthDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
pub(crate) mod test {
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
        pub(crate) fn arb_ymd() (year in any::<i32>(), month in month_strategy(), day in day_strategy()) -> (String, i32, u8, u8) {
            let (month_str, month_val) = month;
            let (day_str, day_val) = day;
            let ymd_str = format!("{year},{month_str},{day_str}");
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
            let ymd = format!("{year}-{month}-{day}");
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

    #[test]
    fn monthly_works() {
        let ymd = YearMonthDay::monthly();
        assert_eq!(ymd.0, Year::default());
        assert_eq!(ymd.1, Month::default());
        assert_eq!(ymd.2, Day::first());
    }

    #[test]
    fn quarterly_works() {
        let ymd = YearMonthDay::quarterly();
        assert_eq!(ymd.0, Year::default());
        assert_eq!(ymd.1, Month::quarterly());
        assert_eq!(ymd.2, Day::first());
    }

    #[test]
    fn semiannually_works() {
        let ymd = YearMonthDay::semiannually();
        assert_eq!(ymd.0, Year::default());
        assert_eq!(ymd.1, Month::semiannually());
        assert_eq!(ymd.2, Day::first());
    }

    #[test]
    fn yearly_works() {
        let ymd = YearMonthDay::yearly();
        assert_eq!(ymd.0, Year::default());
        assert_eq!(ymd.1, Month::first());
        assert_eq!(ymd.2, Day::first());
    }
}
