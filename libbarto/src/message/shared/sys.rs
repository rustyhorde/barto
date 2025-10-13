// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{Display, Formatter, Result};

use bincode::{Decode, Encode};
use bon::Builder;
use getset::{Getters, Setters};
use sysinfo::System;

/// bartoc client system information
#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Getters, PartialEq)]
pub struct BartocInfo {
    /// The name of the bartoc client
    #[get = "pub"]
    #[builder(default = System::name().unwrap_or_default())]
    name: String,
    /// The hostname of the bartoc client
    #[get = "pub"]
    #[builder(default = System::host_name().unwrap_or_default())]
    hostname: String,
    /// The operating system version of the bartoc client
    #[get = "pub"]
    #[builder(default = System::os_version().unwrap_or_default())]
    os_version: String,
    /// The kernel version of the bartoc client
    #[get = "pub"]
    #[builder(default = System::kernel_version().unwrap_or_default())]
    kernel_version: String,
}

impl Display for BartocInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} {} {}",
            self.name, self.os_version, self.kernel_version
        )
    }
}

/// bartoc client data
#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Getters, PartialEq, Setters)]
pub struct ClientData {
    /// bartoc client description
    #[getset(get = "pub")]
    #[builder(default)]
    description: String,
    /// bartoc client system information
    #[getset(get = "pub", set = "pub")]
    bartoc_info: Option<BartocInfo>,
}

impl Display for ClientData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(bartoc_info) = &self.bartoc_info {
            write!(f, "{bartoc_info}")
        } else {
            write!(f, "{}", self.description)
        }
    }
}
