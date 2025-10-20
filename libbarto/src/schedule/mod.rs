// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod dow;
pub(crate) mod hms;
pub(crate) mod ymd;

use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    sync::LazyLock,
};

use anyhow::{Error, Result};
use bon::Builder;
use regex::Regex;
use time::OffsetDateTime;

use crate::{
    error::Error::{
        InvalidCalendar, InvalidFirstCapture, InvalidRange, InvalidSecondCapture, InvalidTime,
        NoValidCaptures,
    },
    schedule::ymd::YearMonthDay,
    utils::until_err,
};

use self::{dow::DayOfWeek, hms::HourMinuteSecond};

static RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d{1,2})\.\.(\d{1,2})").expect("invalid range regex"));
static REP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{1,2})(\.\.(\d{1,2}))?/(\d{1,2})").expect("invalid repetition regex")
});

const MINUTELY: &str = "minutely";
const HOURLY: &str = "hourly";
const DAILY: &str = "daily";
const WEEKLY: &str = "weekly";
const MONTHLY: &str = "monthly";
const QUARTERLY: &str = "quarterly";
const SEMIANNUALLY: &str = "semiannually";
const YEARLY: &str = "yearly";

trait All {
    fn all() -> Self;
    fn rand() -> Self;
}

// pub(crate) trait Blah<T> {
//     fn min(self: &Self) -> T;
//     fn max(self: &Self) -> T;
//     fn one_based(self: &Self) -> bool;
// }
// #[derive(Builder)]
// struct ConstrainedValue<T> {
//     min: T,
//     max: T,
//     one_based: bool,
//     #[builder(default)]
//     values: Vec<T>,
// }

// impl ConstrainedValue<u8> {
//     fn all(&mut self) {
//         self.values = (self.min..=self.max).collect();
//     }

//     fn rand(&mut self) {
//         let rand_in_range = rng().random_range(self.min..=self.max);
//         self.values = vec![rand_in_range]
//     }

//     pub(crate) fn parse(&mut self, cvish: &str) -> Result<()> {
//         if cvish == "*" {
//             Ok(self.all())
//         } else if cvish == "R" {
//             Ok(self.rand())
//         } else {
//             Err(anyhow::anyhow!("not implemented"))
//         }
//     }
// }

/// A realtime schedule defines the times at which a task should run.
///
/// A realtime schedule is made up of three components:
/// ```text
/// |day of the week| |year-month-day| |hour:minute:second|
///
/// |day of the week| is optional and defaults to every day ('*') if not specified.
/// |year-month-day| is optional and defaults to every year, month, and day ('*-*-*') if not specified.
/// ```
///
/// # Day of the Week EBNF
/// ```text
/// day_of_week     = "*" | day_list | "" ;
/// day_list        = day , {", " , day} ;
/// day             = day_short | day_full | day_range_short | day_range_full ;
/// day_short       = "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun" ;
/// day_full        = "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" | "Saturday" | "Sunday" ;
/// day_range_short = day_short , ".." , day_short ;
/// day_range_full  = day_full , ".." , day_full ;
/// ```
///
/// ## Day of the Week Examples
/// ```text
/// '*'                  - every day of the week
/// 'Sun'                - every Sunday
/// 'Mon,Wed,Fri'        - every Monday, Wednesday, and Friday
/// 'Mon..Fri'           - every weekday (Monday through Friday)
/// 'Sun, Mon..Wed, Sat' - every Sunday, Monday through Wednesday, and Saturday
/// 'Sun, Sun..Fri, Fri' - every Sunday, Sunday through Friday, and Friday (ranges and duplicates allowed, but not recommended)
///
/// # Year-Month-Day EBNF
/// ```text
/// random           = "R" ;
/// non_zero_digit   = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
/// digit            = "0" | non_zero_digit ;
/// year             = digit , digit , digit , digit, { digit } ;
/// month            = non_zero_digit , [ digit ] ;
/// day              = non_zero_digit , [ digit ] ;
/// year_range       = year , ".." , year ;
/// month_range      = month , ".." , month ;
/// day_range        = day , ".." , day ;
/// year_repitition  = year , [ ".." , year ] , "/" , non_zero_digit , { digit } ;
/// month_repitition = month , [ ".." , month ] , "/" , non_zero_digit , [ digit ] ;
/// day_repitition   = day , [ ".." , day ] , "/" , non_zero_digit , [ digit ] ;
/// year_format      = year | "*" | random | year_range | year_repitition ;
/// month_format     = month | "*" | random | month_range | month_repitition ;
/// day_format       = day | "*" | random | day_range | day_repitition ;
/// year_month_day   = year_format , "-" , month_format , "-" , day_format ;
///
/// ## Year-Month-Day Examples
/// ```text
/// '*-*-*' - every year, month, and day
/// "*-R-R" - every year, random month (1 to 12), and random day (1 to 28)"
/// "*-*-1..15" - every year, first 15 days of every month
/// ```
///
#[derive(Builder, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Realtime {
    /// The day(s) of the week to run
    #[builder(default = DayOfWeek::All, into)]
    day_of_week: DayOfWeek,
    /// The year(s), month(s), and day(s) to run
    #[builder(default)]
    ymd: YearMonthDay,
    /// The hour(s), minute(s), and second(s) to run
    #[builder(default, into)]
    hms: HourMinuteSecond,
}

