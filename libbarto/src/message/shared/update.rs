// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::cmp::Ordering;

use bincode::{Decode, Encode};
use bon::Builder;
use getset::{CopyGetters, Getters};

/// The update kind
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum UpdateKind {
    /// A garuda-update message
    Garuda(Vec<Garuda>),
    /// An Archlinux pacman update message
    Pacman(Pacman),
    /// A `CachyOS` update message
    Cachyos(Pacman),
    /// An other update message
    Other,
}

/// A garuda-update message
#[derive(Builder, Clone, Debug, Decode, Encode, Eq, Getters, PartialEq)]
pub struct Garuda {
    /// The channel the package belongs to
    #[get = "pub"]
    #[builder(into)]
    channel: String,
    /// The package that was updated
    #[get = "pub"]
    #[builder(into)]
    package: String,
    /// The old version of the package
    #[get = "pub"]
    #[builder(into)]
    old_version: String,
    /// The new version of the package
    #[get = "pub"]
    #[builder(into)]
    new_version: String,
    /// The net change in size (in MiB)
    #[get = "pub"]
    #[builder(into)]
    size_change: String,
    /// The download size (in MiB)
    #[get = "pub"]
    #[builder(into)]
    download_size: String,
}

impl Ord for Garuda {
    fn cmp(&self, other: &Self) -> Ordering {
        self.channel
            .cmp(&other.channel)
            .then(self.package.cmp(&other.package))
    }
}

impl PartialOrd for Garuda {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A garuda-update message
#[derive(Builder, Clone, CopyGetters, Debug, Decode, Encode, Getters, PartialEq)]
pub struct Pacman {
    /// The package update count
    #[get_copy = "pub"]
    update_count: usize,
    /// The channel the package belongs to
    #[get = "pub"]
    #[builder(into)]
    packages: Vec<String>,
    /// The install size (in MiB)
    #[get_copy = "pub"]
    install_size: f64,
    /// The net size change (in MiB)
    #[get_copy = "pub"]
    net_size: f64,
    /// The download size (in MiB)
    #[get_copy = "pub"]
    download_size: f64,
}
