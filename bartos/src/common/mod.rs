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

/// A signal broadcast from bartos to every connected bartoc worker task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerSignal {
    /// Re-send the (possibly updated) schedules to the worker.
    Reload,
    /// Ask the worker to clean up old entries from its local redb database.
    Cleanup,
}

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

#[cfg(test)]
mod tests {
    use libbarto::BartocInfo;
    use uuid::Uuid;

    use super::{Clients, WorkerSignal};

    #[test]
    fn add_client_inserts_and_replaces() {
        let mut clients = Clients::builder().build();
        let id = Uuid::new_v4();
        assert!(clients.add_client(id, "host1", "10.0.0.1").is_none());
        assert_eq!(clients.clients().len(), 1);
        let cd = clients.clients().get(&id).expect("present");
        assert_eq!(cd.name(), "host1");
        assert_eq!(cd.ip(), "10.0.0.1");
        // Re-adding the same id returns the previous ClientData.
        let prev = clients.add_client(id, "host1-new", "10.0.0.2");
        assert!(prev.is_some());
        assert_eq!(clients.clients().len(), 1);
        assert_eq!(
            clients.clients().get(&id).expect("present").name(),
            "host1-new"
        );
    }

    #[test]
    fn remove_client_by_id() {
        let mut clients = Clients::builder().build();
        let id = Uuid::new_v4();
        let _old = clients.add_client(id, "host1", "10.0.0.1");
        let removed = clients.remove_client(&id).expect("removed");
        assert_eq!(removed.name(), "host1");
        assert!(clients.clients().is_empty());
        // Removing again returns None.
        assert!(clients.remove_client(&id).is_none());
    }

    #[test]
    fn remove_client_by_name_present_and_absent() {
        let mut clients = Clients::builder().build();
        let id = Uuid::new_v4();
        let _old = clients.add_client(id, "host1", "10.0.0.1");
        assert!(clients.remove_client_by_name("nope").is_none());
        let removed = clients.remove_client_by_name("host1").expect("removed");
        assert_eq!(removed.name(), "host1");
        assert!(clients.clients().is_empty());
    }

    #[test]
    fn add_client_data_sets_info_for_known_id_only() {
        let mut clients = Clients::builder().build();
        let id = Uuid::new_v4();
        let _old = clients.add_client(id, "host1", "10.0.0.1");
        let info = BartocInfo::builder()
            .name("client".to_string())
            .os_version("1.0".to_string())
            .kernel_version("6.0".to_string())
            .version("1.5.11".to_string())
            .build();
        clients.add_client_data(&id, info.clone());
        assert_eq!(
            clients.clients().get(&id).expect("present").bartoc_info(),
            &Some(info)
        );
        // Unknown id is a no-op (does not panic, does not insert).
        clients.add_client_data(&Uuid::new_v4(), BartocInfo::builder().build());
        assert_eq!(clients.clients().len(), 1);
    }

    #[test]
    fn worker_signal_equality() {
        assert_eq!(WorkerSignal::Reload, WorkerSignal::Reload);
        assert_ne!(WorkerSignal::Reload, WorkerSignal::Cleanup);
        let copied = WorkerSignal::Cleanup;
        assert_eq!(copied, WorkerSignal::Cleanup);
    }
}
