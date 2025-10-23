// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{str::FromStr, sync::LazyLock};

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

pub(crate) struct HourMinuteSecond(pub(crate) Hour, pub(crate) Minute, pub(crate) Second);

impl HourMinuteSecond {
    pub(crate) fn take(self) -> HourMinuteSecondTuple {
        (self.0, self.1, self.2)
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

#[cfg(test)]
mod test {
    use proptest::{prelude::proptest, prop_compose};

    use crate::realtime::hms::{
        HourMinuteSecond, hour::test::hour_strategy, minute::test::minute_strategy,
        second::test::second_strategy,
    };

    // Valid strategies
    prop_compose! {
        fn arb_hms() (hour in hour_strategy(), minute in minute_strategy(), second in second_strategy()) -> (String, u8, u8, u8) {
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
}