impl Realtime {
    /// Should this schedule run at this time
    #[must_use]
    pub fn should_run(&self, now: OffsetDateTime) -> bool {
        self.day_of_week.matches(now.weekday())
            && self.ymd.year().matches(now.year())
            && self.ymd.month().matches(now.month().into())
            && self.ymd.day().matches(now.day())
            && self.hms.hour().matches(now.hour())
            && self.hms.minute().matches(now.minute())
            && self.hms.second().matches(now.second())
    }
}

impl Default for Realtime {
    fn default() -> Self {
        Self {
            day_of_week: DayOfWeek::All,
            ymd: YearMonthDay::default(),
            hms: HourMinuteSecond::default(),
        }
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
                return Ok(Realtime::builder()
                    .hms(HourMinuteSecond::minutely())
                    .build());
            } else if parts[0] == HOURLY {
                return Ok(Realtime::builder().hms(HourMinuteSecond::hourly()).build());
            } else if parts[0] == DAILY {
                return Ok(Realtime::builder().hms(HourMinuteSecond::daily()).build());
            } else if parts[0] == WEEKLY {
                return Ok(Realtime::builder()
                    .day_of_week(1)
                    .hms(HourMinuteSecond::daily())
                    .build());
            } else if parts[0] == MONTHLY {
                return Ok(Realtime::builder()
                    .ymd(YearMonthDay::monthly())
                    .hms(HourMinuteSecond::daily())
                    .build());
            } else if parts[0] == QUARTERLY {
                return Ok(Realtime::builder()
                    .ymd(YearMonthDay::quarterly())
                    .hms(HourMinuteSecond::daily())
                    .build());
            } else if parts[0] == SEMIANNUALLY {
                return Ok(Realtime::builder()
                    .ymd(YearMonthDay::semiannually())
                    .hms(HourMinuteSecond::daily())
                    .build());
            } else if parts[0] == YEARLY {
                return Ok(Realtime::builder()
                    .ymd(YearMonthDay::yearly())
                    .hms(HourMinuteSecond::daily())
                    .build());
            }
            ("*", "*", parts[0])
        } else {
            return Err(InvalidCalendar {
                calendar: calendar.to_string(),
            }
            .into());
        };

        let dow: DayOfWeek = day_of_week.try_into()?;
        let ymd: YearMonthDay = date.try_into()?;
        let hms: HourMinuteSecond = hms.try_into()?;
        Ok(Realtime::builder()
            .day_of_week(dow)
            .ymd(ymd)
            .hms(hms)
            .build())
    }
}

impl Display for Realtime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.day_of_week, self.ymd, self.hms)
    }
}

fn parse_time_chunk<T>(part: &str, max: u8, one_based: bool) -> Result<T>
where
    T: All + TryFrom<Vec<u8>, Error = Error>,
{
    if part == "*" {
        Ok(T::all())
    } else if part == "R" {
        Ok(T::rand())
    } else {
        let mut err = Ok(());
        let prrv_fn = |hour: &str| -> Result<Vec<u8>> { parse_rep_range_val(hour, max, one_based) };
        let mut time_v: Vec<u8> = part
            .split(',')
            .map(prrv_fn)
            .scan(&mut err, until_err)
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        err?;
        time_v.sort_unstable();
        T::try_from(time_v)
    }
}

