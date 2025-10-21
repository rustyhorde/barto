// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use core::panic;
use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use regex::Regex;

use crate::{
    error::Error::InvalidMonthOfYear,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

static MONTH_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.\.(\d+)$").expect("invalid month range regex"));
static MONTH_REPETITION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)(\.\.(\d+))?/(\d+)$").expect("invalid repetition regex"));

pub(crate) type Month = ConstrainedValue<MonthOfYear>;

impl TryFrom<&str> for Month {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Month::parse(s)
    }
}

impl FromStr for Month {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Month::try_from(s)
    }
}

impl ConstrainedValueParser<'_, MonthOfYear> for Month {
    fn invalid(s: &str) -> Error {
        InvalidMonthOfYear(s.to_string()).into()
    }

    fn all() -> Self {
        Month::All
    }

    fn repetition_regex() -> Regex {
        MONTH_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        MONTH_RANGE_RE.clone()
    }

    fn rep(start: MonthOfYear, end: Option<MonthOfYear>, rep: u8) -> Self {
        Month::Repetition { start, end, rep }
    }

    fn range(first: MonthOfYear, second: MonthOfYear) -> Self {
        Month::Range(first, second)
    }

    fn specific(values: Vec<MonthOfYear>) -> Self {
        Month::Specific(values)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MonthOfYear(pub(crate) u8);

impl Bounded for MonthOfYear {
    fn min_value() -> Self {
        MonthOfYear(1)
    }

    fn max_value() -> Self {
        MonthOfYear(12)
    }
}

impl ToPrimitive for MonthOfYear {
    fn to_i64(&self) -> Option<i64> {
        Some(<i64 as From<u8>>::from(self.0))
    }

    fn to_u64(&self) -> Option<u64> {
        Some(<u64 as From<u8>>::from(self.0))
    }
}

impl FromPrimitive for MonthOfYear {
    fn from_i64(n: i64) -> Option<Self> {
        if (1..=12).contains(&n) {
            Some(MonthOfYear(u8::try_from(n).ok()?))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        if (1..=12).contains(&n) {
            Some(MonthOfYear(u8::try_from(n).ok()?))
        } else {
            None
        }
    }
}

impl Zero for MonthOfYear {
    fn zero() -> Self {
        MonthOfYear(1)
    }

    fn is_zero(&self) -> bool {
        *self == MonthOfYear::zero()
    }
}

impl One for MonthOfYear {
    fn one() -> Self {
        MonthOfYear(2)
    }
}

// impl NumCast for MonthOfYear {
//     fn from<T: ToPrimitive>(n: T) -> Option<Self> {
//         n.to_u8().and_then(|v| {
//             if (1..=12).contains(&v) {
//                 Some(MonthOfYear(v))
//             } else {
//                 None
//             }
//         })
//     }
// }

impl Add for MonthOfYear {
    type Output = MonthOfYear;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else {
            let new = MonthOfYear(self.0 + rhs.0 - 1);
            if new > MonthOfYear::max_value() {
                panic!("MonthOfYear addition overflowed");
            } else {
                new
            }
        }
    }
}

// 1  2  3  4 5  6  7  8  9 10 11 12
// 0  1  2  3 4  5  6  7  8  9 10 11
impl Sub for MonthOfYear {
    type Output = MonthOfYear;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.0.cmp(&self.0) {
            Ordering::Greater => panic!("MonthOfYear subtraction underflowed"),
            Ordering::Equal => MonthOfYear::zero(),
            Ordering::Less => MonthOfYear(self.0 - rhs.0 + 1),
        }
    }
}

impl Mul for MonthOfYear {
    type Output = MonthOfYear;

    fn mul(self, _rhs: Self) -> Self::Output {
        panic!("MonthOfYear multiplication is not supported");
    }
}

impl Div for MonthOfYear {
    type Output = MonthOfYear;

    fn div(self, _rhs: Self) -> Self::Output {
        panic!("MonthOfYear division is not supported");
    }
}

impl Rem for MonthOfYear {
    type Output = MonthOfYear;

    fn rem(self, rhs: Self) -> Self::Output {
        MonthOfYear(((self.0 - 1) % rhs.0) + 1)
    }
}

impl FromStr for MonthOfYear {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = s
            .parse::<u8>()
            .map_err(|_| InvalidMonthOfYear(s.to_string()))?;
        if (1..=12).contains(&value) {
            Ok(MonthOfYear(value))
        } else {
            Err(InvalidMonthOfYear(s.to_string()).into())
        }
    }
}

