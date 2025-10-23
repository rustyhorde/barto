// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::{Error, Result};
use bincode::{Decode, Encode};

/// A message from a worker client to a worker session
#[derive(Clone, Debug, Decode, Encode)]
pub enum BartoCli {
    /// An info request from barto-cli
    Info {
        /// Output the information in JSON format
        json: bool,
    },
    /// A request from barto-cli to check for recent updates to a bartoc client
    Updates {
        /// The name of the bartoc client to check for recent updates
        name: String,
        /// The update kind we are requesting
        kind: UpdateKind,
    },
    /// A request from barto-cli to clean up old database entries
    Cleanup,
    /// The currently connected clients
    Clients,
    /// A query to run on bartos
    Query {
        /// The query to run on bartos
        query: String,
    },
}

/// The update kind we are requesting
#[derive(Clone, Copy, Debug, Decode, Encode)]
pub enum UpdateKind {
    /// A garuda-update message
    Garuda,
    /// An Archlinux pacman update message
    Pacman,
    /// A `CachyOS` update message
    Cachyos,
    /// An other update message
    Other,
}

impl TryFrom<&str> for UpdateKind {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "garuda" => Ok(UpdateKind::Garuda),
            "pacman" => Ok(UpdateKind::Pacman),
            "other" => Ok(UpdateKind::Other),
            _ => Err(crate::Error::InvalidUpdateKind {
                kind: value.to_string(),
            }
            .into()),
        }
    }
}
