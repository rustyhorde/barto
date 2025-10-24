// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use rand::{Rng as _, rng};
use regex::Regex;

use crate::{
    error::Error::InvalidDayOfMonth,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

pub(crate) static DAY_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\+?\d+)\.\.(\+?\d+)$").expect("invalid day range regex"));
pub(crate) static DAY_REPETITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\+?\d+)(\.\.(\+?\d+))?/(\+?\d+)$").expect("invalid day repetition regex")
});

/// Represents a constrained value for the day of the month (1-31)
pub type Day = ConstrainedValue<DayOfMonth>;

impl Day {
    pub(crate) fn first() -> Self {
        Day::Specific(vec![DayOfMonth::min_value()])
    }
}

impl Default for Day {
    fn default() -> Self {
        Day::All
    }
}

impl TryFrom<&str> for Day {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Day::parse(s)
    }
}

impl FromStr for Day {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Day::try_from(s)
    }
}

impl ConstrainedValueParser<'_, DayOfMonth> for Day {
    fn invalid(s: &str) -> Error {
        InvalidDayOfMonth(s.to_string()).into()
    }

    fn allow_rand() -> bool {
        true
    }

    fn all() -> Self {
        Day::All
    }

    fn rand() -> Self {
        let rand_day = rng()
            .random_range(u8::from(DayOfMonth::min_value())..=u8::from(DayOfMonth::max_value()));
        Day::Specific(vec![DayOfMonth(rand_day)])
    }

    fn repetition_regex() -> Regex {
        DAY_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        DAY_RANGE_RE.clone()
    }

    fn rep(start: DayOfMonth, end: Option<DayOfMonth>, rep: u8) -> Self {
        Day::Repetition { start, end, rep }
    }

    fn range(first: DayOfMonth, second: DayOfMonth) -> Self {
        Day::Range(first, second)
    }

    fn specific(values: Vec<DayOfMonth>) -> Self {
        Day::Specific(values)
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Day::All => write!(f, "*"),
            Day::Specific(values) => {
                let strs: Vec<String> = values.iter().map(|d| d.0.to_string()).collect();
                write!(f, "{}", strs.join(","))
            }
            Day::Range(first, second) => write!(f, "{}..{}", first.0, second.0),
            Day::Repetition { start, end, rep } => {
                if let Some(end) = end {
                    write!(f, "{}..{}/{}", start.0, end.0, rep)
                } else {
                    write!(f, "{}/{}", start.0, rep)
                }
            }
        }
    }
}

/// Represents a day of the month (1-31)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DayOfMonth(pub(crate) u8);

impl Bounded for DayOfMonth {
    fn min_value() -> Self {
        DayOfMonth(1)
    }

    fn max_value() -> Self {
        DayOfMonth(31)
    }
}

impl ToPrimitive for DayOfMonth {
    fn to_i64(&self) -> Option<i64> {
        Some(<i64 as From<u8>>::from(self.0))
    }

    fn to_u64(&self) -> Option<u64> {
        Some(<u64 as From<u8>>::from(self.0))
    }
}

impl FromPrimitive for DayOfMonth {
    fn from_i64(n: i64) -> Option<Self> {
        if (1..=31).contains(&n) {
            Some(DayOfMonth(u8::try_from(n).ok()?))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        if (1..=31).contains(&n) {
            Some(DayOfMonth(u8::try_from(n).ok()?))
        } else {
            None
        }
    }
}

impl Zero for DayOfMonth {
    fn zero() -> Self {
        DayOfMonth(1)
    }

    fn is_zero(&self) -> bool {
        *self == DayOfMonth::zero()
    }
}

impl One for DayOfMonth {
    fn one() -> Self {
        DayOfMonth(2)
    }
}

impl Add for DayOfMonth {
    type Output = DayOfMonth;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else {
            let new = DayOfMonth(self.0 + rhs.0 - 1);
            if new > DayOfMonth::max_value() {
                panic!("DayOfMonth addition overflowed");
            } else {
                new
            }
        }
    }
}

impl Sub for DayOfMonth {
    type Output = DayOfMonth;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.0.cmp(&self.0) {
            Ordering::Greater => panic!("DayOfMonth subtraction underflowed"),
            Ordering::Equal => DayOfMonth::zero(),
            Ordering::Less => DayOfMonth(self.0 - rhs.0 + 1),
        }
    }
}

impl Mul for DayOfMonth {
    type Output = DayOfMonth;

    fn mul(self, _rhs: Self) -> Self::Output {
        panic!("DayOfMonth multiplication is not supported");
    }
}

