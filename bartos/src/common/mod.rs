// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::HashMap;

use bon::Builder;
use getset::Getters;
use uuid::Uuid;

#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq)]
pub(crate) struct Clients {
    #[getset(get = "pub(crate)")]
    #[builder(default)]
    clients: HashMap<Uuid, String>,
}

impl Clients {
    pub(crate) fn add_client(&mut self, id: Uuid, description: String) -> Option<String> {
        self.clients.insert(id, description)
    }

    pub(crate) fn remove_client(&mut self, id: &Uuid) -> Option<String> {
        self.clients.remove(id)
    }
}
