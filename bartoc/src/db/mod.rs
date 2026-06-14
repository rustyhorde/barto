// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use libbarto::{Bincode, Data, Output, Status, midnight};
use redb::{Database, ReadableTableMetadata, TableDefinition};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::interval,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

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
    redb_path: PathBuf,
}

impl BartocDatabase {
    pub(crate) async fn monitor(
        &mut self,
        mut data_rx: UnboundedReceiver<Data>,
        mut cleanup_rx: UnboundedReceiver<()>,
        output_token: CancellationToken,
    ) -> Result<()> {
        let mut interval = interval(Duration::from_mins(1));
        loop {
            select! {
                () = output_token.cancelled() => {
                    trace!("cancellation token triggered, shutting down output handler");
                    break;
                }
                _ = cleanup_rx.recv() => {
                    if let Err(e) = self.cleanup_redb() {
                        error!("unable to clean up redb tables: {e}");
                    }
                    if let Err(e) = self.compact_redb() {
                        error!("unable to compact redb database: {e}");
                    }
                }
                rx_opt = data_rx.recv() => {
                    if let Some(data) = rx_opt {
                        match data {
                            Data::Output(output) => {
                                if let Err(e) = self.write_output(&OutputKey::from(&output), &OutputValue::from(&output)) {
                                    error!("unable to write output to database: {e}");
                                }
                            }
                            Data::Status(status) => {
                                if let Err(e) = self.write_status(&StatusKey::from(&status), &StatusValue::from(&status)) {
                                    error!("unable to write status to database: {e}");
                                }
                            }
                        }
                    }
                },
                _val = interval.tick() => {
                    if let Err(e) = self.flush_output() {
                        error!("unable to flush output table: {e}");
                    }
                    if let Err(e) = self.flush_status() {
                        error!("unable to flush status table: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn new(config: &Config, db_tx: UnboundedSender<BartocMessage>) -> Result<Self> {
        let redb_path = config.redb_path().as_ref().ok_or(Error::NoRedbPath)?;
        trace!("Using redb database path: {}", redb_path.display());
        if let Some(parent) = redb_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(redb_path)?;
        let bartoc_name = config.name().clone();
        Ok(Self {
            bartoc_name,
            db,
            db_tx,
            redb_path: redb_path.clone(),
        })
    }

    fn write_output(&mut self, key: &OutputKey, value: &OutputValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(OUTPUT_TABLE)?;
            let _old = table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn write_status(&mut self, key: &StatusKey, value: &StatusValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(STATUS_TABLE)?;
            let _old = table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn flush_output(&mut self) -> Result<()> {
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
                            .cmd_name(value.value().name().clone())
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

    fn flush_status(&mut self) -> Result<()> {
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

    /// Delete entries from both redb tables whose timestamp is older than today's midnight,
    /// mirroring the date-based cleanup `bartos` performs on its `MariaDB` tables. The `output`
    /// table is keyed by timestamp, while the `status` table keeps its timestamp in the value.
    fn cleanup_redb(&mut self) -> Result<(u64, u64)> {
        let cutoff = midnight()?;
        info!("cleaning up redb records older than: {cutoff}");
        let write_txn = self.db.begin_write()?;
        let (output_deleted, status_deleted) = {
            let mut output_table = write_txn.open_table(OUTPUT_TABLE)?;
            let before = output_table.len()?;
            output_table.retain(|key, _value| key.timestamp().0 >= cutoff)?;
            let output_deleted = before - output_table.len()?;

            let mut status_table = write_txn.open_table(STATUS_TABLE)?;
            let before = status_table.len()?;
            status_table.retain(|_key, value| value.timestamp().0 >= cutoff)?;
            let status_deleted = before - status_table.len()?;
            (output_deleted, status_deleted)
        };
        write_txn.commit()?;
        info!("deleted {output_deleted} redb output rows");
        info!("deleted {status_deleted} redb status rows");
        Ok((output_deleted, status_deleted))
    }

    /// Compact the redb file to reclaim disk space. redb uses a copy-on-write B-tree, so deleting
    /// rows only frees pages for reuse — the file never shrinks on its own. [`Database::compact`]
    /// rewrites the live pages and truncates the file, returning the freed space to the filesystem.
    fn compact_redb(&mut self) -> Result<()> {
        let before = std::fs::metadata(&self.redb_path)?.len();
        let compacted = self.db.compact()?;
        if compacted {
            let after = std::fs::metadata(&self.redb_path)?.len();
            info!("compacted redb database: {before} bytes -> {after} bytes");
        } else {
            info!("redb database already compact ({before} bytes)");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use libbarto::{OffsetDataTimeWrapper, Output, OutputKind, Status, UuidWrapper};
    use time::OffsetDateTime;
    use tokio::sync::mpsc::unbounded_channel;
    use uuid::Uuid;

    use crate::{
        config::Config,
        db::data::{
            output::{OutputKey, OutputValue},
            status::{StatusKey, StatusValue},
        },
        handler::BartocMessage,
    };

    use super::BartocDatabase;

    fn make_db() -> (
        BartocDatabase,
        tokio::sync::mpsc::UnboundedReceiver<BartocMessage>,
    ) {
        let path = std::env::temp_dir().join(format!("{}.redb", Uuid::new_v4()));
        let (db_tx, rx) = unbounded_channel::<BartocMessage>();
        let mut config = Config::default();
        let _ = config.set_redb_path(Some(path));
        let db = BartocDatabase::new(&config, db_tx).expect("BartocDatabase::new");
        (db, rx)
    }

    fn make_output_kv() -> (OutputKey, OutputValue) {
        let output = Output::builder()
            .bartoc_uuid(UuidWrapper(Uuid::new_v4()))
            .bartoc_name("test-bartoc".to_string())
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .cmd_name("test-cmd".to_string())
            .kind(OutputKind::Stdout)
            .data("output line".to_string())
            .build();
        (OutputKey::from(&output), OutputValue::from(&output))
    }

    fn make_status_kv() -> (StatusKey, StatusValue) {
        let status = Status::builder()
            .cmd_uuid(UuidWrapper(Uuid::new_v4()))
            .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
            .exit_code(Some(0))
            .success(true)
            .build();
        (StatusKey::from(&status), StatusValue::from(&status))
    }

    #[test]
    fn new_creates_database() {
        let (_db, _rx) = make_db();
    }

    #[test]
    fn write_output_succeeds() {
        let (mut db, _rx) = make_db();
        let (key, value) = make_output_kv();
        db.write_output(&key, &value).expect("write_output");
    }

    #[test]
    fn write_status_succeeds() {
        let (mut db, _rx) = make_db();
        let (key, value) = make_status_kv();
        db.write_status(&key, &value).expect("write_status");
    }

    #[test]
    fn flush_output_sends_to_channel() {
        let (mut db, mut rx) = make_db();
        let (key, value) = make_output_kv();
        db.write_output(&key, &value).expect("write_output");
        db.flush_output().expect("flush_output");
        let msg = rx.try_recv().expect("message from flush");
        assert!(matches!(msg, BartocMessage::RecordData(_)));
    }

    #[test]
    fn flush_status_sends_to_channel() {
        let (mut db, mut rx) = make_db();
        let (key, value) = make_status_kv();
        db.write_status(&key, &value).expect("write_status");
        db.flush_status().expect("flush_status");
        let msg = rx.try_recv().expect("message from flush");
        assert!(matches!(msg, BartocMessage::RecordData(_)));
    }

    #[test]
    fn flush_output_empties_table() {
        let (mut db, _rx) = make_db();
        let (key, value) = make_output_kv();
        db.write_output(&key, &value).expect("write_output");
        db.flush_output().expect("first flush");
        // A second flush on an empty table sends nothing and succeeds.
        db.flush_output().expect("second flush (empty)");
    }

    #[test]
    fn cleanup_redb_returns_zero_for_todays_records() {
        let (mut db, _rx) = make_db();
        let (key, value) = make_output_kv();
        db.write_output(&key, &value).expect("write_output");
        let (out_deleted, status_deleted) = db.cleanup_redb().expect("cleanup_redb");
        assert_eq!(out_deleted, 0);
        assert_eq!(status_deleted, 0);
    }

    #[test]
    fn compact_redb_succeeds() {
        let (mut db, _rx) = make_db();
        db.compact_redb().expect("compact_redb");
    }

    #[test]
    fn flush_output_multiple_records() {
        let (mut db, mut rx) = make_db();
        for _ in 0..3_u8 {
            let (key, value) = make_output_kv();
            db.write_output(&key, &value).expect("write_output");
        }
        db.flush_output().expect("flush_output");
        let mut count = 0_u8;
        while rx.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
