// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod mysql;
mod utils;

use std::collections::BTreeMap;

use anyhow::Result;
use libbarto::{CliUpdateKind, FailedOutput, ListOutput, UpdateKind};

use crate::config::Config;

pub(crate) trait Queryable {
    async fn delete_data(&self, config: &Config) -> Result<(u64, u64)>;
    async fn update_data(
        &self,
        config: &Config,
        kind: CliUpdateKind,
        name: &str,
    ) -> Result<UpdateKind>;
    async fn cmd_name_data(
        &self,
        config: &Config,
        name: &str,
        cmd_name: &str,
    ) -> Result<Vec<ListOutput>>;
    async fn failed_cmd_data(&self, config: &Config) -> Result<Vec<FailedOutput>>;
    async fn query(&self, query: &str) -> Result<BTreeMap<usize, BTreeMap<String, String>>>;
}
