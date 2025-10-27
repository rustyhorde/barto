// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::HashMap;

use actix_web::web::{Bytes, Data};
use actix_ws::Session;
use anyhow::Result;
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use bon::Builder;
use libbarto::{BartoCli, BartosToBartoCli, CliUpdateKind, ClientData, UuidWrapper};
use tokio::sync::Mutex;
use tracing::{info, trace};
use vergen_pretty::{Pretty, PrettyExt, vergen_pretty_env};

use crate::{common::Clients, config::Config, db::Queryable};

#[derive(Builder, Clone, Debug)]
pub(crate) struct BinaryMessageHandler {
    config: Data<Config>,
    clients_mutex: Data<Mutex<Clients>>,
}

impl BinaryMessageHandler {
    fn config(&self) -> &Config {
        self.config.as_ref()
    }

    pub(crate) async fn handle<T: Queryable>(
        &mut self,
        bytes: Bytes,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        let (message, size) = decode_from_slice::<BartoCli, _>(&bytes, standard())?;
        trace!("decoded binary message of size {size} bytes");

        match message {
            BartoCli::Info { json } => self.handle_info(json, session).await,
            BartoCli::Updates { name, kind } => {
                self.handle_updates(name, kind, session, queryable).await
            }
            BartoCli::Cleanup => self.handle_cleanup(session, queryable).await,
            BartoCli::Clients => self.handle_clients(session).await,
            BartoCli::Query { query } => self.handle_query(query, session, queryable).await,
            BartoCli::List { name, cmd_name } => {
                self.handle_list(&name, &cmd_name, session, queryable).await
            }
            BartoCli::Failed => self.handle_failed(session, queryable).await,
        }
    }

    async fn handle_info(&mut self, json: bool, session: &mut Session) -> Result<()> {
        info!("received info message");
        let pretty = Pretty::builder().env(vergen_pretty_env!());

        let btbc: BartosToBartoCli = if json {
            let new_pretty = pretty.flatten(true);
            BartosToBartoCli::InfoJson(serde_json::to_string(&new_pretty.build())?)
        } else {
            let pretty_ext = PrettyExt::from(pretty.build());
            BartosToBartoCli::Info(pretty_ext)
        };
        let encoded = encode_to_vec(&btbc, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_updates<T: Queryable>(
        &mut self,
        name: String,
        kind: CliUpdateKind,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        let update_kind = queryable.update_data(self.config(), kind, &name).await?;
        let msg = BartosToBartoCli::Updates(update_kind);
        let encoded = encode_to_vec(&msg, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_cleanup<T: Queryable>(
        &mut self,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        info!("received cleanup message");
        let counts = queryable.delete_data(self.config()).await?;
        info!("deleted {} output rows", counts.0);
        info!("deleted {} exit status rows", counts.1);
        let cleanup = BartosToBartoCli::Cleanup(counts);
        let encoded = encode_to_vec(&cleanup, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_clients(&mut self, session: &mut Session) -> Result<()> {
        info!("received clients message");
        let clients = self.clients_mutex.lock().await;
        let mapped_clients = clients
            .clients()
            .iter()
            .map(|c| (UuidWrapper(*c.0), c.1.clone()))
            .collect::<HashMap<UuidWrapper, ClientData>>();
        let clients = BartosToBartoCli::Clients(mapped_clients);
        let encoded = encode_to_vec(&clients, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_list<T: Queryable>(
        &mut self,
        name: &str,
        cmd_name: &str,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        info!("received list message for '{name}' (cmd: {cmd_name})");
        let list_output = queryable
            .cmd_name_data(self.config(), name, cmd_name)
            .await?;
        let msg = BartosToBartoCli::List(list_output);
        let encoded = encode_to_vec(&msg, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_failed<T: Queryable>(
        &mut self,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        info!("received failed message");
        let failed_output = queryable.failed_cmd_data(self.config()).await?;
        let msg = BartosToBartoCli::Failed(failed_output);
        let encoded = encode_to_vec(&msg, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }

    async fn handle_query<T: Queryable>(
        &mut self,
        query: String,
        session: &mut Session,
        queryable: T,
    ) -> Result<()> {
        info!("received query message");
        let map = queryable.query(&query).await?;
        info!("query returned {} rows", map.len());
        let query_result = BartosToBartoCli::Query(map);
        let encoded = encode_to_vec(&query_result, standard())?;
        session.binary(encoded).await?;
        Ok(())
    }
}
