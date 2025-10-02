// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::vec;

use anyhow::{Error, Result};
use bon::Builder;
use getset::Getters;
use rand::Rng;

use crate::{
    error::Error::InvalidTime,
    schedule::{All, parse_time_chunk},
};

const HOURS_PER_DAY: u8 = 24;
const MINUTES_PER_HOUR: u8 = 60;
const SECONDS_PER_MINUTE: u8 = 60;

/// The hour for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Hour {
    /// Every hour
    All,
    /// Specific hours
    Hours(Vec<u8>),
}

impl Hour {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Hour::All => true,
            Hour::Hours(hours) => hours.contains(&given),
        }
    }
}

impl Default for Hour {
    fn default() -> Self {
        Self::Hours(vec![0])
    }
}

impl All for Hour {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(0..HOURS_PER_DAY);
        Hour::Hours(vec![rand_in_range])
    }
}

impl From<Vec<u8>> for Hour {
    fn from(value: Vec<u8>) -> Self {
        Hour::Hours(value)
    }
}

impl From<u8> for Hour {
    fn from(value: u8) -> Self {
        Hour::Hours(vec![value])
    }
}

/// The minute for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Minute {
    /// Every minute
    All,
    /// Specific minutes
    Minutes(Vec<u8>),
}

impl Minute {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Minute::All => true,
            Minute::Minutes(minutes) => minutes.contains(&given),
        }
    }
}

impl Default for Minute {
    fn default() -> Self {
        Self::Minutes(vec![0])
    }
}

impl All for Minute {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(0..MINUTES_PER_HOUR);
        Minute::Minutes(vec![rand_in_range])
    }
}

impl From<Vec<u8>> for Minute {
    fn from(value: Vec<u8>) -> Self {
        Minute::Minutes(value)
    }
}

impl From<u8> for Minute {
    fn from(value: u8) -> Self {
        Minute::Minutes(vec![value])
    }
}

/// The seconds for a realtime schedule
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Second {
    /// Every second
    All,
    /// Specific seconds
    Seconds(Vec<u8>),
}

impl Second {
    pub(crate) fn matches(&self, given: u8) -> bool {
        match self {
            Second::All => true,
            Second::Seconds(seconds) => seconds.contains(&given),
        }
    }
}

impl Default for Second {
    fn default() -> Self {
        Self::Seconds(vec![0])
    }
}

impl All for Second {
    fn all() -> Self {
        Self::All
    }

    fn rand() -> Self {
        let mut rng = rand::rng();
        let rand_in_range = rng.random_range(0..60);
        Second::Seconds(vec![rand_in_range])
    }
}

impl From<Vec<u8>> for Second {
    fn from(value: Vec<u8>) -> Self {
        Second::Seconds(value)
    }
}

impl From<u8> for Second {
    fn from(value: u8) -> Self {
        Second::Seconds(vec![value])
    }
}

/// An hour, minute, and second combination
#[derive(Builder, Clone, Debug, Default, Eq, Getters, Hash, PartialEq)]
#[getset(get = "pub")]
pub struct HourMinuteSecond {
    /// The hour(s) to run
    #[builder(default = Hour::All, into)]
    hour: Hour,
    /// The minute(s) to run
    #[builder(default = Minute::All, into)]
    minute: Minute,
    /// The second(s) to run
    #[builder(default = Second::All, into)]
    second: Second,
}

impl HourMinuteSecond {
    /// A helper to create a daily schedule at midnight
    #[must_use]
    pub fn daily() -> Self {
        HourMinuteSecond::builder()
            .hour(0)
            .minute(0)
            .second(0)
            .build()
    }

    /// A helper to create an hourly schedule at the top of the hour
    #[must_use]
    pub fn hourly() -> Self {
        HourMinuteSecond::builder().minute(0).second(0).build()
    }

    /// A helper to create a minutely schedule at the top of the minute
    #[must_use]
    pub fn minutely() -> Self {
        HourMinuteSecond::builder().second(0).build()
    }
}

