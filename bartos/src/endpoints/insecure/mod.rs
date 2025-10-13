// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;
mod worker;

use actix_web::{
    HttpRequest,
    web::{ServiceConfig, get},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Name {
    name: Option<String>,
}

impl Name {
    pub(crate) fn describe(&self, request: &HttpRequest) -> String {
        let conn_info = request.connection_info();
        let ip = conn_info
            .realip_remote_addr()
            .map_or_else(Self::unknown, ToString::to_string);
        let name = self
            .name
            .as_deref()
            .map_or_else(Self::unknown, ToString::to_string);
        format!("{name} ({ip})")
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone().unwrap_or_else(Self::unknown)
    }

    pub(crate) fn ip(request: &HttpRequest) -> String {
        let conn_info = request.connection_info();
        conn_info
            .realip_remote_addr()
            .map_or_else(Self::unknown, ToString::to_string)
    }

    fn unknown() -> String {
        String::from("Unknown")
    }
}

pub(crate) fn insecure_config(cfg: &mut ServiceConfig) {
    _ = cfg
        .route("/ws/cli", get().to(cli::cli))
        .route("/ws/worker", get().to(worker::worker));
}
