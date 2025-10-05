// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use redb::{Database, TableDefinition};
use tracing::info;

use crate::{
    config::Config,
    db::data::{
        bincode::Bincode,
        output::{OutputKey, OutputValue},
    },
    error::Error,
};

pub(crate) mod data;

const OUTPUT_TABLE: TableDefinition<'_, Bincode<OutputKey>, Bincode<OutputValue>> =
    TableDefinition::new("output");

#[derive(Debug)]
pub(crate) struct BartocDatabase {
    db: Database,
}

impl BartocDatabase {
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let redb_path = config.redb_path().as_ref().ok_or(Error::NoRedbPath)?;
        info!("Using redb database path: {}", redb_path.display());
        let db = Database::create(config.redb_path().as_ref().ok_or(Error::NoRedbPath)?)?;
        Ok(Self { db })
    }

    pub(crate) fn write_kv(&mut self, key: &OutputKey, value: &OutputValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(OUTPUT_TABLE)?;
            let _old = table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
