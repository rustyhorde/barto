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
    error::Error::InvalidMinuteOfHour,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

pub(crate) static MINUTE_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.\.(\d+)$").expect("invalid minute range regex"));
pub(crate) static MINUTE_REPETITION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)(\.\.(\d+))?/(\d+)$").expect("invalid repetition regex"));

pub(crate) type Minute = ConstrainedValue<MinuteOfHour>;

impl TryFrom<&str> for Minute {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Minute::parse(s)
    }
}

impl FromStr for Minute {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Minute::try_from(s)
    }
}

impl ConstrainedValueParser<'_, MinuteOfHour> for Minute {
    fn invalid(s: &str) -> Error {
        InvalidMinuteOfHour(s.to_string()).into()
    }

    fn allow_rand() -> bool {
        true
    }

    fn all() -> Self {
        Minute::All
    }

    fn rand() -> Self {
        let rand_minute = rng().random_range(
            u8::from(MinuteOfHour::min_value())..=u8::from(MinuteOfHour::max_value()),
        );
        Minute::Specific(vec![MinuteOfHour(rand_minute)])
    }

    fn repetition_regex() -> Regex {
        MINUTE_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        MINUTE_RANGE_RE.clone()
    }

    fn rep(start: MinuteOfHour, end: Option<MinuteOfHour>, rep: u8) -> Self {
        Minute::Repetition { start, end, rep }
    }

    fn range(first: MinuteOfHour, second: MinuteOfHour) -> Self {
        Minute::Range(first, second)
    }

    fn specific(values: Vec<MinuteOfHour>) -> Self {
        Minute::Specific(values)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MinuteOfHour(pub(crate) u8);

impl Bounded for MinuteOfHour {
    fn min_value() -> Self {
        MinuteOfHour(0)
    }

    fn max_value() -> Self {
        MinuteOfHour(59)
    }
}

impl ToPrimitive for MinuteOfHour {
    fn to_i64(&self) -> Option<i64> {
        Some(<i64 as From<u8>>::from(self.0))
    }

    fn to_u64(&self) -> Option<u64> {
        Some(<u64 as From<u8>>::from(self.0))
    }
}

impl FromPrimitive for MinuteOfHour {
    fn from_i64(n: i64) -> Option<Self> {
        if (0..=59).contains(&n) {
            Some(MinuteOfHour(u8::try_from(n).ok()?))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        if (0..=59).contains(&n) {
            Some(MinuteOfHour(u8::try_from(n).ok()?))
        } else {
            None
        }
    }
}

impl Zero for MinuteOfHour {
    fn zero() -> Self {
        MinuteOfHour(0)
    }

    fn is_zero(&self) -> bool {
        *self == MinuteOfHour::zero()
    }
}

impl One for MinuteOfHour {
    fn one() -> Self {
        MinuteOfHour(1)
    }
}

impl Add for MinuteOfHour {
    type Output = MinuteOfHour;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else {
            let new = MinuteOfHour(self.0 + rhs.0);
            if new > MinuteOfHour::max_value() {
                panic!("MinuteOfHour addition overflowed");
            } else {
                new
            }
        }
    }
}

impl Sub for MinuteOfHour {
    type Output = MinuteOfHour;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.0.cmp(&self.0) {
            Ordering::Greater => panic!("MinuteOfHour subtraction underflowed"),
            Ordering::Equal => MinuteOfHour::zero(),
            Ordering::Less => MinuteOfHour(self.0 - rhs.0),
        }
    }
}

impl Mul for MinuteOfHour {
    type Output = MinuteOfHour;

    fn mul(self, _rhs: Self) -> Self::Output {
        panic!("MinuteOfHour multiplication is not supported");
    }
}

impl Div for MinuteOfHour {
    type Output = MinuteOfHour;

    fn div(self, _rhs: Self) -> Self::Output {
        panic!("MinuteOfHour division is not supported");
    }
}

impl Rem for MinuteOfHour {
    type Output = MinuteOfHour;

    fn rem(self, rhs: Self) -> Self::Output {
        MinuteOfHour(self.0 % rhs.0)
    }
}