impl TryFrom<&str> for HourMinuteSecond {
    type Error = Error;

    fn try_from(hms: &str) -> Result<Self> {
        let hms_parts: Vec<&str> = hms.split(':').collect();
        if hms_parts.len() == 3 {
            let hour = parse_time_chunk::<Hour>(hms_parts[0], HOURS_PER_DAY, false)?;
            let minute = parse_time_chunk::<Minute>(hms_parts[1], MINUTES_PER_HOUR, false)?;
            let second = parse_time_chunk::<Second>(hms_parts[2], SECONDS_PER_MINUTE, false)?;
            Ok(HourMinuteSecond {
                hour,
                minute,
                second,
            })
        } else {
            Err(InvalidTime {
                time: hms.to_string(),
            }
            .into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        HOURS_PER_DAY, Hour, HourMinuteSecond, MINUTES_PER_HOUR, Minute, SECONDS_PER_MINUTE, Second,
    };
    use anyhow::{Result, anyhow};

    #[test]
    fn simple() -> Result<()> {
        let hms = HourMinuteSecond::try_from("10:00:00")?;
        assert_eq!(hms.hour, Hour::Hours(vec![10]));
        assert_eq!(hms.minute, Minute::Minutes(vec![0]));
        assert_eq!(hms.second, Second::Seconds(vec![0]));
        Ok(())
    }

    #[test]
    fn range() -> Result<()> {
        let hms = HourMinuteSecond::try_from("9..17:15..45:20..50")?;
        assert_eq!(hms.hour, Hour::Hours((9..=17).collect()));
        assert_eq!(hms.minute, Minute::Minutes((15..=45).collect()));
        assert_eq!(hms.second, Second::Seconds((20..=50).collect()));
        Ok(())
    }

    #[test]
    fn simple_repetition() -> Result<()> {
        let hms = HourMinuteSecond::try_from("0/2:0/3:0/4")?;
        assert_eq!(
            hms.hour,
            Hour::Hours((0..HOURS_PER_DAY).step_by(2).collect())
        );
        assert_eq!(
            hms.minute,
            Minute::Minutes((0..MINUTES_PER_HOUR).step_by(3).collect())
        );
        assert_eq!(
            hms.second,
            Second::Seconds((0..SECONDS_PER_MINUTE).step_by(4).collect())
        );
        Ok(())
    }

    #[test]
    fn range_repetition() -> Result<()> {
        let hms = HourMinuteSecond::try_from("9..17/2:12..44/4:20..50/4")?;
        assert_eq!(hms.hour, Hour::Hours((9..=17).step_by(2).collect()));
        assert_eq!(hms.minute, Minute::Minutes((12..=44).step_by(4).collect()));
        assert_eq!(hms.second, Second::Seconds((20..=50).step_by(4).collect()));
        Ok(())
    }

    #[test]
    fn random() -> Result<()> {
        let hms = HourMinuteSecond::try_from("R:R:R")?;

        if let Hour::Hours(vals) = hms.hour {
            assert_eq!(vals.len(), 1);
            assert!(vals[0] < 24);
        } else {
            return Err(anyhow!("This isn't the correct kind of hour"));
        }
        if let Minute::Minutes(vals) = hms.minute {
            assert_eq!(vals.len(), 1);
            assert!(vals[0] < 60);
        } else {
            return Err(anyhow!("This isn't the correct kind of minute"));
        }
        if let Second::Seconds(vals) = hms.second {
            assert_eq!(vals.len(), 1);
            assert!(vals[0] < 60);
        } else {
            return Err(anyhow!("This isn't the correct kind of second"));
        }
        Ok(())
    }

    #[test]
    fn invalid_hour_range() -> Result<()> {
        match HourMinuteSecond::try_from("17..9:00:00") {
            Ok(_) => Err(anyhow!("this time should be invalid")),
            Err(e) => {
                assert_eq!(format!("{e}"), "invalid range: '17..9'");
                Ok(())
            }
        }
    }
}
