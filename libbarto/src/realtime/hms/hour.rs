// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use rand::{Rng as _, rng};
use regex::Regex;

use crate::{
    error::Error::InvalidHourOfDay,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

pub(crate) static HOUR_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.\.(\d+)$").expect("invalid hour range regex"));
pub(crate) static HOUR_REPETITION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)(\.\.(\d+))?/(\d+)$").expect("invalid repetition regex"));

pub(crate) type Hour = ConstrainedValue<HourOfDay>;

impl TryFrom<&str> for Hour {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Hour::parse(s)
    }
}

impl FromStr for Hour {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Hour::try_from(s)
    }
}

impl ConstrainedValueParser<'_, HourOfDay> for Hour {
    fn invalid(s: &str) -> Error {
        InvalidHourOfDay(s.to_string()).into()
    }

    fn allow_rand() -> bool {
        true
    }

    fn all() -> Self {
        Hour::All
    }

    fn rand() -> Self {
        let rand_hour =
            rng().random_range(u8::from(HourOfDay::min_value())..=u8::from(HourOfDay::max_value()));
        Hour::Specific(vec![HourOfDay(rand_hour)])
    }

    fn repetition_regex() -> Regex {
        HOUR_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        HOUR_RANGE_RE.clone()
    }

    fn rep(start: HourOfDay, end: Option<HourOfDay>, rep: u8) -> Self {
        Hour::Repetition { start, end, rep }
    }

    fn range(first: HourOfDay, second: HourOfDay) -> Self {
        Hour::Range(first, second)
    }

    fn specific(values: Vec<HourOfDay>) -> Self {
        Hour::Specific(values)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HourOfDay(pub(crate) u8);

impl Bounded for HourOfDay {
    fn min_value() -> Self {
        HourOfDay(0)
    }

    fn max_value() -> Self {
        HourOfDay(23)
    }
}

impl ToPrimitive for HourOfDay {
    fn to_i64(&self) -> Option<i64> {
        Some(<i64 as From<u8>>::from(self.0))
    }

    fn to_u64(&self) -> Option<u64> {
        Some(<u64 as From<u8>>::from(self.0))
    }
}

impl FromPrimitive for HourOfDay {
    fn from_i64(n: i64) -> Option<Self> {
        if (0..=23).contains(&n) {
            Some(HourOfDay(u8::try_from(n).ok()?))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        if (0..=23).contains(&n) {
            Some(HourOfDay(u8::try_from(n).ok()?))
        } else {
            None
        }
    }
}

impl Zero for HourOfDay {
    fn zero() -> Self {
        HourOfDay(0)
    }

    fn is_zero(&self) -> bool {
        *self == HourOfDay::zero()
    }
}

impl One for HourOfDay {
    fn one() -> Self {
        HourOfDay(1)
    }
}

impl Add for HourOfDay {
    type Output = HourOfDay;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else {
            let new = HourOfDay(self.0 + rhs.0);
            if new > HourOfDay::max_value() {
                panic!("HourOfDay addition overflowed");
            } else {
                new
            }
        }
    }
}

impl Sub for HourOfDay {
    type Output = HourOfDay;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.0.cmp(&self.0) {
            Ordering::Greater => panic!("HourOfDay subtraction underflowed"),
            Ordering::Equal => HourOfDay::zero(),
            Ordering::Less => HourOfDay(self.0 - rhs.0),
        }
    }
}

impl Mul for HourOfDay {
    type Output = HourOfDay;

    fn mul(self, _rhs: Self) -> Self::Output {
        panic!("HourOfDay multiplication is not supported");
    }
}

impl Div for HourOfDay {
    type Output = HourOfDay;

    fn div(self, _rhs: Self) -> Self::Output {
        panic!("HourOfDay division is not supported");
    }
}

impl Rem for HourOfDay {
    type Output = HourOfDay;

    fn rem(self, rhs: Self) -> Self::Output {
        HourOfDay(self.0 % rhs.0)
    }
}

impl FromStr for HourOfDay {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = s
            .parse::<u8>()
            .map_err(|_| InvalidHourOfDay(s.to_string()))?;
        if (0..=23).contains(&value) {
            Ok(HourOfDay(value))
        } else {
            Err(InvalidHourOfDay(s.to_string()).into())
        }
    }
}

impl From<HourOfDay> for u8 {
    fn from(hour: HourOfDay) -> u8 {
        hour.0
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::Ordering, fmt::Write as _, sync::LazyLock};

