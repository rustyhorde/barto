// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod cv;
pub(crate) mod dow;
pub(crate) mod hms;
pub(crate) mod ymd;

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::{Error, Result};
use bon::Builder;
#[cfg(test)]
use getset::{Getters, Setters};
use num_traits::FromPrimitive as _;
use time::OffsetDateTime;

use crate::{
    error::Error::InvalidCalendar,
    realtime::{
        cv::ConstrainedValueMatcher as _,
        hms::{
            HourMinuteSecond,
            hour::{Hour, HourOfDay},
            minute::{Minute, MinuteOfHour},
            second::{Second, SecondOfMinute},
        },
        ymd::{
            day::{Day, DayOfMonth},
            month::{Month, MonthOfYear},
            year::Year,
        },
    },
};

use self::{dow::Dow, ymd::YearMonthDay};

const MINUTELY: &str = "minutely";
const HOURLY: &str = "hourly";
const DAILY: &str = "daily";
const WEEKLY: &str = "weekly";
const MONTHLY: &str = "monthly";
const QUARTERLY: &str = "quarterly";
const SEMIANNUALLY: &str = "semiannually";
const YEARLY: &str = "yearly";

/// A realtime schedule definition
#[derive(Builder, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Getters, Setters))]
pub struct Realtime {
    #[builder(default)]
    day_of_week: Dow,
    #[builder(default)]
    year: Year,
    #[builder(default)]
    #[cfg_attr(test, getset(set))]
    month: Month,
    #[builder(default)]
    day: Day,
    #[builder(default)]
    hour: Hour,
    #[builder(default)]
    minute: Minute,
    #[builder(default)]
    second: Second,
}

impl Realtime {
    /// Should this schedule run at this time
    #[must_use]
    pub fn is_now(&self, now: OffsetDateTime) -> bool {
        let dow_match = match &self.day_of_week.0 {
            Some(dows) => dows.contains(&now.weekday().number_days_from_sunday()),
            None => true,
        };
        let year_match = self.year.matches(now.year());
        let month_match =
            MonthOfYear::from_u8(now.month().into()).is_some_and(|month| self.month.matches(month));
        let day_match = DayOfMonth::from_u8(now.day()).is_some_and(|day| self.day.matches(day));
        let hour_match = HourOfDay::from_u8(now.hour()).is_some_and(|hour| self.hour.matches(hour));
        let minute_match =
            MinuteOfHour::from_u8(now.minute()).is_some_and(|minute| self.minute.matches(minute));
        let second_match =
            SecondOfMinute::from_u8(now.second()).is_some_and(|second| self.second.matches(second));

        dow_match
            && year_match
            && month_match
            && day_match
            && hour_match
            && minute_match
            && second_match
    }
}

impl TryFrom<&str> for Realtime {
    type Error = Error;

