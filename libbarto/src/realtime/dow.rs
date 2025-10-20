// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Error, Result};
use regex::Regex;

use crate::{
    error::Error::{InvalidDayOfWeek, InvalidRange},
    utils::until_err,
};

pub(crate) struct Dow(pub(crate) Option<Vec<u8>>);

static DOW_RANGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-zA-Z]{3,9})\.\.([a-zA-Z]{3,9})$").expect("invalid day of week range regex")
});

impl Dow {
    fn invalid_dow(dow: &str) -> Error {
        InvalidDayOfWeek(dow.to_string()).into()
    }

    fn parse_dowish(dowish: &str) -> Result<Vec<u8>> {
        if DOW_RANGE_RE.is_match(dowish) {
            Self::parse_dow_range(dowish)
        } else {
            Self::parse_dow_v(dowish)
        }
    }

    fn parse_dow_range(dow_range: &str) -> Result<Vec<u8>> {
        if let Some(caps) = DOW_RANGE_RE.captures(dow_range) {
            let first = Self::parse_dow(&caps[1])?;
            let second = Self::parse_dow(&caps[2])?;
            if second < first {
                Err(InvalidRange(dow_range.to_string()).into())
            } else {
                Ok((first..=second).collect())
            }
        } else {
            Err(InvalidRange(dow_range.to_string()).into())
        }
    }

    fn parse_dow_v(dow: &str) -> Result<Vec<u8>> {
        Self::parse_dow(dow).map(|x| vec![x])
    }

    fn parse_dow(dow: &str) -> Result<u8> {
        if dow.len() > 9 {
            Err(Self::invalid_dow(dow))
        } else {
            let res = if dow == "Sun" || dow == "Sunday" {
                0
            } else if dow == "Mon" || dow == "Monday" {
                1
            } else if dow == "Tue" || dow == "Tuesday" {
                2
            } else if dow == "Wed" || dow == "Wednesday" {
                3
            } else if dow == "Thu" || dow == "Thursday" {
                4
            } else if dow == "Fri" || dow == "Friday" {
                5
            } else if dow == "Sat" || dow == "Saturday" {
                6
            } else {
                return Err(Self::invalid_dow(dow));
            };
            Ok(res)
        }
    }
}

impl TryFrom<&str> for Dow {
    type Error = Error;

    fn try_from(dowish: &str) -> Result<Self> {
        if dowish.is_empty() {
            Err(Self::invalid_dow(dowish))
        } else if dowish == "*" {
            Ok(Dow(None))
        } else {
            let mut err = Ok(());
            let mut dows: Vec<u8> = dowish
                .split(',')
                .map(Self::parse_dowish)
                .scan(&mut err, until_err)
                .flatten()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            err?;
            dows.sort_unstable();
            Ok(Dow(Some(dows)))
        }
    }
}

impl FromStr for Dow {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Dow::try_from(s)
    }
}

