// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::BTreeMap;

use actix_web::web::Data;
use anyhow::Result;
use bon::Builder;
use libbarto::{
    CliUpdateKind, FailedOutput, ListOutput, OffsetDataTimeWrapper, OutputTableName, UpdateKind,
};
use sqlx::{Column, MySqlPool, Row};
use time::{
    OffsetDateTime,
    macros::{offset, time},
};
use tracing::info;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{
        Queryable,
        utils::{cachyos_filter, garuda_filter, pacman_filter},
    },
};

#[derive(Builder, Clone, Debug)]
pub(crate) struct MySqlHandler {
    pool: Data<MySqlPool>,
}

impl MySqlHandler {
    async fn delete_output_data(&self) -> Result<(u64, u64)> {
        let midnight = Self::midnight()?;
        let output_count = sqlx::query!("DELETE FROM output WHERE timestamp < ?", midnight)
            .execute(self.pool.as_ref())
            .await?
            .rows_affected();
        let exit_status_count =
            sqlx::query!("DELETE FROM exit_status WHERE timestamp < ?", midnight)
                .execute(self.pool.as_ref())
                .await?
                .rows_affected();
        Ok((output_count, exit_status_count))
    }

    async fn delete_output_test_data(&self) -> Result<(u64, u64)> {
        let midnight = Self::midnight()?;
        let output_count = sqlx::query!("DELETE FROM output_test WHERE timestamp < ?", midnight)
            .execute(self.pool.as_ref())
            .await?
            .rows_affected();
        let exit_status_count =
            sqlx::query!("DELETE FROM exit_status_test WHERE timestamp < ?", midnight)
                .execute(self.pool.as_ref())
                .await?
                .rows_affected();
        Ok((output_count, exit_status_count))
    }