    fn try_from(calendar: &str) -> Result<Self> {
        let parts: Vec<&str> = calendar.split_whitespace().collect();

        let (day_of_week, date, hms) = if parts.len() == 3 {
            // has day of week
            (parts[0], parts[1], parts[2])
        } else if parts.len() == 2 {
            // no day of week
            ("*", parts[0], parts[1])
        } else if parts.len() == 1 {
            // no day of week, or date
            if parts[0] == MINUTELY {
                let (hour, minute, second) = HourMinuteSecond::minutely().take();
                return Ok(Realtime::builder()
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == HOURLY {
                let (hour, minute, second) = HourMinuteSecond::hourly().take();
                return Ok(Realtime::builder()
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == DAILY {
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == WEEKLY {
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .day_of_week(Dow::monday())
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == MONTHLY {
                let (year, month, day) = YearMonthDay::monthly().take();
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .year(year)
                    .month(month)
                    .day(day)
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == QUARTERLY {
                let (year, month, day) = YearMonthDay::quarterly().take();
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .year(year)
                    .month(month)
                    .day(day)
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == SEMIANNUALLY {
                let (year, month, day) = YearMonthDay::semiannually().take();
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .year(year)
                    .month(month)
                    .day(day)
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            } else if parts[0] == YEARLY {
                let (year, month, day) = YearMonthDay::yearly().take();
                let (hour, minute, second) = HourMinuteSecond::daily().take();
                return Ok(Realtime::builder()
                    .year(year)
                    .month(month)
                    .day(day)
                    .hour(hour)
                    .minute(minute)
                    .second(second)
                    .build());
            }
            ("*", "*", parts[0])
        } else {
            return Err(InvalidCalendar(calendar.to_string()).into());
        };

        let day_of_week = day_of_week.parse::<Dow>()?;
        let ymd = date.parse::<YearMonthDay>()?;
        let (year, month, day) = ymd.clone().take();
        let hms = hms.parse::<HourMinuteSecond>()?;
        let (hour, minute, second) = hms.clone().take();

        let rt = Realtime {
            day_of_week,
            year,
            month,
            day,
            hour,
            minute,
            second,
        };
        Ok(rt)
    }
}

impl FromStr for Realtime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Realtime::try_from(s)
    }
}

impl Display for Realtime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ymd = YearMonthDay(self.year.clone(), self.month.clone(), self.day.clone());
        let hms = HourMinuteSecond(self.hour.clone(), self.minute.clone(), self.second.clone());
        write!(f, "{} {} {}", self.day_of_week, ymd, hms)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::proptest, prop_compose};
    use time::{OffsetDateTime, macros::date};

    use crate::realtime::{dow::test::arb_dow, hms::test::arb_hms, ymd::test::arb_ymd};

    use super::Realtime;

    prop_compose! {
        fn arb_realtime() (dow in arb_dow(), ymd in arb_ymd(), hms in arb_hms()) -> String {
            let (dow_str, _dow_val) = dow;
            let (ymd_str, _year_val, _month_val, _day_val) = ymd;
            let (hms_str, _hour_val, _minute_val, _second_val) = hms;
            format!("{dow_str} {ymd_str} {hms_str}")
        }
    }

    prop_compose! {
        fn arb_partial_realtime() (ymd in arb_ymd(), hms in arb_hms()) -> String {
            let (ymd_str, _year_val, _month_val, _day_val) = ymd;
            let (hms_str, _hour_val, _minute_val, _second_val) = hms;
            format!("{ymd_str} {hms_str}")
        }
    }

    // Valid inputs
    proptest! {
        #[test]
        fn arb_realtime_works(s in arb_realtime()) {
            assert!(Realtime::try_from(s.as_str()).is_ok());
        }

        #[test]
        fn arb_partial_realtime_works(s in arb_partial_realtime()) {
            assert!(Realtime::try_from(s.as_str()).is_ok());
        }

        #[test]
        fn arb_realtime_display_works(s in arb_realtime()) {
            let rt_res = Realtime::try_from(s.as_str());
            assert!(rt_res.is_ok());
            let rt = rt_res.unwrap();
            let rt_str = rt.to_string();
            assert!(!rt_str.is_empty());
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Realtime::try_from("").is_err());
    }

    #[test]
    fn minutely_works() {
        let re_res = Realtime::try_from("minutely");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let new_now_res = now.replace_second(0);
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn hourly_works() {
        let re_res = Realtime::try_from("hourly");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let new_now_res = now.replace_minute(0).and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn daily_works() {
        let re_res = Realtime::try_from("daily");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn weekly_works() {
        let re_res = Realtime::try_from("weekly");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let now = now.replace_date(date!(2025 - 10 - 20));
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn monthly_works() {
        let re_res = Realtime::try_from("monthly");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let now = now.replace_date(date!(2025 - 10 - 1));
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn quarterly_works() {
        let re_res = Realtime::try_from("quarterly");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let now = now.replace_date(date!(2025 - 10 - 1));
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn semiannually_works() {
        let re_res = Realtime::try_from("semiannually");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let now = now.replace_date(date!(2025 - 07 - 1));
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn yearly_works() {
        let re_res = Realtime::try_from("yearly");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let now = now.replace_date(date!(2025 - 01 - 01));
        let new_now_res = now
            .replace_hour(0)
            .and_then(|dt| dt.replace_minute(0))
            .and_then(|dt| dt.replace_second(0));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn only_hms_works() {
        let re_res = Realtime::try_from("12:30:15");
        assert!(re_res.is_ok());
        let rt = re_res.unwrap();
        let now = OffsetDateTime::now_utc();
        let new_now_res = now
            .replace_hour(12)
            .and_then(|dt| dt.replace_minute(30))
            .and_then(|dt| dt.replace_second(15));
        assert!(new_now_res.is_ok());
        let new_now = new_now_res.unwrap();
        assert!(rt.is_now(new_now));
    }

    #[test]
    fn random_works() {
        let re_res = Realtime::try_from("*,*,* 10:R:R");
        assert!(re_res.is_ok());
        let re_res = "*,*,* 0/2:R:R".parse::<Realtime>();
        assert!(re_res.is_ok());
    }
}