impl FromStr for MinuteOfHour {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = s
            .parse::<u8>()
            .map_err(|_| InvalidMinuteOfHour(s.to_string()))?;
        if (0..=59).contains(&value) {
            Ok(MinuteOfHour(value))
        } else {
            Err(InvalidMinuteOfHour(s.to_string()).into())
        }
    }
}

impl From<MinuteOfHour> for u8 {
    fn from(hour: MinuteOfHour) -> u8 {
        hour.0
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::{cmp::Ordering, fmt::Write as _, sync::LazyLock};

    use anyhow::Result;
    use num_traits::{Bounded as _, FromPrimitive as _, One as _, ToPrimitive as _, Zero as _};
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::{MINUTE_RANGE_RE, MINUTE_REPETITION_RE, Minute, MinuteOfHour};

    pub(crate) static VALID_MINUTE_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^\+?(0?|0?[1-9]|[1-5][0-9])$").unwrap());

    // Valid strategy generators
    prop_compose! {
        pub(crate) fn minute_strategy()(num in any::<u8>()) -> (String, u8) {
            let minute = num % 60;
            (minute.to_string(), minute)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in minute_strategy(), second in minute_strategy()) -> (String, u8, u8) {
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
        fn arb_valid_repetition_no_end()(first in minute_strategy(), rep in any::<u8>()) -> String {
            let (mut first_str, _) = first;
            let rep = if rep == 0 { 1 } else { rep };
            write!(first_str, "/{rep}").unwrap();
            first_str
        }
    }

    // Invalid strategy generators
    prop_compose! {
        pub fn invalid_minute_strategy()(num in any::<u8>()) -> String {
            let minute = if num <= 59 {
                num + 60
            } else {
                num
            };
            minute.to_string()
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in minute_strategy(), second in minute_strategy()) -> String {
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
            prop_assume!(!VALID_MINUTE_RE.is_match(s.as_str()));
            prop_assume!(!MINUTE_REPETITION_RE.is_match(s.as_str()));
            prop_assume!(!MINUTE_RANGE_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Minute::try_from(s.as_str()).is_err());
            assert!(s.parse::<Minute>().is_err());
        }

        #[test]
        fn invalid_minute_errors(s in invalid_minute_strategy()) {
            let minute_res = Minute::try_from(s.as_str());
            assert!(minute_res.is_err());
            let minute_res = s.parse::<Minute>();
            assert!(minute_res.is_err());
        }

        #[test]
        fn arb_invalid_range_errors(s in arb_invalid_range()) {
            assert!(Minute::try_from(s.as_str()).is_err());
            assert!(s.parse::<Minute>().is_err());
        }

        #[test]
        fn arb_invalid_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Minute::try_from(s.as_str()).is_err());
            assert!(s.parse::<Minute>().is_err());
        }
    }

    // Valid input tests
    proptest! {
        #[test]
        fn arb_valid_minute(value in minute_strategy()) {
            let (minute_str, _) = value;
            let minute_res = Minute::try_from(minute_str.as_str());
            assert!(minute_res.is_ok());
            let minute_res = minute_str.parse::<Minute>();
            assert!(minute_res.is_ok());
        }

        #[test]
        fn arb_valid_minute_range(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Minute::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Minute>().is_ok());
        }