fn parse_rep_range_val(val: &str, max: u8, one_based: bool) -> Result<Vec<u8>> {
    if REP_RE.is_match(val) {
        parse_repetition(val, max)
    } else if RANGE_RE.is_match(val) {
        parse_range(val, max, one_based)
    } else {
        parse_value(val)
    }
}

fn parse_range(range: &str, max: u8, one_based: bool) -> Result<Vec<u8>> {
    let caps = RANGE_RE.captures(range).ok_or(NoValidCaptures)?;
    let first = caps
        .get(1)
        .ok_or(InvalidFirstCapture)?
        .as_str()
        .parse::<u8>()?;
    let second = caps
        .get(2)
        .ok_or(InvalidSecondCapture)?
        .as_str()
        .parse::<u8>()?;
    if second < first
        || (one_based && first == 0)
        || ((one_based && second > max) || (!one_based && second >= max))
    {
        Err(InvalidRange(range.to_string()).into())
    } else {
        Ok((first..=second).collect())
    }
}

fn parse_repetition(rep: &str, max: u8) -> Result<Vec<u8>> {
    let caps = REP_RE.captures(rep).ok_or(NoValidCaptures)?;

    if caps.len() == 5 {
        let start = caps
            .get(1)
            .ok_or(InvalidFirstCapture)?
            .as_str()
            .parse::<u8>()?;
        let rep = caps
            .get(4)
            .ok_or(InvalidSecondCapture)?
            .as_str()
            .parse::<usize>()?;
        if let Some(end) = caps.get(3) {
            let end = end.as_str().parse::<u8>()?;
            if end < start || end >= max {
                Err(InvalidRange(format!("{start}..{end}")).into())
            } else {
                Ok((start..=end).step_by(rep).collect())
            }
        } else {
            Ok((start..max).step_by(rep).collect())
        }
    } else {
        Err(InvalidTime(rep.to_string()).into())
    }
}

fn parse_value(value: &str) -> Result<Vec<u8>> {
    Ok(vec![value.parse::<u8>()?])
}

#[cfg(test)]
mod test {
    use crate::{
        DayOfWeek, HourMinuteSecond, Second, YearMonthDay,
        schedule::{
            MONTHLY, QUARTERLY, SEMIANNUALLY, WEEKLY, YEARLY, hms::hour::Hour, hms::minute::Minute,
        },
    };

    use super::{DAILY, HOURLY, MINUTELY, Realtime};
    use anyhow::{Result, anyhow};
    use time::OffsetDateTime;

    #[test]
    fn invalid_calendar() -> Result<()> {
        match Realtime::try_from("this is a bad calendar") {
            Ok(_) => Err(anyhow!("this should be a bad calendar")),
            Err(e) => {
                assert_eq!(
                    format!("{e}"),
                    "invalid calendar string: 'this is a bad calendar'"
                );
                Ok(())
            }
        }
    }