    use anyhow::Result;
    use num_traits::{Bounded as _, FromPrimitive as _, One as _, ToPrimitive as _, Zero as _};
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};
    use regex::Regex;

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::{HOUR_RANGE_RE, HOUR_REPETITION_RE, Hour, HourOfDay};

    pub(crate) static VALID_HOUR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\+?(0?|0?[1-9]|1[0-9]|2[0-3])$").unwrap());

    // Valid strategy generators
    prop_compose! {
        pub(crate) fn hour_strategy()(num in any::<u8>()) -> (String, u8) {
            let hour = num % 24;
            (hour.to_string(), hour)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in hour_strategy(), second in hour_strategy()) -> (String, u8, u8) {
            let (first_str, first_val) = first;
            let (second_str, second_val) = second;
            if first_val <= second_val {
                (format!("{first_str}..{second_str}"), first_val, second_val)
            } else {
                (format!("{second_str}..{first_str}"), second_val, first_val)
            }
        }
    }

    prop_compose! {
        fn arb_valid_repetition()(s in arb_valid_range(), rep in any::<u8>()) -> (String, u8, u8, u8) {
            let (mut prefix, min, max) = s;
            let rep = if rep == 0 { 1 } else { rep };
            write!(prefix, "/{rep}").unwrap();
            (prefix, min, max, rep)
        }
    }

    prop_compose! {
        fn arb_valid_repetition_no_end()(first in hour_strategy(), rep in any::<u8>()) -> String {
            let (mut first_str, _) = first;
            let rep = if rep == 0 { 1 } else { rep };
            write!(first_str, "/{rep}").unwrap();
            first_str
        }
    }

    // Invalid strategy generators
    prop_compose! {
        pub fn invalid_hour_strategy()(num in any::<u8>()) -> String {
            let hour = if num <= 23 {
                num + 24
            } else {
                num
            };
            hour.to_string()
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in hour_strategy(), second in hour_strategy()) -> String {
            let (_, first_val) = first;
            let (_, second_val) = second;

            let new_first = if first_val == second_val && first_val > 0 {
                first_val - 1
            } else if first_val == second_val && first_val == 0 {
                first_val + 1
            } else {
                first_val
            };
            match new_first.cmp(&second_val) {
                Ordering::Less | Ordering::Equal => format!("{second_val}..{new_first}"),
                Ordering::Greater => format!("{new_first}..{second_val}"),
            }
        }
    }

    prop_compose! {
        fn arb_invalid_repetition()(s in arb_invalid_range(), rep in any::<u8>()) -> String {
            let mut prefix = s;
            write!(prefix, "/{rep}").unwrap();
            prefix
        }
    }

    prop_compose! {
        fn arb_invalid_repetition_zero_rep()(s in arb_valid_range()) -> String {
            let (mut prefix, _, _) = s;
            write!(prefix, "/0").unwrap();
            prefix
        }
    }

    // Invalid input tests
    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_HOUR_RE.is_match(s.as_str()));
            prop_assume!(!HOUR_REPETITION_RE.is_match(s.as_str()));
            prop_assume!(!HOUR_RANGE_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Hour::try_from(s.as_str()).is_err());
            assert!(s.parse::<Hour>().is_err());
        }

        #[test]
        fn invalid_hour_errors(s in invalid_hour_strategy()) {
            let hour_res = Hour::try_from(s.as_str());
            assert!(hour_res.is_err());
            let hour_res = s.parse::<Hour>();
            assert!(hour_res.is_err());
        }

        #[test]
        fn arb_invalid_range_errors(s in arb_invalid_range()) {
            assert!(Hour::try_from(s.as_str()).is_err());
            assert!(s.parse::<Hour>().is_err());
        }

        #[test]
        fn arb_invalid_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Hour::try_from(s.as_str()).is_err());
            assert!(s.parse::<Hour>().is_err());
        }
    }

    // Valid input tests
    proptest! {
        #[test]
        fn arb_valid_hour(value in hour_strategy()) {
            let (hour_str, _) = value;
            let hour_res = Hour::try_from(hour_str.as_str());
            assert!(hour_res.is_ok());
            let hour_res = hour_str.parse::<Hour>();
            assert!(hour_res.is_ok());
        }

        #[test]
        fn arb_valid_hour_range(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Hour::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Hour>().is_ok());
        }

        #[test]
        fn arb_valid_hour_repetition(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Hour::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Hour>().is_ok());
        }

        #[test]
        fn arb_valid_hour_repetition_no_end(s in arb_valid_repetition_no_end()) {
            assert!(Hour::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Hour>().is_ok());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            prop_assume!(min != max);
            match Hour::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(cv_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    assert!(cv_range.matches(HourOfDay(in_range)), "hour {in_range} should match range '{range_str}'");
                    if min > u8::from(HourOfDay::min_value()) {
                        let below = rng().random_range(u8::from(HourOfDay::min_value())..min);
                        assert!(!cv_range.matches(HourOfDay(below)), "hour {below} should not match range '{range_str}'");
                    }
                    if max + 1 < u8::from(HourOfDay::max_value()) {
                        let above = rng().random_range((max + 1)..=u8::from(HourOfDay::max_value()));
                        assert!(!cv_range.matches(HourOfDay(above)), "hour {above} should not match range '{range_str}'");
                    }
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Hour::try_from("").is_err());
        assert!("".parse::<Hour>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Hour::All, Hour::try_from("*")?);
        assert_eq!(Hour::All, "*".parse::<Hour>()?);
        Ok(())
    }

    #[test]
    fn rand_works() {
        assert!(Hour::try_from("R").is_ok());
        assert!("R".parse::<Hour>().is_ok());
    }

    #[test]
    #[should_panic(expected = "HourOfDay addition overflowed")]
    fn add_panics_properly() {
        let hour1 = HourOfDay(28);
        let hour2: HourOfDay = HourOfDay(5);
        let _ = hour1 + hour2;
    }

    #[test]
    #[should_panic(expected = "HourOfDay subtraction underflowed")]
    fn sub_panics_properly() {
        let hour1 = HourOfDay(5);
        let hour2 = HourOfDay(8);
        let _ = hour1 - hour2;
    }

    #[test]
    #[should_panic(expected = "HourOfDay multiplication is not supported")]
    fn mul_panics_properly() {
        let hour1 = HourOfDay(5);
        let hour2 = HourOfDay(8);
        let _ = hour1 * hour2;
    }

    #[test]
    #[should_panic(expected = "HourOfDay division is not supported")]
    fn div_panics_properly() {
        let hour1 = HourOfDay(5);
        let hour2 = HourOfDay(8);
        let _ = hour1 / hour2;
    }

    #[test]
    fn sub_works() {
        let hour = HourOfDay::zero();
        let hour1 = HourOfDay(10);
        let hour2 = HourOfDay(3);
        let result = hour1 - hour2;
        assert_eq!(HourOfDay(10), hour1 - hour);
        assert_eq!(HourOfDay(0), hour1 - hour1);
        assert_eq!(result.0, 7);
    }

    #[test]
    fn add_works() {
        let hour = HourOfDay::zero();
        let hour1 = HourOfDay::one();
        let hour2 = HourOfDay(5);
        assert_eq!(HourOfDay(5), hour + hour2);
        assert_eq!(HourOfDay(5), hour2 + hour);
        assert_eq!(HourOfDay(6), hour1 + hour2);
    }

    #[test]
    fn rem_works() {
        let hour = HourOfDay::zero();
        let hour1 = HourOfDay::one();
        let hour2 = HourOfDay(3);
        assert_eq!(HourOfDay(0), hour % hour1);
        assert_eq!(HourOfDay(0), hour1 % hour1);
        assert_eq!(HourOfDay(1), hour1 % hour2);
    }

    #[test]
    fn from_i64_works() -> Result<()> {
        for i in 0..=23 {
            let hour_opt = HourOfDay::from_i64(i);
            assert!(hour_opt.is_some());
            let hour = hour_opt.unwrap();
            assert_eq!(u8::try_from(i)?, hour.0);
        }
        assert!(HourOfDay::from_i64(24).is_none());
        Ok(())
    }

    #[test]
    fn from_u64_works() {
        for i in 0..=23 {
            let hour_opt = HourOfDay::from_u64(u64::from(i));
            assert!(hour_opt.is_some());
            let hour = hour_opt.unwrap();
            assert_eq!(i, hour.0);
        }
        assert!(HourOfDay::from_u64(24).is_none());
    }

    #[test]
    fn to_i64_works() {
        for i in 0..=23 {
            let hour = HourOfDay(i);
            let hour_i64_opt = hour.to_i64();
            assert!(hour_i64_opt.is_some());
            let hour_i64 = hour_i64_opt.unwrap();
            assert_eq!(i64::from(i), hour_i64);
        }
    }

    #[test]
    fn to_u64_works() {
        for i in 1..=31 {
            let hour = HourOfDay(i);
            let hour_u64_opt = hour.to_u64();
            assert!(hour_u64_opt.is_some());
            let hour_u64 = hour_u64_opt.unwrap();
            assert_eq!(u64::from(i), hour_u64);
        }
    }

    #[test]
    fn u8_from_works() {
        for i in 1..=31 {
            let hour = HourOfDay(i);
            let hour_u8 = u8::from(hour);
            assert_eq!(i, hour_u8);
        }
    }
}
