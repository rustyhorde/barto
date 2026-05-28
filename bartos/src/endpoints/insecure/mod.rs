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
use subtle::ConstantTimeEq as _;

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

/// Returns `true` if the request carries a valid `Authorization: Bearer <token>` header
/// matching `expected`, using a constant-time comparison to prevent timing attacks.
/// Returns `true` unconditionally when `expected` is `None` (no auth configured).
pub(crate) fn bearer_auth_ok(request: &HttpRequest, expected: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    let Some(header_val) = request.headers().get("Authorization") else {
        return false;
    };
    let Ok(header_str) = header_val.to_str() else {
        return false;
    };
    let Some(token) = header_str.strip_prefix("Bearer ") else {
        return false;
    };
    token.as_bytes().ct_eq(expected.as_bytes()).into()
}

pub(crate) fn insecure_config(cfg: &mut ServiceConfig) {
    _ = cfg
        .route("/ws/cli", get().to(cli::cli))
        .route("/ws/worker", get().to(worker::worker));
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;

    use super::bearer_auth_ok;

    #[test]
    fn no_api_key_configured_always_passes() {
        let req = TestRequest::get().to_http_request();
        assert!(bearer_auth_ok(&req, None));
    }

    #[test]
    fn correct_bearer_token_passes() {
        let req = TestRequest::get()
            .insert_header(("Authorization", "Bearer my-secret"))
            .to_http_request();
        assert!(bearer_auth_ok(&req, Some("my-secret")));
    }

    #[test]
    fn wrong_token_rejected() {
        let req = TestRequest::get()
            .insert_header(("Authorization", "Bearer wrong-token"))
            .to_http_request();
        assert!(!bearer_auth_ok(&req, Some("my-secret")));
    }

    #[test]
    fn missing_header_rejected() {
        let req = TestRequest::get().to_http_request();
        assert!(!bearer_auth_ok(&req, Some("my-secret")));
    }

    #[test]
    fn non_bearer_scheme_rejected() {
        let req = TestRequest::get()
            .insert_header(("Authorization", "Basic my-secret"))
            .to_http_request();
        assert!(!bearer_auth_ok(&req, Some("my-secret")));
    }

    #[test]
    fn empty_token_rejected() {
        let req = TestRequest::get()
            .insert_header(("Authorization", "Bearer "))
            .to_http_request();
        assert!(!bearer_auth_ok(&req, Some("my-secret")));
    }
}
