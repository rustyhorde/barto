// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use regex::Regex;

use crate::{
    error::Error::InvalidTime,
    realtime::hms::{hour::Hour, minute::Minute, second::Second},
};

pub(crate) mod hour;
pub(crate) mod minute;
pub(crate) mod second;

static HMS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{1,2}):(\d{1,2}):(\d{1,2})$").expect("invalid HMS regex"));

pub(crate) type HourMinuteSecondTuple = (Hour, Minute, Second);

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HourMinuteSecond(pub(crate) Hour, pub(crate) Minute, pub(crate) Second);

impl HourMinuteSecond {
    pub(crate) fn take(self) -> HourMinuteSecondTuple {
        (self.0, self.1, self.2)
    }

    pub(crate) fn minutely() -> Self {
        HourMinuteSecond(Hour::default(), Minute::default(), Second::zero())
    }

    pub(crate) fn hourly() -> Self {
        HourMinuteSecond(Hour::default(), Minute::zero(), Second::zero())
    }

    pub(crate) fn daily() -> Self {
        HourMinuteSecond(Hour::zero(), Minute::zero(), Second::zero())
    }
}

impl TryFrom<&str> for HourMinuteSecond {
    type Error = Error;

    fn try_from(hms: &str) -> Result<Self> {
        if let Some(caps) = HMS_RE.captures(hms) {
            let hour = caps[1].parse::<Hour>()?;
            let minute = caps[2].parse::<Minute>()?;
            let second = caps[3].parse::<Second>()?;
            Ok(HourMinuteSecond(hour, minute, second))
        } else {
            Err(InvalidTime(hms.to_string()).into())
        }
    }
}

impl FromStr for HourMinuteSecond {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        HourMinuteSecond::try_from(s)
    }
}

impl Display for HourMinuteSecond {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use proptest::{prelude::proptest, prop_assume, prop_compose};

    use crate::realtime::{
        cv::ConstrainedValueParser as _,
        hms::{
            HourMinuteSecond,
            hour::{
                HOUR_RANGE_RE, HOUR_REPETITION_RE, Hour,
                test::{VALID_HOUR_RE, hour_strategy},
            },
            minute::{
                MINUTE_RANGE_RE, MINUTE_REPETITION_RE, Minute,
                test::{VALID_MINUTE_RE, minute_strategy},
            },
            second::{
                SECOND_RANGE_RE, SECOND_REPETITION_RE, Second,
                test::{VALID_SECOND_RE, second_strategy},
            },
        },
    };

    // Valid strategies
    prop_compose! {
        pub(crate) fn arb_hms() (hour in hour_strategy(), minute in minute_strategy(), second in second_strategy()) -> (String, u8, u8, u8) {
            let (hour_str, hour_val) = hour;
            let (minute_str, minute_val) = minute;
            let (second_str, second_val) = second;
            let hms_str = format!("{hour_str}:{minute_str}:{second_str}");
            (hms_str, hour_val, minute_val, second_val)
        }
    }

    // Valid inputs
    proptest! {
        #[test]
        fn arb_hms_works(s in arb_hms()) {
            let (hms_str, _, _, _) = s;
            assert!(HourMinuteSecond::try_from(hms_str.as_str()).is_ok());
        }
    }

    // Invalid inputs
    proptest! {
        #[test]
        fn random_input_errors(hour in "\\PC*", minute in "\\PC*", second in "\\PC*") {
            prop_assume!(!VALID_HOUR_RE.is_match(hour.as_str()));
            prop_assume!(!HOUR_REPETITION_RE.is_match(hour.as_str()));
            prop_assume!(!HOUR_RANGE_RE.is_match(hour.as_str()));
            prop_assume!(hour.as_str() != "*");
            prop_assume!(!VALID_MINUTE_RE.is_match(minute.as_str()));
            prop_assume!(!MINUTE_REPETITION_RE.is_match(minute.as_str()));
            prop_assume!(!MINUTE_RANGE_RE.is_match(minute.as_str()));
            prop_assume!(minute.as_str() != "*");
            prop_assume!(minute.as_str() != "R");
            prop_assume!(!VALID_SECOND_RE.is_match(second.as_str()));
            prop_assume!(!SECOND_REPETITION_RE.is_match(second.as_str()));
            prop_assume!(!SECOND_RANGE_RE.is_match(second.as_str()));
            prop_assume!(second.as_str() != "*");
            prop_assume!(second.as_str() != "R");
            let hms = format!("{hour}:{minute}:{second}");
            assert!(HourMinuteSecond::try_from(hms.as_str()).is_err());
            assert!(hms.as_str().parse::<HourMinuteSecond>().is_err());
        }
    }

    #[test]
    fn take_works() {
        let hms = HourMinuteSecond(Hour::all(), Minute::all(), Second::all());
        let (hour, minute, second) = hms.take();
        assert_eq!(hour, Hour::all());
        assert_eq!(minute, Minute::all());
        assert_eq!(second, Second::all());
    }

    #[test]
    fn minutely_works() {
        let hms = HourMinuteSecond::minutely();
        let (hour, minute, second) = hms.take();
        assert_eq!(hour, Hour::default());
        assert_eq!(minute, Minute::default());
        assert_eq!(second, Second::zero());
    }

    #[test]
    fn hourly_works() {
        let hms = HourMinuteSecond::hourly();
        let (hour, minute, second) = hms.take();
        assert_eq!(hour, Hour::default());
        assert_eq!(minute, Minute::zero());
        assert_eq!(second, Second::zero());
    }

    #[test]
    fn daily_works() {
        let hms = HourMinuteSecond::daily();
        let (hour, minute, second) = hms.take();
        assert_eq!(hour, Hour::zero());
        assert_eq!(minute, Minute::zero());
        assert_eq!(second, Second::zero());
    }
}
