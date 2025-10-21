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
    error::Error::InvalidYear,
    realtime::cv::{ConstrainedValue, ConstrainedValueParser},
};

static YEAR_RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(-?\d+)\.\.(-?\d+)$").expect("invalid year range regex"));
static YEAR_REPETITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(-?\d+)(\.\.(-?\d+))?/(\d+)$").expect("invalid repetition regex")
});

pub(crate) type Year = ConstrainedValue<i32>;

impl TryFrom<&str> for Year {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        Year::parse(s)
    }
}

impl FromStr for Year {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Year::try_from(s)
    }
}

impl ConstrainedValueParser<'_, i32> for Year {
    fn invalid(s: &str) -> Error {
        InvalidYear(s.to_string()).into()
    }

    fn all() -> Self {
        Year::All
    }

    fn repetition_regex() -> Regex {
        YEAR_REPETITION_RE.clone()
    }

    fn range_regex() -> Regex {
        YEAR_RANGE_RE.clone()
    }

    fn rep(start: i32, end: Option<i32>, rep: u8) -> Self {
        Year::Repetition { start, end, rep }
    }

    fn range(first: i32, second: i32) -> Self {
        Year::Range(first, second)
    }

    fn specific(values: Vec<i32>) -> Self {
        Year::Specific(values)
    }
}
