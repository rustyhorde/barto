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
use libbarto::{BartocInfo, ClientData};
use uuid::Uuid;

#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq)]
pub(crate) struct Clients {
    #[getset(get = "pub(crate)")]
    #[builder(default)]
    clients: HashMap<Uuid, ClientData>,
}

impl Clients {
    pub(crate) fn add_client(&mut self, id: Uuid, name: &str, ip: &str) -> Option<ClientData> {
        let cd = ClientData::builder()
            .name(name.to_string())
            .ip(ip.to_string())
            .build();
        self.clients.insert(id, cd)
    }

    pub(crate) fn remove_client(&mut self, id: &Uuid) -> Option<ClientData> {
        self.clients.remove(id)
    }

    pub(crate) fn remove_client_by_name(&mut self, name: &str) -> Option<ClientData> {
        let id = self
            .clients
            .iter()
            .find_map(|(id, cd)| if cd.name() == name { Some(*id) } else { None })?;
        self.clients.remove(&id)
    }

    pub(crate) fn add_client_data(&mut self, id: &Uuid, bartoc_info: BartocInfo) {
        if let Some(cd) = self.clients.get_mut(id) {
            let _ = cd.set_bartoc_info(Some(bartoc_info));
        }
    }
}
