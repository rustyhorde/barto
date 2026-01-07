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
    error::Error::InvalidSecondOfMinute,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

pub(crate) static SECOND_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\+?\d+)\.\.(\+?\d+)$").expect("invalid second range regex"));
pub(crate) static SECOND_REPETITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\+?\d+)(\.\.(\+?\d+))?/(\+?\d+)$").expect("invalid second repetition regex")
});

/// Represents a constrained value matcher for seconds of the minute
pub type Second = ConstrainedValue<SecondOfMinute>;

impl Second {
    pub(crate) fn zero() -> Self {
        Second::Specific(vec![SecondOfMinute::zero()])
    }
}

impl Default for Second {
    fn default() -> Self {
        Second::All
    }
}

impl TryFrom<&str> for Second {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Second::parse(s)
    }
}

impl FromStr for Second {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Second::try_from(s)
    }
}

impl ConstrainedValueParser<'_, SecondOfMinute> for Second {
    fn invalid(s: &str) -> Error {
        InvalidSecondOfMinute(s.to_string()).into()
    }

    fn allow_rand() -> bool {
        true
    }

    fn all() -> Self {
        Second::All
    }

    fn rand() -> Self {
        let rand_second = rng().random_range(
            u8::from(SecondOfMinute::min_value())..=u8::from(SecondOfMinute::max_value()),
        );
        Second::Specific(vec![SecondOfMinute(rand_second)])
    }

    fn repetition_regex() -> Regex {
        SECOND_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        SECOND_RANGE_RE.clone()
    }

    fn rep(start: SecondOfMinute, end: Option<SecondOfMinute>, rep: u8) -> Self {
        Second::Repetition { start, end, rep }
    }

    fn range(first: SecondOfMinute, second: SecondOfMinute) -> Self {
        Second::Range(first, second)
    }

    fn specific(values: Vec<SecondOfMinute>) -> Self {
        Second::Specific(values)
    }
}

impl Display for Second {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Second::All => write!(f, "*"),
            Second::Specific(values) => {
                let mut first = true;
                for value in values {
                    if !first {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", value.0)?;
                    first = false;
                }
                Ok(())
            }
            Second::Range(start, end) => write!(f, "{}..{}", start.0, end.0),
            Second::Repetition { start, end, rep } => {
                if let Some(end) = end {
                    write!(f, "{}..{}/{}", start.0, end.0, rep)
                } else {
                    write!(f, "{}/{}", start.0, rep)
                }
            }
        }
    }
}

/// Represents a second of the minute (0-59)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SecondOfMinute(pub(crate) u8);

impl Bounded for SecondOfMinute {
    fn min_value() -> Self {
        SecondOfMinute(0)
    }

    fn max_value() -> Self {
        SecondOfMinute(59)
    }
}

impl ToPrimitive for SecondOfMinute {
    fn to_i64(&self) -> Option<i64> {
        Some(<i64 as From<u8>>::from(self.0))
    }

    fn to_u64(&self) -> Option<u64> {
        Some(<u64 as From<u8>>::from(self.0))
    }
}

impl FromPrimitive for SecondOfMinute {
    fn from_i64(n: i64) -> Option<Self> {
        if (0..=59).contains(&n) {
            Some(SecondOfMinute(u8::try_from(n).ok()?))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        if (0..=59).contains(&n) {
            Some(SecondOfMinute(u8::try_from(n).ok()?))
        } else {
            None
        }
    }
}

impl Zero for SecondOfMinute {
    fn zero() -> Self {
        SecondOfMinute(0)
    }

    fn is_zero(&self) -> bool {
        *self == SecondOfMinute::zero()
    }
}

impl One for SecondOfMinute {
    fn one() -> Self {
        SecondOfMinute(1)
    }
}

impl Add for SecondOfMinute {
    type Output = SecondOfMinute;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else {
            let new = SecondOfMinute(self.0 + rhs.0);
            if new > SecondOfMinute::max_value() {
                panic!("SecondOfMinute addition overflowed");
            } else {
                new
            }
        }
    }
}

