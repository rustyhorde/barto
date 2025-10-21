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
use num_traits::{Bounded, FromPrimitive, NumCast, One, ToPrimitive, Zero};
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

impl NumCast for MonthOfYear {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        n.to_u8().and_then(|v| {
            if (1..=12).contains(&v) {
                Some(MonthOfYear(v))
            } else {
                None
            }
        })
    }
}

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
        let new = ((self.0 - 1) % rhs.0) + 1;
        if new == 0 {
            MonthOfYear::zero()
        } else {
            MonthOfYear(new)
        }
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
    use num_traits::{One as _, Zero as _};

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::{Month, MonthOfYear};

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
        assert_eq!(MonthOfYear(6), month1 + month2);
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