impl Display for Dow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(vals) => {
                let len = vals.len();
                for (idx, val) in vals.iter().enumerate() {
                    match val {
                        0 => write!(f, "Sun")?,
                        1 => write!(f, "Mon")?,
                        2 => write!(f, "Tue")?,
                        3 => write!(f, "Wed")?,
                        4 => write!(f, "Thu")?,
                        5 => write!(f, "Fri")?,
                        6 => write!(f, "Sat")?,
                        _ => write!(f, "Unk")?,
                    }
                    if idx < len - 1 {
                        write!(f, ",")?;
                    }
                }
            }
            None => write!(f, "*")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::Ordering, sync::LazyLock};

    use anyhow::Result;
    use proptest::{
        prelude::{any, proptest},
        prop_assume, prop_compose,
    };

    use super::Dow;

    static SHORT_DOWS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    static LONG_DOWS: &[&str] = &[
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    static ALL_DOWS: LazyLock<Vec<&str>> = LazyLock::new(|| {
        SHORT_DOWS
            .iter()
            .chain(LONG_DOWS.iter())
            .copied()
            .collect::<Vec<&str>>()
    });

    prop_compose! {
        fn arb_dow() (idx in any::<u8>(), long in any::<bool>()) -> (String, u8) {
            let idx = idx % 7;
            if long {
                (LONG_DOWS[usize::from(idx)].to_string(), idx)
            } else {
                (SHORT_DOWS[usize::from(idx)].to_string(), idx)
            }
        }
    }

    prop_compose! {
        fn arb_dow_range() (first in arb_dow(), second in arb_dow()) -> (String, u8, u8) {
            let (first_dow, first_idx) = first;
            let (second_dow, second_idx) = second;
            if first_idx <= second_idx {
                (format!("{first_dow}..{second_dow}"), first_idx, second_idx)
            } else {
                (format!("{second_dow}..{first_dow}"), second_idx, first_idx)
            }
        }
    }

    prop_compose! {
        fn invalid_dow_range() (first in arb_dow(), second in arb_dow()) -> (String, u8, u8) {
            let (first_dow, first_idx) = first;
            let (second_dow, second_idx) = second;

            match first_idx.cmp(&second_idx) {
                Ordering::Less => (format!("{second_dow}..{first_dow}"), second_idx, first_idx),
                Ordering::Equal => {
                    let new_first_idx = (first_idx + 1) % 7;
                    (format!("{new_first_idx}..{second_dow}"), new_first_idx, second_idx)
                },
                Ordering::Greater => (format!("{first_dow}..{second_dow}"), first_idx, second_idx),
            }
        }
    }

    proptest! {
        #[test]
        fn random_input_errors(s in "\\PC*") {
            prop_assume!(!ALL_DOWS.contains(&s.as_str()));
            prop_assume!(s != "*");
            assert!(Dow::try_from(s.as_str()).is_err());
            assert!(s.parse::<Dow>().is_err());
        }

        #[test]
        fn input_too_long_errors(s in "[a-zA-Z]{10,}") {
            assert!(Dow::try_from(s.as_str()).is_err());
            assert!(s.parse::<Dow>().is_err());
        }

        #[test]
        fn input_invalid_errors(s in "[a-zA-Z]{0,9}") {
            prop_assume!(!ALL_DOWS.contains(&s.as_str()));
            assert!(Dow::try_from(s.as_str()).is_err());
            assert!(s.parse::<Dow>().is_err());
        }

        #[test]
        fn invalid_dow_range_errors(s in invalid_dow_range()) {
            let (range_str, _, _) = s;
            assert!(Dow::try_from(range_str.as_str()).is_err());
        }

        fn valid_single_dow_works(s in arb_dow()) {
            let (dow_str, _) = s;
            assert!(Dow::try_from(dow_str.as_str()).is_ok());
        }

        #[test]
        fn valid_dow_range_works(s in arb_dow_range()) {
            let (range_str, _, _) = s;
            assert!(Dow::try_from(range_str.as_str()).is_ok());
        }
    }

    #[test]
    fn empty_string_errors() {
        assert!(Dow::try_from("").is_err());
        assert!("".parse::<Dow>().is_err());
    }

    #[test]
    fn all() {
        let res = Dow::try_from("*");
        assert!(res.is_ok());
        let dow = res.unwrap();
        assert!(dow.0.is_none());
    }

    #[test]
    fn invalid_range() {
        assert!(Dow::try_from("Mon..Hogwash,Wed").is_err());
    }

    #[test]
    fn all_display_works() -> Result<()> {
        assert_eq!("*".parse::<Dow>()?.to_string(), "*");
        assert_eq!("Sun,Tue,Thu".parse::<Dow>()?.to_string(), "Sun,Tue,Thu");
        assert_eq!(
            "Sun,Mon,Tue,Wed,Thu,Fri,Sat".parse::<Dow>()?.to_string(),
            "Sun,Mon,Tue,Wed,Thu,Fri,Sat"
        );
        assert_eq!(Dow(Some(vec![7])).to_string(), "Unk");
        Ok(())
    }

    #[test]
    fn invalid_caps() {
        assert!(Dow::parse_dow_range("sUn").is_err());
    }
}
