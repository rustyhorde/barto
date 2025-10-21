// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use num_traits::{Bounded, NumCast, ToPrimitive};

use crate::realtime::cv::ConstrainedValue;

#[allow(dead_code)]
pub(crate) type Day = ConstrainedValue<DayOfMonth>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub(crate) struct DayOfMonth(pub(crate) u8);

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

impl NumCast for DayOfMonth {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        n.to_u8().and_then(|v| {
            if (1..=31).contains(&v) {
                Some(DayOfMonth(v))
            } else {
                None
            }
        })
    }
}