impl Sub for SecondOfMinute {
    type Output = SecondOfMinute;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.0.cmp(&self.0) {
            Ordering::Greater => panic!("SecondOfMinute subtraction underflowed"),
            Ordering::Equal => SecondOfMinute::zero(),
            Ordering::Less => SecondOfMinute(self.0 - rhs.0),
        }
    }
}

impl Mul for SecondOfMinute {
    type Output = SecondOfMinute;

    fn mul(self, _rhs: Self) -> Self::Output {
        panic!("SecondOfMinute multiplication is not supported");
    }
}

impl Div for SecondOfMinute {
    type Output = SecondOfMinute;

    fn div(self, _rhs: Self) -> Self::Output {
        panic!("SecondOfMinute division is not supported");
    }
}

impl Rem for SecondOfMinute {
    type Output = SecondOfMinute;

    fn rem(self, rhs: Self) -> Self::Output {
        SecondOfMinute(self.0 % rhs.0)
    }
}

impl FromStr for SecondOfMinute {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = s
            .parse::<u8>()
            .map_err(|_| InvalidSecondOfMinute(s.to_string()))?;
        if (0..=59).contains(&value) {
            Ok(SecondOfMinute(value))
        } else {
            Err(InvalidSecondOfMinute(s.to_string()).into())
        }
    }
}