impl Div for DayOfMonth {
    type Output = DayOfMonth;

    fn div(self, _rhs: Self) -> Self::Output {
        panic!("DayOfMonth division is not supported");
    }
}

impl Rem for DayOfMonth {
    type Output = DayOfMonth;

    fn rem(self, rhs: Self) -> Self::Output {
        DayOfMonth(((self.0 - 1) % rhs.0) + 1)
    }
}

impl FromStr for DayOfMonth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = s
            .parse::<u8>()
            .map_err(|_| InvalidDayOfMonth(s.to_string()))?;
        if (1..=31).contains(&value) {
            Ok(DayOfMonth(value))
        } else {
            Err(InvalidDayOfMonth(s.to_string()).into())
        }
    }
}

impl From<DayOfMonth> for u8 {
    fn from(day: DayOfMonth) -> u8 {
        day.0
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::{cmp::Ordering, fmt::Write as _, sync::LazyLock};

    use crate::realtime::cv::ConstrainedValueMatcher as _;

    use super::{DAY_RANGE_RE, DAY_REPETITION_RE, Day, DayOfMonth};
    use anyhow::Result;
    use num_traits::{Bounded as _, FromPrimitive as _, One as _, ToPrimitive as _, Zero as _};
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };
    use rand::{Rng as _, rng};
    use regex::Regex;

    pub(crate) static VALID_DAY_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\+?(0?[1-9]|[12][0-9]|3[01])$").unwrap());

    // Valid strategy generators
    prop_compose! {
        pub(crate) fn day_strategy()(num in any::<u8>(), sign in any::<bool>()) -> (String, u8) {
            let day = (num % 31) + 1;
            let day_str = if sign {
                format!("+{day}")
            } else {
                day.to_string()
            };
            (day_str, day)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in day_strategy(), second in day_strategy()) -> (String, u8, u8) {
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
        fn arb_valid_repetition()(s in arb_valid_range(), rep in any::<u8>(), sign in any::<bool>()) -> (String, u8, u8, u8) {
            let (mut prefix, min, max) = s;
            let rep = if rep == 0 { 1 } else { rep };
            let rep_str = if sign {
                format!("+{rep}")
            } else {
                rep.to_string()
            };
            write!(prefix, "/{rep_str}").unwrap();
            (prefix, min, max, rep)
        }
    }

    prop_compose! {
        fn arb_valid_repetition_no_end()(first in day_strategy(), rep in any::<u8>()) -> String {
            let (mut first_str, _) = first;
            let rep = if rep == 0 { 1 } else { rep };
            write!(first_str, "/{rep}").unwrap();
            first_str
        }
    }

    // Invalid strategy generators
    prop_compose! {
        pub fn invalid_day_strategy()(num in any::<u8>()) -> String {
            let day = if num > 0 && num <= 31 {
                num + 31
            } else {
                num
            };
            day.to_string()
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in day_strategy(), second in day_strategy()) -> String {
            let (_, first_val) = first;
            let (_, second_val) = second;

            let new_first = if first_val == second_val {
                first_val - 1
            } else {
                first_val
            };
            match first_val.cmp(&second_val) {
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
            prop_assume!(!VALID_DAY_RE.is_match(s.as_str()));
            prop_assume!(!DAY_REPETITION_RE.is_match(s.as_str()));
            prop_assume!(!DAY_RANGE_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Day::try_from(s.as_str()).is_err());
            assert!(s.parse::<Day>().is_err());
        }

        #[test]
        fn invalid_day_errors(s in invalid_day_strategy()) {
            let day_res = Day::try_from(s.as_str());
            assert!(day_res.is_err());
            let day_res = s.parse::<Day>();
            assert!(day_res.is_err());
        }

        #[test]
        fn arb_invalid_range_errors(s in arb_invalid_range()) {
            assert!(Day::try_from(s.as_str()).is_err());
            assert!(s.parse::<Day>().is_err());
        }

        #[test]
        fn arb_invalid_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Day::try_from(s.as_str()).is_err());
            assert!(s.parse::<Day>().is_err());
        }
    }

    // Valid input tests
    proptest! {
        #[test]
        fn arb_valid_day(value in day_strategy()) {
            let (day_str, _) = value;
            let day_res = Day::try_from(day_str.as_str());
            assert!(day_res.is_ok());
            let day_res = day_str.parse::<Day>();
            assert!(day_res.is_ok());
        }

        #[test]
        fn arb_valid_day_range(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Day::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Day>().is_ok());
        }

