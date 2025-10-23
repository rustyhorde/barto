// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{collections::HashSet, hash::Hash, str::FromStr};

use anyhow::{Error, Result};
use num_traits::{Bounded, FromPrimitive, NumOps, ToPrimitive, Zero};
use regex::Regex;

use crate::utils::until_err;

/// A value constrained by specific rules (such as the day of the month)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConstrainedValue<T>
where
    T: Constrainable,
{
    /// Every value
    All,
    /// A range of values
    Range(T, T),
    /// A repetition of values
    ///
    /// This is a sequence of values: start, start + rep, start + 2*rep
    /// up to the optional end value.  If no end value is given, it continues
    /// to the maximum `Bounded` value for the type.
    Repetition {
        /// The value to start
        start: T,
        /// An optional end value
        end: Option<T>,
        /// The repetition value
        rep: u8,
    },
    /// Specific values
    Specific(Vec<T>),
}

impl<T> ConstrainedValueMatcher<T> for ConstrainedValue<T>
where
    T: Constrainable + NumOps + Zero + Copy + FromPrimitive,
{
    fn matches(&self, value: T) -> bool {
        match self {
            Self::All => true,
            Self::Range(first, second) => value >= *first && value <= *second,
            Self::Repetition { start, end, rep } => {
                if value < *start {
                    return false;
                }
                if let Some(end_value) = end
                    && value > *end_value
                {
                    return false;
                }
                let diff = value - *start;
                diff % T::from_u8(*rep).unwrap() == T::zero()
            }
            Self::Specific(values) => values.contains(&value),
        }
    }
}

/// A trait for types that can be constrained
pub trait Constrainable:
    Bounded + Copy + Eq + FromStr + Hash + Ord + PartialEq + PartialOrd + ToPrimitive
{
}

impl<T> Constrainable for T where
    T: Bounded + Copy + Eq + FromStr + Hash + Ord + PartialEq + PartialOrd + ToPrimitive
{
}

/// A trait for parsing constrained values
pub trait ConstrainedValueParser<'a, T>:
    FromStr<Err = Error> + TryFrom<&'a str, Error = Error>
where
    T: Constrainable,
{
    /// The error to return for an invalid parse
    fn invalid(s: &str) -> Error;

    /// The regex to match repetitions
    fn repetition_regex() -> Regex;

    /// The regex to match ranges
    fn range_regex() -> Regex;

    /// Whether to allow 'R' for random value
    #[must_use]
    fn allow_rand() -> bool {
        false
    }

    /// The 'all' constrained value
    fn all() -> Self;

    /// The 'rand' constrained value
    fn rand() -> Self;

    /// The 'repetition' constrained value
    fn rep(start: T, end: Option<T>, rep: u8) -> Self;

    /// The 'range' constrained value
    fn range(first: T, second: T) -> Self;

    /// The 'specific' constrained value
    fn specific(values: Vec<T>) -> Self;

    /// Parse a constrained value from a string
    ///
    /// # Errors
    ///
    fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            Err(Self::invalid(s))
        } else if s == "*" {
            Ok(Self::all())
        } else if s == "R" && Self::allow_rand() {
            Ok(Self::rand())
        } else if Self::repetition_regex().is_match(s) {
            Self::parse_repetition(s)
        } else if Self::range_regex().is_match(s) {
            Self::parse_range(s)
        } else {
            Self::parse_specific(s)
        }
    }

    /// Parse a range constrained value from a string
    ///
    /// # Errors
    ///
    fn parse_range(s: &str) -> Result<Self> {
        if let Some(caps) = Self::range_regex().captures(s) {
            let first = caps[1].parse::<T>().map_err(|_| Self::invalid(s))?;
            let second = caps[2].parse::<T>().map_err(|_| Self::invalid(s))?;
            if (first < T::min_value() || first > T::max_value())
                || (second < T::min_value() || second > T::max_value())
                || (first > second)
            {
                Err(Self::invalid(s))
            } else {
                Ok(Self::range(first, second))
            }
        } else {
            Err(Self::invalid(s))
        }
    }

    /// Parse a repetition constrained value from a string
    ///
    /// # Errors
    ///
    fn parse_repetition(s: &str) -> Result<Self> {
        if let Some(caps) = Self::repetition_regex().captures(s) {
            let start = caps[1].parse::<T>().map_err(|_| Self::invalid(s))?;
            let end = if let Some(end_match) = caps.get(3) {
                Some(
                    end_match
                        .as_str()
                        .parse::<T>()
                        .map_err(|_| Self::invalid(s))?,
                )
            } else {
                None
            };
            let rep = caps[4].parse::<u8>().map_err(|_| Self::invalid(s))?;
            if rep == 0 || start < T::min_value() || start > T::max_value() {
                Err(Self::invalid(s))
            } else if let Some(end_val) = end {
                if end_val < start || end_val < T::min_value() || end_val > T::max_value() {
                    Err(Self::invalid(s))
                } else {
                    Ok(Self::rep(start, Some(end_val), rep))
                }
            } else {
                Ok(Self::rep(start, None, rep))
            }
        } else {
            Err(Self::invalid(s))
        }
    }

    /// Parse a specific constrained value from a string
    ///
    /// # Errors
    ///
    fn parse_specific(s: &str) -> Result<Self> {
        let mut err = Ok(());
        let mut values: Vec<T> = s
            .split(',')
            .map(|part| part.parse::<T>().map_err(|_| Self::invalid(s)))
            .scan(&mut err, until_err)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        err?;
        values.sort_unstable();
        Ok(Self::specific(values))
    }
}

/// A trait for matching constrained values
pub trait ConstrainedValueMatcher<T>
where
    T: Constrainable,
{
    /// Check if the constrained value matches the given value
    fn matches(&self, value: T) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    type Test = ConstrainedValue<u8>;

    impl TryFrom<&str> for Test {
        type Error = Error;

        #[cfg_attr(coverage_nightly, coverage(off))]
        fn try_from(s: &str) -> Result<Self> {
            Test::parse(s)
        }
    }

    impl FromStr for Test {
        type Err = Error;

        #[cfg_attr(coverage_nightly, coverage(off))]
        fn from_str(s: &str) -> Result<Self> {
            Test::try_from(s)
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    impl ConstrainedValueParser<'_, u8> for Test {
        fn invalid(s: &str) -> Error {
            Error::msg(format!("invalid constrained value: {s}"))
        }

        fn all() -> Self {
            Test::All
        }

        fn rand() -> Self {
            Test::All
        }

        fn repetition_regex() -> Regex {
            Regex::new(r"^(\d{1,3})(-(\d{1,3}))?/(\d{1,3})$").unwrap()
        }

        fn range_regex() -> Regex {
            Regex::new(r"^(\d{1,3})-(\d{1,3})$").unwrap()
        }

        fn rep(start: u8, end: Option<u8>, rep: u8) -> Self {
            Test::Repetition { start, end, rep }
        }

        fn range(first: u8, second: u8) -> Self {
            Test::Range(first, second)
        }

        fn specific(values: Vec<u8>) -> Self {
            Test::Specific(values)
        }
    }

    #[test]
    fn parse_range_errors() {
        assert!(Test::parse_range("").is_err());
    }

    #[test]
    fn parse_repetition_errors() {
        assert!(Test::parse_repetition("").is_err());
    }
}
