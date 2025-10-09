// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use libbarto::{Bincode, Data, Output, Status};
use redb::{Database, TableDefinition};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;

use crate::{
    config::Config,
    db::data::{
        output::{OutputKey, OutputValue},
        status::{StatusKey, StatusValue},
    },
    error::Error,
    handler::BartocMessage,
};

pub(crate) mod data;

const OUTPUT_TABLE: TableDefinition<'_, Bincode<OutputKey>, Bincode<OutputValue>> =
    TableDefinition::new("output");
const STATUS_TABLE: TableDefinition<'_, Bincode<StatusKey>, Bincode<StatusValue>> =
    TableDefinition::new("status");

#[derive(Debug)]
pub(crate) struct BartocDatabase {
    bartoc_name: String,
    db: Database,
    db_tx: UnboundedSender<BartocMessage>,
}

impl BartocDatabase {
    pub(crate) fn new(config: &Config, db_tx: UnboundedSender<BartocMessage>) -> Result<Self> {
        let redb_path = config.redb_path().as_ref().ok_or(Error::NoRedbPath)?;
        trace!("Using redb database path: {}", redb_path.display());
        let db = Database::create(config.redb_path().as_ref().ok_or(Error::NoRedbPath)?)?;
        let bartoc_name = config.name().clone();
        Ok(Self {
            bartoc_name,
            db,
            db_tx,
        })
    }

    pub(crate) fn write_output(&mut self, key: &OutputKey, value: &OutputValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(OUTPUT_TABLE)?;
            let _old = table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub(crate) fn write_status(&mut self, key: &StatusKey, value: &StatusValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(STATUS_TABLE)?;
            let _old = table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub(crate) fn flush_output(&mut self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        trace!("Flushing output to bartos");
        {
            let mut table = write_txn.open_table(OUTPUT_TABLE)?;
            loop {
                match table.pop_first() {
                    Ok(Some((key, value))) => {
                        let output = Output::builder()
                            .bartoc_uuid(key.value().bartoc_id())
                            .bartoc_name(self.bartoc_name.clone())
                            .cmd_uuid(key.value().cmd_uuid())
                            .timestamp(key.value().timestamp())
                            .kind(value.value().kind())
                            .data(value.value().data().clone())
                            .build();
                        self.db_tx
                            .send(BartocMessage::RecordData(Data::Output(output)))?;
                        trace!("Flushed output record: {}", key.value());
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub(crate) fn flush_status(&mut self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        trace!("Flushing status to bartos");
        {
            let mut table = write_txn.open_table(STATUS_TABLE)?;
            loop {
                match table.pop_first() {
                    Ok(Some((key, value))) => {
                        let status = Status::builder()
                            .cmd_uuid(key.value().cmd_uuid())
                            .timestamp(value.value().timestamp())
                            .exit_code(value.value().exit_code())
                            .success(value.value().success())
                            .build();
                        self.db_tx
                            .send(BartocMessage::RecordData(Data::Status(status)))?;
                        trace!("Flushed status record: {}", key.value());
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