        #[test]
        fn arb_valid_day_repetition(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Day::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Day>().is_ok());
        }

        #[test]
        fn arb_valid_day_repetition_no_end(s in arb_valid_repetition_no_end()) {
            assert!(Day::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Day>().is_ok());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            prop_assume!(min != max);
            match Day::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(cv_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    assert!(cv_range.matches(DayOfMonth(in_range)), "day {in_range} should match range '{range_str}'");
                    if min > u8::from(DayOfMonth::min_value()) {
                        let below = rng().random_range(u8::from(DayOfMonth::min_value())..min);
                        assert!(!cv_range.matches(DayOfMonth(below)), "day {below} should not match range '{range_str}'");
                    }
                    if max + 1 < u8::from(DayOfMonth::max_value()) {
                        let above = rng().random_range((max + 1)..=u8::from(DayOfMonth::max_value()));
                        assert!(!cv_range.matches(DayOfMonth(above)), "day {above} should not match range '{range_str}'");
                    }
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Day::try_from("").is_err());
        assert!("".parse::<Day>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Day::All, Day::try_from("*")?);
        assert_eq!(Day::All, "*".parse::<Day>()?);
        Ok(())
    }

    #[test]
    fn rand_works() {
        assert!(Day::try_from("R").is_ok());
        assert!("R".parse::<Day>().is_ok());
    }

    #[test]
    #[should_panic(expected = "DayOfMonth addition overflowed")]
    fn add_panics_properly() {
        let day1 = DayOfMonth(28);
        let day2 = DayOfMonth(5);
        let _ = day1 + day2;
    }

    #[test]
    #[should_panic(expected = "DayOfMonth subtraction underflowed")]
    fn sub_panics_properly() {
        let day1 = DayOfMonth(5);
        let day2 = DayOfMonth(8);
        let _ = day1 - day2;
    }

    #[test]
    #[should_panic(expected = "DayOfMonth multiplication is not supported")]
    fn mul_panics_properly() {
        let day1 = DayOfMonth(5);
        let day2 = DayOfMonth(8);
        let _ = day1 * day2;
    }

    #[test]
    #[should_panic(expected = "DayOfMonth division is not supported")]
    fn div_panics_properly() {
        let day1 = DayOfMonth(5);
        let day2 = DayOfMonth(8);
        let _ = day1 / day2;
    }

    #[test]
    fn invalid_input_errors() {
        assert!("0".parse::<DayOfMonth>().is_err());
    }

    #[test]
    fn sub_works() {
        let day = DayOfMonth::zero();
        let day1 = DayOfMonth(10);
        let day2 = DayOfMonth(3);
        let result = day1 - day2;
        assert_eq!(DayOfMonth(10), day1 - day);
        assert_eq!(result.0, 8);
    }

    #[test]
    fn add_works() {
        let day = DayOfMonth::zero();
        let day1 = DayOfMonth::one();
        let day2 = DayOfMonth(5);
        assert_eq!(DayOfMonth(5), day + day2);
        assert_eq!(DayOfMonth(5), day2 + day);
        assert_eq!(DayOfMonth(6), day1 + day2);
    }

    #[test]
    fn rem_works() {
        let day = DayOfMonth::zero();
        let day1 = DayOfMonth::one();
        let day2 = DayOfMonth(3);
        assert_eq!(DayOfMonth(1), day % day1);
        assert_eq!(DayOfMonth(2), day1 % day1);
        assert_eq!(DayOfMonth(1), day2 % day1);
    }

    #[test]
    fn from_i64_works() -> Result<()> {
        for i in 1..=31 {
            let day_opt = DayOfMonth::from_i64(i);
            assert!(day_opt.is_some());
            let day = day_opt.unwrap();
            assert_eq!(u8::try_from(i)?, day.0);
        }
        assert!(DayOfMonth::from_i64(0).is_none());
        assert!(DayOfMonth::from_i64(32).is_none());
        Ok(())
    }

    #[test]
    fn from_u64_works() {
        for i in 1..=31 {
            let day_opt = DayOfMonth::from_u64(u64::from(i));
            assert!(day_opt.is_some());
            let day = day_opt.unwrap();
            assert_eq!(i, day.0);
        }
        assert!(DayOfMonth::from_u64(0).is_none());
        assert!(DayOfMonth::from_u64(32).is_none());
    }

    #[test]
    fn to_i64_works() {
        for i in 1..=31 {
            let day = DayOfMonth(i);
            let day_i64_opt = day.to_i64();
            assert!(day_i64_opt.is_some());
            let day_i64 = day_i64_opt.unwrap();
            assert_eq!(i64::from(i), day_i64);
        }
    }

    #[test]
    fn to_u64_works() {
        for i in 1..=31 {
            let day = DayOfMonth(i);
            let day_u64_opt = day.to_u64();
            assert!(day_u64_opt.is_some());
            let day_u64 = day_u64_opt.unwrap();
            assert_eq!(u64::from(i), day_u64);
        }
    }

    #[test]
    fn u8_from_works() {
        for i in 1..=31 {
            let day = DayOfMonth(i);
            let day_u8 = u8::from(day);
            assert_eq!(i, day_u8);
        }
    }

    #[test]
    fn matches() {
        let day = "1..12/2".parse::<Day>().unwrap();
        assert!(day.matches(DayOfMonth(1)));
        assert!(!day.matches(DayOfMonth(2)));
        assert!(day.matches(DayOfMonth(3)));
        assert!(!day.matches(DayOfMonth(4)));
        assert!(day.matches(DayOfMonth(5)));
        assert!(!day.matches(DayOfMonth(6)));
        assert!(day.matches(DayOfMonth(7)));
        assert!(!day.matches(DayOfMonth(8)));
        assert!(day.matches(DayOfMonth(9)));
        assert!(!day.matches(DayOfMonth(10)));
        assert!(day.matches(DayOfMonth(11)));
        assert!(!day.matches(DayOfMonth(12)));
        let day = "1..12/3".parse::<Day>().unwrap();
        assert!(day.matches(DayOfMonth(1)));
        assert!(!day.matches(DayOfMonth(2)));
        assert!(!day.matches(DayOfMonth(3)));
        assert!(day.matches(DayOfMonth(4)));
        assert!(!day.matches(DayOfMonth(5)));
        assert!(!day.matches(DayOfMonth(6)));
        assert!(day.matches(DayOfMonth(7)));
        assert!(!day.matches(DayOfMonth(8)));
        assert!(!day.matches(DayOfMonth(9)));
        assert!(day.matches(DayOfMonth(10)));
        assert!(!day.matches(DayOfMonth(11)));
        assert!(!day.matches(DayOfMonth(12)));
        let day = "1..12/4".parse::<Day>().unwrap();
        assert!(day.matches(DayOfMonth(1)));
        assert!(!day.matches(DayOfMonth(2)));
        assert!(!day.matches(DayOfMonth(3)));
        assert!(!day.matches(DayOfMonth(4)));
        assert!(day.matches(DayOfMonth(5)));
        assert!(!day.matches(DayOfMonth(6)));
        assert!(!day.matches(DayOfMonth(7)));
        assert!(!day.matches(DayOfMonth(8)));
        assert!(day.matches(DayOfMonth(9)));
        assert!(!day.matches(DayOfMonth(10)));
        assert!(!day.matches(DayOfMonth(11)));
        assert!(!day.matches(DayOfMonth(12)));
        let day = "2..10/3".parse::<Day>().unwrap();
        assert!(!day.matches(DayOfMonth(1)));
        assert!(day.matches(DayOfMonth(2)));
        assert!(!day.matches(DayOfMonth(3)));
        assert!(!day.matches(DayOfMonth(4)));
        assert!(day.matches(DayOfMonth(5)));
        assert!(!day.matches(DayOfMonth(6)));
        assert!(!day.matches(DayOfMonth(7)));
        assert!(day.matches(DayOfMonth(8)));
        assert!(!day.matches(DayOfMonth(9)));
        assert!(!day.matches(DayOfMonth(10)));
        assert!(!day.matches(DayOfMonth(11)));
        assert!(!day.matches(DayOfMonth(12)));
    }

    #[test]
    fn default_works() {
        let default_day = Day::default();
        assert_eq!(Day::All, default_day);
    }

    #[test]
    fn first_works() {
        let first_day = Day::first();
        assert_eq!(Day::Specific(vec![DayOfMonth::min_value()]), first_day);
    }

    #[test]
    fn display_works() {
        let day = Day::All;
        assert_eq!("*", day.to_string());
        let month = Day::Specific(vec![DayOfMonth(1), DayOfMonth(3), DayOfMonth(12)]);
        assert_eq!("1,3,12", month.to_string());
        let month = Day::Range(DayOfMonth(2), DayOfMonth(8));
        assert_eq!("2..8", month.to_string());
        let month = Day::Repetition {
            start: DayOfMonth(1),
            end: Some(DayOfMonth(12)),
            rep: 3,
        };
        assert_eq!("1..12/3", month.to_string());
        let month = Day::Repetition {
            start: DayOfMonth(4),
            end: None,
            rep: 2,
        };
        assert_eq!("4/2", month.to_string());
    }
}