    async fn update_output_data_garuda(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Garuda(garuda_filter(
            self.output_data(name).await?,
        )))
    }

    async fn update_output_test_data_garuda(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Garuda(garuda_filter(
            self.output_test_data(name).await?,
        )))
    }

    async fn update_output_data_pacman(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Pacman(pacman_filter(
            &self.output_data(name).await?,
        )))
    }

    async fn update_output_test_data_pacman(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Pacman(pacman_filter(
            &self.output_test_data(name).await?,
        )))
    }

    async fn update_output_data_cachyos(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Cachyos(cachyos_filter(
            &self.output_data(name).await?,
        )))
    }

    async fn update_output_test_data_cachyos(&self, name: &str) -> Result<UpdateKind> {
        Ok(UpdateKind::Cachyos(cachyos_filter(
            &self.output_test_data(name).await?,
        )))
    }

    async fn output_data(&self, name: &str) -> Result<Vec<String>> {
        Ok(sqlx::query!(
            r#"SELECT 
  output.data 
FROM 
  output
right join
  exit_status on exit_status.cmd_uuid = output.cmd_uuid
WHERE 
  output.bartoc_name = ?
and
  exit_status.exit_code = 0
order by 
  output.timestamp"#,
            name,
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .filter_map(|r| r.data)
        .collect::<Vec<String>>())
    }

    async fn output_test_data(&self, name: &str) -> Result<Vec<String>> {
        Ok(sqlx::query!(
            r#"SELECT 
  output_test.data 
FROM 
  output_test
right join
  exit_status_test on exit_status_test.cmd_uuid = output_test.cmd_uuid
WHERE 
  output_test.bartoc_name = ?
and
  exit_status_test.exit_code = 0
order by 
  output_test.timestamp"#,
            name,
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .filter_map(|r| r.data)
        .collect::<Vec<String>>())
    }

    async fn cmd_name_data_output(&self, name: &str, cmd_name: &str) -> Result<Vec<ListOutput>> {
        let all_output = sqlx::query!(
            "SELECT
  output.timestamp,
  output.cmd_name,
  output.data,
  exit_status.exit_code,
  exit_status.success
FROM
  output
RIGHT JOIN
  exit_status ON exit_status.cmd_uuid = output.cmd_uuid
WHERE
  output.bartoc_name = ?
AND
  output.cmd_name = ?
ORDER BY
  output.timestamp",
            name,
            cmd_name
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .map(|r| {
            ListOutput::builder()
                .maybe_timestamp(r.timestamp.map(OffsetDataTimeWrapper))
                .maybe_data(r.data)
                .exit_code(r.exit_code)
                .success(r.success)
                .build()
        })
        .collect::<Vec<ListOutput>>();

        Ok(all_output)
    }

    async fn cmd_name_data_output_test(
        &self,
        name: &str,
        cmd_name: &str,
    ) -> Result<Vec<ListOutput>> {
        let all_output = sqlx::query!(
            "SELECT
  output_test.timestamp,
  output_test.cmd_name,
  output_test.data,
  exit_status_test.exit_code,
  exit_status_test.success
FROM
  output_test
RIGHT JOIN
  exit_status_test ON exit_status_test.cmd_uuid = output_test.cmd_uuid
WHERE
  output_test.bartoc_name = ?
AND
  output_test.cmd_name = ?
ORDER BY
  output_test.timestamp",
            name,
            cmd_name
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .map(|r| {
            ListOutput::builder()
                .maybe_timestamp(r.timestamp.map(OffsetDataTimeWrapper))
                .maybe_data(r.data)
                .exit_code(r.exit_code)
                .success(r.success)
                .build()
        })
        .collect::<Vec<ListOutput>>();

        Ok(all_output)
    }

    async fn failed_cmd_data_output(&self) -> Result<Vec<FailedOutput>> {
        let all_output = sqlx::query!(
            "
select
  output.timestamp,
  output.bartoc_name,
  output.cmd_name,
  output.data,
  exit_status.exit_code,
  exit_status.success
from
  output
right join
  exit_status on output.cmd_uuid = exit_status.cmd_uuid
where
  exit_code != 0"
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .map(|r| {
            FailedOutput::builder()
                .maybe_timestamp(r.timestamp.map(OffsetDataTimeWrapper))
                .maybe_bartoc_name(r.bartoc_name)
                .maybe_cmd_name(r.cmd_name)
                .maybe_data(r.data)
                .exit_code(r.exit_code)
                .success(r.success)
                .build()
        })
        .collect::<Vec<FailedOutput>>();

        Ok(all_output)
    }

    async fn failed_cmd_data_output_test(&self) -> Result<Vec<FailedOutput>> {
        let all_output = sqlx::query!(
            "
select
  output_test.timestamp,
  output_test.bartoc_name,
  output_test.cmd_name,
  output_test.data,
  exit_status_test.exit_code,
  exit_status_test.success
from
  output_test
right join
  exit_status_test on output_test.cmd_uuid = exit_status_test.cmd_uuid
where
  exit_code != 0"
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .into_iter()
        .map(|r| {
            FailedOutput::builder()
                .maybe_timestamp(r.timestamp.map(OffsetDataTimeWrapper))
                .maybe_bartoc_name(r.bartoc_name)
                .maybe_cmd_name(r.cmd_name)
                .maybe_data(r.data)
                .exit_code(r.exit_code)
                .success(r.success)
                .build()
        })
        .collect::<Vec<FailedOutput>>();

        Ok(all_output)
    }

    async fn query(&self, query: &str) -> Result<BTreeMap<usize, BTreeMap<String, String>>> {
        let results = sqlx::query(query).fetch_all(self.pool.as_ref()).await?;
        let mut map = BTreeMap::new();
        for (i, row) in results.iter().enumerate() {
            let mut row_map = BTreeMap::new();
            for (j, column) in row.columns().iter().enumerate() {
                if let Ok(value) = row.try_get::<u64, usize>(j) {
                    let _old = row_map.insert(column.name().to_string(), value.to_string());
                } else if let Ok(value) = row.try_get::<OffsetDateTime, usize>(j) {
                    let value = value.to_offset(offset!(-4));
                    let _old = row_map.insert(column.name().to_string(), value.to_string());
                } else if let Ok(value) = row.try_get::<String, usize>(j) {
                    let _old = row_map.insert(column.name().to_string(), value);
                } else if let Ok(value) = row.try_get::<Uuid, usize>(j) {
                    let _old = row_map.insert(column.name().to_string(), value.to_string());
                }
            }
            let _old = map.insert(i, row_map);
        }
        Ok(map)
    }

    fn midnight() -> Result<OffsetDateTime> {
        let now = OffsetDateTime::now_local()?;
        let midnight = now.replace_time(time!(0:0:0));
        info!("deleting records older than: {midnight}");
        Ok(midnight)
    }
}

impl Queryable for MySqlHandler {
    async fn delete_data(&self, config: &Config) -> Result<(u64, u64)> {
        match config.mariadb().output_table() {
            OutputTableName::Output => self.delete_output_data().await,
            OutputTableName::OutputTest => self.delete_output_test_data().await,
        }
    }

    async fn update_data(
        &self,
        config: &Config,
        kind: CliUpdateKind,
        name: &str,
    ) -> Result<UpdateKind> {
        match (config.mariadb().output_table(), kind) {
            (OutputTableName::Output, CliUpdateKind::Garuda) => {
                self.update_output_data_garuda(name).await
            }
            (OutputTableName::Output, CliUpdateKind::Pacman) => {
                self.update_output_data_pacman(name).await
            }
            (OutputTableName::Output, CliUpdateKind::Cachyos) => {
                self.update_output_data_cachyos(name).await
            }
            (OutputTableName::OutputTest, CliUpdateKind::Garuda) => {
                self.update_output_test_data_garuda(name).await
            }
            (OutputTableName::OutputTest, CliUpdateKind::Pacman) => {
                self.update_output_test_data_pacman(name).await
            }
            (OutputTableName::OutputTest, CliUpdateKind::Cachyos) => {
                self.update_output_test_data_cachyos(name).await
            }
            (OutputTableName::Output | OutputTableName::OutputTest, CliUpdateKind::Other) => {
                Ok(UpdateKind::Other)
            }
        }
    }

    async fn cmd_name_data(
        &self,
        config: &Config,
        name: &str,
        cmd_name: &str,
    ) -> Result<Vec<ListOutput>> {
        match config.mariadb().output_table() {
            OutputTableName::Output => self.cmd_name_data_output(name, cmd_name).await,
            OutputTableName::OutputTest => self.cmd_name_data_output_test(name, cmd_name).await,
        }
    }

    async fn failed_cmd_data(&self, config: &Config) -> Result<Vec<FailedOutput>> {
        match config.mariadb().output_table() {
            OutputTableName::Output => self.failed_cmd_data_output().await,
            OutputTableName::OutputTest => self.failed_cmd_data_output_test().await,
        }
    }

    async fn query(&self, query: &str) -> Result<BTreeMap<usize, BTreeMap<String, String>>> {
        self.query(query).await
    }
}