impl From<SecondOfMinute> for u8 {
    fn from(hour: SecondOfMinute) -> u8 {
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

    use super::{SECOND_RANGE_RE, SECOND_REPETITION_RE, Second, SecondOfMinute};

    pub(crate) static VALID_SECOND_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^\+?(0|0?[0-9]|[1-5][0-9])$").unwrap());

    // Valid strategy generators
    prop_compose! {
        pub(crate) fn second_strategy()(num in any::<u8>(), sign in any::<bool>()) -> (String, u8) {
            let second = num % 60;
            let second_str = if sign {
                format!("+{second}")
            } else {
                second.to_string()
            };
            (second_str, second)
        }
    }

    prop_compose! {
        fn arb_valid_range()(first in second_strategy(), second in second_strategy()) -> (String, u8, u8) {
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
        fn arb_valid_repetition_no_end()(first in second_strategy(), rep in any::<u8>()) -> String {
            let (mut first_str, _) = first;
            let rep = if rep == 0 { 1 } else { rep };
            write!(first_str, "/{rep}").unwrap();
            first_str
        }
    }

    // Invalid strategy generators
    prop_compose! {
        pub fn invalid_second_strategy()(num in any::<u8>()) -> String {
            let second = if num <= 59 {
                num + 60
            } else {
                num
            };
            second.to_string()
        }
    }

    prop_compose! {
        fn arb_invalid_range()(first in second_strategy(), second in second_strategy()) -> String {
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
            prop_assume!(!VALID_SECOND_RE.is_match(s.as_str()));
            prop_assume!(!SECOND_REPETITION_RE.is_match(s.as_str()));
            prop_assume!(!SECOND_RANGE_RE.is_match(s.as_str()));
            prop_assume!(s.as_str() != "*");
            prop_assume!(s.as_str() != "R");
            assert!(Second::try_from(s.as_str()).is_err());
            assert!(s.parse::<Second>().is_err());
        }

        #[test]
        fn invalid_second_errors(s in invalid_second_strategy()) {
            let second_res = Second::try_from(s.as_str());
            assert!(second_res.is_err());
            let second_res = s.parse::<Second>();
            assert!(second_res.is_err());
        }

        #[test]
        fn arb_invalid_range_errors(s in arb_invalid_range()) {
            assert!(Second::try_from(s.as_str()).is_err());
            assert!(s.parse::<Second>().is_err());
        }

        #[test]
        fn arb_invalid_repetition_zero_rep_errors(s in arb_invalid_repetition_zero_rep()) {
            assert!(Second::try_from(s.as_str()).is_err());
            assert!(s.parse::<Second>().is_err());
        }
    }

    // Valid input tests
    proptest! {
        #[test]
        fn arb_valid_second(value in second_strategy()) {
            let (second_str, _) = value;
            let second_res = Second::try_from(second_str.as_str());
            assert!(second_res.is_ok());
            let second_res = second_str.parse::<Second>();
            assert!(second_res.is_ok());
        }

        #[test]
        fn arb_valid_second_range(s in arb_valid_range()) {
            let (s, _, _) = s;
            assert!(Second::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Second>().is_ok());
        }

        #[test]
        fn arb_valid_second_repetition(s in arb_valid_repetition()) {
            let (prefix, _, _, _) = s;
            assert!(Second::try_from(prefix.as_str()).is_ok());
            assert!(prefix.parse::<Second>().is_ok());
        }

        #[test]
        fn arb_valid_second_repetition_no_end(s in arb_valid_repetition_no_end()) {
            assert!(Second::try_from(s.as_str()).is_ok());
            assert!(s.parse::<Second>().is_ok());
        }

        #[test]
        fn any_valid_range_matches(s in arb_valid_range()) {
            let (range_str, min, max) = s;
            prop_assume!(min != max);
            match Second::try_from(range_str.as_str()) {
                Err(e) => panic!("valid range '{range_str}' failed to parse: {e}"),
                Ok(cv_range) => for _ in 0..256 {
                    let in_range = rng().random_range(min..=max);
                    assert!(cv_range.matches(SecondOfMinute(in_range)), "second {in_range} should match range '{range_str}'");
                    if min > u8::from(SecondOfMinute::min_value()) {
                        let below = rng().random_range(u8::from(SecondOfMinute::min_value())..min);
                        assert!(!cv_range.matches(SecondOfMinute(below)), "second {below} should not match range '{range_str}'");
                    }
                    if max + 1 < u8::from(SecondOfMinute::max_value()) {
                        let above = rng().random_range((max + 1)..=u8::from(SecondOfMinute::max_value()));
                        assert!(!cv_range.matches(SecondOfMinute(above)), "second {above} should not match range '{range_str}'");
                    }
                },
            }
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Second::try_from("").is_err());
        assert!("".parse::<Second>().is_err());
    }

    #[test]
    fn all() -> Result<()> {
        assert_eq!(Second::All, Second::try_from("*")?);
        assert_eq!(Second::All, "*".parse::<Second>()?);
        Ok(())
    }

    #[test]
    fn rand_works() {
        assert!(Second::try_from("R").is_ok());
        assert!("R".parse::<Second>().is_ok());
    }

    #[test]
    #[should_panic(expected = "SecondOfMinute addition overflowed")]
    fn add_panics_properly() {
        let second1 = SecondOfMinute(58);
        let second2 = SecondOfMinute(5);
        let _ = second1 + second2;
    }

    #[test]
    #[should_panic(expected = "SecondOfMinute subtraction underflowed")]
    fn sub_panics_properly() {
        let second1 = SecondOfMinute(5);
        let second2 = SecondOfMinute(8);
        let _ = second1 - second2;
    }

    #[test]
    #[should_panic(expected = "SecondOfMinute multiplication is not supported")]
    fn mul_panics_properly() {
        let second1 = SecondOfMinute(5);
        let second2 = SecondOfMinute(8);
        let _ = second1 * second2;
    }

    #[test]
    #[should_panic(expected = "SecondOfMinute division is not supported")]
    fn div_panics_properly() {
        let second1 = SecondOfMinute(5);
        let second2 = SecondOfMinute(8);
        let _ = second1 / second2;
    }

    #[test]
    fn sub_works() {
        let second = SecondOfMinute::zero();
        let second1 = SecondOfMinute(10);
        let second2 = SecondOfMinute(3);
        let result = second1 - second2;
        assert_eq!(SecondOfMinute(10), second1 - second);
        assert_eq!(SecondOfMinute(0), second1 - second1);
        assert_eq!(result.0, 7);
    }

    #[test]
    fn add_works() {
        let second = SecondOfMinute::zero();
        let second1 = SecondOfMinute::one();
        let second2 = SecondOfMinute(5);
        assert_eq!(SecondOfMinute(5), second + second2);
        assert_eq!(SecondOfMinute(5), second2 + second);
        assert_eq!(SecondOfMinute(6), second1 + second2);
    }

    #[test]
    fn rem_works() {
        let second = SecondOfMinute::zero();
        let second1 = SecondOfMinute::one();
        let second2 = SecondOfMinute(3);
        assert_eq!(SecondOfMinute(0), second % second1);
        assert_eq!(SecondOfMinute(0), second1 % second1);
        assert_eq!(SecondOfMinute(1), second1 % second2);
    }

    #[test]
    fn from_i64_works() -> Result<()> {
        for i in 0..=59 {
            let second_opt = SecondOfMinute::from_i64(i);
            assert!(second_opt.is_some());
            let second = second_opt.unwrap();
            assert_eq!(u8::try_from(i)?, second.0);
        }
        assert!(SecondOfMinute::from_i64(60).is_none());
        Ok(())
    }

    #[test]
    fn from_u64_works() {
        for i in 0..=59 {
            let second_opt = SecondOfMinute::from_u64(u64::from(i));
            assert!(second_opt.is_some());
            let second = second_opt.unwrap();
            assert_eq!(i, second.0);
        }
        assert!(SecondOfMinute::from_u64(60).is_none());
    }

    #[test]
    fn to_i64_works() {
        for i in 0..=59 {
            let second = SecondOfMinute(i);
            let second_i64_opt = second.to_i64();
            assert!(second_i64_opt.is_some());
            let second_i64 = second_i64_opt.unwrap();
            assert_eq!(i64::from(i), second_i64);
        }
    }

    #[test]
    fn to_u64_works() {
        for i in 1..=59 {
            let second = SecondOfMinute(i);
            let second_u64_opt = second.to_u64();
            assert!(second_u64_opt.is_some());
            let second_u64 = second_u64_opt.unwrap();
            assert_eq!(u64::from(i), second_u64);
        }
    }

    #[test]
    fn u8_from_works() {
        for i in 1..=59 {
            let second = SecondOfMinute(i);
            let second_u8 = u8::from(second);
            assert_eq!(i, second_u8);
        }
    }

    #[test]
    fn default_works() {
        assert_eq!(Second::All, Second::default());
    }

    #[test]
    fn zero_works() {
        let second = Second::zero();
        assert_eq!(Second::Specific(vec![SecondOfMinute::zero()]), second);
    }

    #[test]
    fn display_works() {
        let second_all = Second::All;
        assert_eq!("*", second_all.to_string());
        let second_specific = Second::Specific(vec![SecondOfMinute(5)]);
        assert_eq!("5", second_specific.to_string());
        let multiple_seconds_specific = Second::Specific(vec![
            SecondOfMinute(10),
            SecondOfMinute(20),
            SecondOfMinute(30),
        ]);
        assert_eq!("10,20,30", multiple_seconds_specific.to_string());
        let second_range = Second::Range(SecondOfMinute(10), SecondOfMinute(15));
        assert_eq!("10..15", second_range.to_string());
        let second_repetition = Second::Repetition {
            start: SecondOfMinute(20),
            end: None,
            rep: 3,
        };
        assert_eq!("20/3", second_repetition.to_string());
        let second_repetition_with_end = Second::Repetition {
            start: SecondOfMinute(25),
            end: Some(SecondOfMinute(30)),
            rep: 5,
        };
        assert_eq!("25..30/5", second_repetition_with_end.to_string());
    }
}
