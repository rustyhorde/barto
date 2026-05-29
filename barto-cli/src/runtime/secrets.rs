// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Platform keychain management for barto client secrets.
//!
//! Backed by GNOME Keyring / `KWallet` on Linux (via `zbus` / Secret Service
//! protocol), macOS Keychain on macOS, and Windows Credential Manager on
//! Windows.

use anyhow::{Context as _, Result};
use keyring_core::Entry;

use crate::runtime::cli::SecretsSubcommand;

const SERVICE: &str = "barto";

/// Known client-side barto secrets managed via the platform keychain.
///
/// `bartos` system-service secrets (`hmac_key`, `signing_key`, `api_key`,
/// `mariadb_password`) are managed separately via `bartos-secrets-init` and
/// systemd credentials — not listed here.
const KNOWN_SECRETS: &[(&str, &str)] = &[
    (
        "BARTOC_HMAC_KEY",
        "Shared HMAC-SHA256 key (must match bartos hmac_key)",
    ),
    (
        "BARTOC_SERVER_PUBLIC_KEY",
        "Ed25519 public key to verify messages from bartos",
    ),
    (
        "BARTOC_BARTOS__API_KEY",
        "Bearer token for bartoc WebSocket connection",
    ),
    (
        "BARTO_CLI_BARTOS__API_KEY",
        "Bearer token for barto-cli WebSocket connection",
    ),
];

fn init_store() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use zbus_secret_service_keyring_store::Store;
        let store = Store::new().context("failed to connect to Secret Service")?;
        keyring_core::set_default_store(store);
    }
    #[cfg(target_os = "macos")]
    {
        use apple_native_keyring_store::keychain::Store;
        let store = Store::new().context("failed to connect to macOS Keychain")?;
        keyring_core::set_default_store(store);
    }
    #[cfg(target_os = "windows")]
    {
        use windows_native_keyring_store::Store;
        let store = Store::new().context("failed to connect to Windows Credential Manager")?;
        keyring_core::set_default_store(store);
    }
    Ok(())
}

pub(crate) fn handle(cmd: &SecretsSubcommand) -> Result<()> {
    init_store()?;
    match cmd {
        SecretsSubcommand::Set { key } => set(key),
        SecretsSubcommand::Get { key } => get(key),
        SecretsSubcommand::List => {
            list();
            Ok(())
        }
        SecretsSubcommand::Delete { key } => delete(key),
    }
}

fn set(key: &str) -> Result<()> {
    let value = rpassword::prompt_password(format!("Enter value for {key}: "))
        .with_context(|| format!("failed to read secret for {key}"))?;
    Entry::new(SERVICE, key)
        .with_context(|| format!("failed to open keychain entry for {key}"))?
        .set_password(&value)
        .with_context(|| format!("failed to store {key} in keychain"))?;
    println!("{key} stored.");
    Ok(())
}

fn get(key: &str) -> Result<()> {
    let value = Entry::new(SERVICE, key)
        .with_context(|| format!("failed to open keychain entry for {key}"))?
        .get_password()
        .with_context(|| format!("{key} not found in keychain"))?;
    println!("{value}");
    Ok(())
}

fn list() {
    for (key, desc) in KNOWN_SECRETS {
        let status = Entry::new(SERVICE, key)
            .ok()
            .and_then(|e| e.get_password().ok())
            .map_or("not set", |_| "set    ");
        println!("{key:<35} [{status}]  {desc}");
    }
}

fn delete(key: &str) -> Result<()> {
    Entry::new(SERVICE, key)
        .with_context(|| format!("failed to open keychain entry for {key}"))?
        .delete_credential()
        .with_context(|| format!("failed to delete {key} from keychain"))?;
    println!("{key} deleted.");
    Ok(())
}
