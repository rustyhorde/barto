// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod dow;

use anyhow::{Error, Result};
use time::OffsetDateTime;

use crate::error::Error::InvalidCalendar;

use self::dow::Dow;

/// A realtime schedule definition
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RealtimeNew {
    day_of_week: Option<Vec<u8>>,
    year: Option<Vec<i32>>,
    month: Option<Vec<u8>>,
    day: Option<Vec<u8>>,
    hour: Option<Vec<u8>>,
    minute: Option<Vec<u8>>,
    second: Option<Vec<u8>>,
}

impl RealtimeNew {
    /// Should this schedule run at this time
    #[must_use]
    pub fn is_now(&self, now: OffsetDateTime) -> bool {
        let dow_match = match &self.day_of_week {
            Some(dows) => dows.contains(&now.weekday().number_days_from_sunday()),
            None => true,
        };
        let year_match = match &self.year {
            Some(years) => years.contains(&now.year()),
            None => true,
        };
        let month_match = match &self.month {
            Some(months) => months.contains(&(now.month().into())),
            None => true,
        };
        let day_match = match &self.day {
            Some(days) => days.contains(&now.day()),
            None => true,
        };
        let hour_match = match &self.hour {
            Some(hours) => hours.contains(&now.hour()),
            None => true,
        };
        let minute_match = match &self.minute {
            Some(minutes) => minutes.contains(&now.minute()),
            None => true,
        };
        let second_match = match &self.second {
            Some(seconds) => seconds.contains(&now.second()),
            None => true,
        };

        dow_match
            && year_match
            && month_match
            && day_match
            && hour_match
            && minute_match
            && second_match
    }
}

impl TryFrom<&str> for RealtimeNew {
    type Error = Error;

    fn try_from(calendar: &str) -> Result<Self> {
        let parts: Vec<&str> = calendar.split_whitespace().collect();

        let (day_of_week, _date, _hms) = if parts.len() == 3 {
            // has day of week
            (parts[0], parts[1], parts[2])
        } else if parts.len() == 2 {
            // no day of week
            ("*", parts[0], parts[1])
        } else if parts.len() == 1 {
            // no day of week, or date
            ("*", "*", parts[0])
        } else {
            return Err(InvalidCalendar(calendar.to_string()).into());
        };

        let day_of_week = day_of_week.parse::<Dow>()?.0;
        let rt = RealtimeNew {
            day_of_week,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
        };
        Ok(rt)
    }
}