        #[test]
        fn arb_valid_minute_repetition(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Minute::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Minute>().is_ok());
        }

        #[test]
        fn arb_valid_minute_repetition_no_end(s in arb_valid_repetition_no_end()) {
            assert!(Minute::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Minute>().is_ok());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            prop_assume!(min != max);
            match Minute::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(cv_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    assert!(cv_range.matches(MinuteOfHour(in_range)), "minute {in_range} should match range '{range_str}'");
                    if min > u8::from(MinuteOfHour::min_value()) {
                        let below = rng().random_range(u8::from(MinuteOfHour::min_value())..min);
                        assert!(!cv_range.matches(MinuteOfHour(below)), "minute {below} should not match range '{range_str}'");
                    }
                    if max + 1 < u8::from(MinuteOfHour::max_value()) {
                        let above = rng().random_range((max + 1)..=u8::from(MinuteOfHour::max_value()));
                        assert!(!cv_range.matches(MinuteOfHour(above)), "minute {above} should not match range '{range_str}'");
                    }
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Minute::try_from("").is_err());
        assert!("".parse::<Minute>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Minute::All, Minute::try_from("*")?);
        assert_eq!(Minute::All, "*".parse::<Minute>()?);
        Ok(())
    }

    #[test]
    fn rand_works() {
        assert!(Minute::try_from("R").is_ok());
        assert!("R".parse::<Minute>().is_ok());
    }

    #[test]
    #[should_panic(expected = "MinuteOfHour addition overflowed")]
    fn add_panics_properly() {
        let minute1 = MinuteOfHour(58);
        let minute2 = MinuteOfHour(5);
        let _ = minute1 + minute2;
    }

    #[test]
    #[should_panic(expected = "MinuteOfHour subtraction underflowed")]
    fn sub_panics_properly() {
        let minute1 = MinuteOfHour(5);
        let minute2 = MinuteOfHour(8);
        let _ = minute1 - minute2;
    }

    #[test]
    #[should_panic(expected = "MinuteOfHour multiplication is not supported")]
    fn mul_panics_properly() {
        let minute1 = MinuteOfHour(5);
        let minute2 = MinuteOfHour(8);
        let _ = minute1 * minute2;
    }

    #[test]
    #[should_panic(expected = "MinuteOfHour division is not supported")]
    fn div_panics_properly() {
        let minute1 = MinuteOfHour(5);
        let minute2 = MinuteOfHour(8);
        let _ = minute1 / minute2;
    }

    #[test]
    fn sub_works() {
        let minute = MinuteOfHour::zero();
        let minute1 = MinuteOfHour(10);
        let minute2 = MinuteOfHour(3);
        let result = minute1 - minute2;
        assert_eq!(MinuteOfHour(10), minute1 - minute);
        assert_eq!(MinuteOfHour(0), minute1 - minute1);
        assert_eq!(result.0, 7);
    }

    #[test]
    fn add_works() {
        let minute = MinuteOfHour::zero();
        let minute1 = MinuteOfHour::one();
        let minute2 = MinuteOfHour(5);
        assert_eq!(MinuteOfHour(5), minute + minute2);
        assert_eq!(MinuteOfHour(5), minute2 + minute);
        assert_eq!(MinuteOfHour(6), minute1 + minute2);
    }

    #[test]
    fn rem_works() {
        let minute = MinuteOfHour::zero();
        let minute1 = MinuteOfHour::one();
        let minute2 = MinuteOfHour(3);
        assert_eq!(MinuteOfHour(0), minute % minute1);
        assert_eq!(MinuteOfHour(0), minute1 % minute1);
        assert_eq!(MinuteOfHour(1), minute1 % minute2);
    }

    #[test]
    fn from_i64_works() -> Result<()> {
        for i in 0..=59 {
            let minute_opt = MinuteOfHour::from_i64(i);
            assert!(minute_opt.is_some());
            let minute = minute_opt.unwrap();
            assert_eq!(u8::try_from(i)?, minute.0);
        }
        assert!(MinuteOfHour::from_i64(60).is_none());
        Ok(())
    }

    #[test]
    fn from_u64_works() {
        for i in 0..=59 {
            let minute_opt = MinuteOfHour::from_u64(u64::from(i));
            assert!(minute_opt.is_some());
            let minute = minute_opt.unwrap();
            assert_eq!(i, minute.0);
        }
        assert!(MinuteOfHour::from_u64(60).is_none());
    }

    #[test]
    fn to_i64_works() {
        for i in 0..=59 {
            let minute = MinuteOfHour(i);
            let minute_i64_opt = minute.to_i64();
            assert!(minute_i64_opt.is_some());
            let minute_i64 = minute_i64_opt.unwrap();
            assert_eq!(i64::from(i), minute_i64);
        }
    }

    #[test]
    fn to_u64_works() {
        for i in 1..=59 {
            let minute = MinuteOfHour(i);
            let minute_u64_opt = minute.to_u64();
            assert!(minute_u64_opt.is_some());
            let minute_u64 = minute_u64_opt.unwrap();
            assert_eq!(u64::from(i), minute_u64);
        }
    }

    #[test]
    fn u8_from_works() {
        for i in 1..=59 {
            let minute = MinuteOfHour(i);
            let minute_u8 = u8::from(minute);
            assert_eq!(i, minute_u8);
        }
    }
}