    #[test]
    fn minutely() -> Result<()> {
        let res: Realtime = MINUTELY.try_into()?;
        let expected = Realtime::builder()
            .hms(HourMinuteSecond::minutely())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn hourly() -> Result<()> {
        let res: Realtime = HOURLY.try_into()?;
        let expected = Realtime::builder().hms(HourMinuteSecond::hourly()).build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn daily() -> Result<()> {
        let res: Realtime = DAILY.try_into()?;
        let expected = Realtime::builder().hms(HourMinuteSecond::daily()).build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn weekly() -> Result<()> {
        let res: Realtime = WEEKLY.try_into()?;
        let expected = Realtime::builder()
            .day_of_week(1)
            .hms(HourMinuteSecond::daily())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn monthly() -> Result<()> {
        let res: Realtime = MONTHLY.try_into()?;
        let expected = Realtime::builder()
            .ymd(YearMonthDay::monthly())
            .hms(HourMinuteSecond::daily())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn quarterly() -> Result<()> {
        let res: Realtime = QUARTERLY.try_into()?;
        let expected = Realtime::builder()
            .ymd(YearMonthDay::quarterly())
            .hms(HourMinuteSecond::daily())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn semiannually() -> Result<()> {
        let res: Realtime = SEMIANNUALLY.try_into()?;
        let expected = Realtime::builder()
            .ymd(YearMonthDay::semiannually())
            .hms(HourMinuteSecond::daily())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn yearly() -> Result<()> {
        let res: Realtime = YEARLY.try_into()?;
        let expected = Realtime::builder()
            .ymd(YearMonthDay::yearly())
            .hms(HourMinuteSecond::daily())
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn no_day_of_week() -> Result<()> {
        let res: Realtime = "*-*-* 3:00:00".try_into()?;
        let expected = Realtime::builder()
            .hms(
                HourMinuteSecond::builder()
                    .hour(Hour::try_from(3)?)
                    .minute(Minute::top_of_hour())
                    .second(Second::top_of_minute())
                    .build(),
            )
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn random() -> Result<()> {
        let res: Realtime = "*-*-* 0/2:R:R".try_into()?;
        assert!(res.day_of_week == DayOfWeek::All);
        match res.hms.hour() {
            Hour::All => return Err(anyhow!("hour should not be all")),
            Hour::Hours(items) => {
                assert_eq!(items, &(0..24).step_by(2).collect::<Vec<u8>>());
            }
        }
        match res.hms.minute() {
            Minute::All => return Err(anyhow!("minute should not be all")),
            Minute::Minutes(items) => {
                assert_eq!(items.len(), 1);
                assert!(items[0] < 60);
            }
        }
        match res.hms.second() {
            Second::All => return Err(anyhow!("second should not be all")),
            Second::Seconds(items) => {
                assert_eq!(items.len(), 1);
                assert!(items[0] < 60);
            }
        }
        Ok(())
    }

    #[test]
    fn full_calendar() -> Result<()> {
        let res: Realtime = "Mon..Fri *-*-* 3:22:17".try_into()?;
        let expected = Realtime::builder()
            .day_of_week((1..=5).collect::<Vec<u8>>())
            .hms(
                HourMinuteSecond::builder()
                    .hour(Hour::try_from(3)?)
                    .minute(Minute::try_from(22)?)
                    .second(Second::try_from(17)?)
                    .build(),
            )
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn funky() -> Result<()> {
        let res: Realtime = "Mon..Thu,Sun,Sat *-*-* 3..7,10,0,14..18/2:22:17".try_into()?;
        let expected = Realtime::builder()
            .day_of_week(vec![0, 1, 2, 3, 4, 6])
            .hms(
                HourMinuteSecond::builder()
                    .hour(Hour::try_from(vec![0, 3, 4, 5, 6, 7, 10, 14, 16, 18])?)
                    .minute(Minute::try_from(22)?)
                    .second(Second::try_from(17)?)
                    .build(),
            )
            .build();
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn invalid_date() -> Result<()> {
        match TryInto::<Realtime>::try_into("*-* 3:11:17") {
            Ok(_) => Err(anyhow!("this should be a bad calendar")),
            Err(e) => {
                assert_eq!(format!("{e}"), "invalid date string: '*-*'");
                Ok(())
            }
        }
    }

    #[test]
    fn invalid_time() -> Result<()> {
        match TryInto::<Realtime>::try_into("*-*-* 12:00") {
            Ok(_) => Err(anyhow!("this should be a bad calendar")),
            Err(e) => {
                assert_eq!(format!("{e}"), "invalid time string: '12:00'");
                Ok(())
            }
        }
    }

    #[test]
    fn should_run() -> Result<()> {
        let hms = HourMinuteSecond::builder()
            .hour(Hour::try_from(4)?)
            .minute(Minute::try_from(37)?)
            .second(Second::try_from(0)?)
            .build();
        let rt = Realtime::builder().hms(hms).build();
        let odt = OffsetDateTime::now_utc();
        let odt = odt.replace_year(2024)?;
        let odt = if odt.day() > 28 {
            odt.replace_day(28)?
        } else {
            odt
        };
        let odt = odt.replace_month(time::Month::February)?;
        let odt = odt.replace_hour(4)?;
        let odt = odt.replace_minute(37)?;
        let odt = odt.replace_second(0)?;
        assert!(rt.should_run(odt));
        Ok(())
    }
}