impl From<MonthOfYear> for u8 {
    fn from(month: MonthOfYear) -> u8 {
        month.0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use anyhow::Result;
    use num_traits::{FromPrimitive as _, One as _, ToPrimitive as _, Zero as _};
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use regex::Regex;

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::{MONTH_RANGE_RE, MONTH_REPETITION_RE, Month, MonthOfYear};

    static VALID_MONTH_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(1[0-2]|[1-9])$").unwrap());

    // Valid strategy generators
    prop_compose! {
        pub fn month_strategy()(num in any::<u8>()) -> (String, u8) {
            let month = (num % 12) + 1;
            (month.to_string(), month)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in month_strategy(), second in month_strategy()) -> (String, u8, u8) {
            let (first_str, first_val) = first;
            let (second_str, second_val) = second;
            if first_val <= second_val {
                (format!("{first_str}..{second_str}"), first_val, second_val)
            } else {
                (format!("{second_str}..{first_str}"), second_val, first_val)
            }
        }
    }

    // Invalid strategy generators
    prop_compose! {
        pub fn invalid_month_strategy()(num in any::<u8>()) -> String {
            let month = if num > 0 && num <= 12 {
                num + 12
            } else {
                num
            };
            month.to_string()
        }
    }

    // Invalid input tests
    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!VALID_MONTH_RE.is_match(s.as_str()));
            prop_assume!(!MONTH_REPETITION_RE.is_match(s.as_str()));
            prop_assume!(!MONTH_RANGE_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Month::try_from(s.as_str()).is_err());
            assert!(s.parse::<Month>().is_err());
        }

        #[test]
        fn invalid_month_errors(s in invalid_month_strategy()) {
            let month_res = Month::try_from(s.as_str());
            assert!(month_res.is_err());
            let month_res = s.parse::<Month>();
            assert!(month_res.is_err());
        }
    }

    // Valid input tests
    proptest! {
        #[test]
        fn arb_valid_month(value in month_strategy()) {
            let (month_str, _) = value;
            let month_res = Month::try_from(month_str.as_str());
            assert!(month_res.is_ok());
            let month_res = month_str.parse::<Month>();
            assert!(month_res.is_ok());
        }

        #[test]
        fn arb_valid_month_range(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Month::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Month>().is_ok());
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Month::try_from("").is_err());
        assert!("".parse::<Month>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Month::All, Month::try_from("*")?);
        assert_eq!(Month::All, "*".parse::<Month>()?);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "MonthOfYear addition overflowed")]
    fn add_panics_properly() {
        let month1 = MonthOfYear(6);
        let month2 = MonthOfYear(8);
        let _ = month1 + month2;
    }

    #[test]
    #[should_panic(expected = "MonthOfYear subtraction underflowed")]
    fn sub_panics_properly() {
        let month1 = MonthOfYear(5);
        let month2 = MonthOfYear(8);
        let _ = month1 - month2;
    }

    #[test]
    #[should_panic(expected = "MonthOfYear multiplication is not supported")]
    fn mul_panics_properly() {
        let month1 = MonthOfYear(5);
        let month2 = MonthOfYear(8);
        let _ = month1 * month2;
    }

    #[test]
    #[should_panic(expected = "MonthOfYear division is not supported")]
    fn div_panics_properly() {
        let month1 = MonthOfYear(5);
        let month2 = MonthOfYear(8);
        let _ = month1 / month2;
    }

    #[test]
    fn invalid_input_errors() {
        assert!("0".parse::<MonthOfYear>().is_err());
    }

    #[test]
    fn sub_works() {
        let month = MonthOfYear::zero();
        let month1 = MonthOfYear(10);
        let month2 = MonthOfYear(3);
        let result = month1 - month2;
        assert_eq!(MonthOfYear(10), month1 - month);
        assert_eq!(result.0, 8);
    }

    #[test]
    fn add_works() {
        let month = MonthOfYear::zero();
        let month1 = MonthOfYear::one();
        let month2 = MonthOfYear(5);
        assert_eq!(MonthOfYear(5), month + month2);
        assert_eq!(MonthOfYear(5), month2 + month);
        assert_eq!(MonthOfYear(6), month1 + month2);
    }

    #[test]
    fn rem_works() {
        let month = MonthOfYear::zero();
        let month1 = MonthOfYear::one();
        let month2 = MonthOfYear(3);
        assert_eq!(MonthOfYear(1), month % month1);
        assert_eq!(MonthOfYear(2), month1 % month1);
        assert_eq!(MonthOfYear(1), month2 % month1);
    }

    #[test]
    fn from_i64_works() -> Result<()> {
        for i in 1..=12 {
            let month_opt = MonthOfYear::from_i64(i);
            assert!(month_opt.is_some());
            let month = month_opt.unwrap();
            assert_eq!(u8::try_from(i)?, month.0);
        }
        assert!(MonthOfYear::from_i64(0).is_none());
        assert!(MonthOfYear::from_i64(13).is_none());
        Ok(())
    }

    #[test]
    fn from_u64_works() {
        for i in 1..=12 {
            let month_opt = MonthOfYear::from_u64(u64::from(i));
            assert!(month_opt.is_some());
            let month = month_opt.unwrap();
            assert_eq!(i, month.0);
        }
        assert!(MonthOfYear::from_u64(0).is_none());
        assert!(MonthOfYear::from_u64(13).is_none());
    }

    #[test]
    fn to_i64_works() {
        for i in 1..=12 {
            let month = MonthOfYear(i);
            let month_i64_opt = month.to_i64();
            assert!(month_i64_opt.is_some());
            let month_i64 = month_i64_opt.unwrap();
            assert_eq!(i64::from(i), month_i64);
        }
    }

    #[test]
    fn to_u64_works() {
        for i in 1..=12 {
            let month = MonthOfYear(i);
            let month_u64_opt = month.to_u64();
            assert!(month_u64_opt.is_some());
            let month_u64 = month_u64_opt.unwrap();
            assert_eq!(u64::from(i), month_u64);
        }
    }

    #[test]
    fn u8_from_works() {
        for i in 1..=12 {
            let month = MonthOfYear(i);
            let month_u8 = u8::from(month);
            assert_eq!(i, month_u8);
        }
    }

    #[test]
    fn matches() {
        let month = "1..12/2".parse::<Month>().unwrap();
        assert!(month.matches(MonthOfYear(1)));
        assert!(!month.matches(MonthOfYear(2)));
        assert!(month.matches(MonthOfYear(3)));
        assert!(!month.matches(MonthOfYear(4)));
        assert!(month.matches(MonthOfYear(5)));
        assert!(!month.matches(MonthOfYear(6)));
        assert!(month.matches(MonthOfYear(7)));
        assert!(!month.matches(MonthOfYear(8)));
        assert!(month.matches(MonthOfYear(9)));
        assert!(!month.matches(MonthOfYear(10)));
        assert!(month.matches(MonthOfYear(11)));
        assert!(!month.matches(MonthOfYear(12)));
        let month = "1..12/3".parse::<Month>().unwrap();
        assert!(month.matches(MonthOfYear(1)));
        assert!(!month.matches(MonthOfYear(2)));
        assert!(!month.matches(MonthOfYear(3)));
        assert!(month.matches(MonthOfYear(4)));
        assert!(!month.matches(MonthOfYear(5)));
        assert!(!month.matches(MonthOfYear(6)));
        assert!(month.matches(MonthOfYear(7)));
        assert!(!month.matches(MonthOfYear(8)));
        assert!(!month.matches(MonthOfYear(9)));
        assert!(month.matches(MonthOfYear(10)));
        assert!(!month.matches(MonthOfYear(11)));
        assert!(!month.matches(MonthOfYear(12)));
        let month = "1..12/4".parse::<Month>().unwrap();
        assert!(month.matches(MonthOfYear(1)));
        assert!(!month.matches(MonthOfYear(2)));
        assert!(!month.matches(MonthOfYear(3)));
        assert!(!month.matches(MonthOfYear(4)));
        assert!(month.matches(MonthOfYear(5)));
        assert!(!month.matches(MonthOfYear(6)));
        assert!(!month.matches(MonthOfYear(7)));
        assert!(!month.matches(MonthOfYear(8)));
        assert!(month.matches(MonthOfYear(9)));
        assert!(!month.matches(MonthOfYear(10)));
        assert!(!month.matches(MonthOfYear(11)));
        assert!(!month.matches(MonthOfYear(12)));
        let month = "2..10/3".parse::<Month>().unwrap();
        assert!(!month.matches(MonthOfYear(1)));
        assert!(month.matches(MonthOfYear(2)));
        assert!(!month.matches(MonthOfYear(3)));
        assert!(!month.matches(MonthOfYear(4)));
        assert!(month.matches(MonthOfYear(5)));
        assert!(!month.matches(MonthOfYear(6)));
        assert!(!month.matches(MonthOfYear(7)));
        assert!(month.matches(MonthOfYear(8)));
        assert!(!month.matches(MonthOfYear(9)));
        assert!(!month.matches(MonthOfYear(10)));
        assert!(!month.matches(MonthOfYear(11)));
        assert!(!month.matches(MonthOfYear(12)));
    }
}
