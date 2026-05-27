// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use rustls::{
    RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use tracing::trace;

use crate::Error;

/// A trait for types that provide TLS configuration details.
pub trait TlsConfig {
    /// The certificate file path.
    fn cert_file_path(&self) -> &str;
    /// The private key file path.
    fn key_file_path(&self) -> &str;
}

/// Generates a `ServerConfig` for TLS using the provided configuration.
///
/// # Errors
/// * Returns an error if the certificate or key files cannot be read
/// * Returns an error if no valid private keys are found in the key file
///
pub fn load_tls_config<C>(config: &C) -> Result<ServerConfig>
where
    C: TlsConfig,
{
    let cert_file_path = config.cert_file_path();
    let key_file_path = config.key_file_path();
    trace!("cert file path: {cert_file_path}");
    trace!("key file path: {key_file_path}");

    let cert_file =
        &mut BufReader::new(File::open(cert_file_path).with_context(|| Error::CertRead)?);

    let cert_chain: Vec<CertificateDer<'_>> = CertificateDer::pem_reader_iter(cert_file)
        .flatten()
        .collect();

    let key_file = &mut BufReader::new(File::open(key_file_path).with_context(|| Error::KeyRead)?);

    let mut private_keys: Vec<PrivateKeyDer<'_>> = PrivateKeyDer::pem_reader_iter(key_file)
        .filter_map(Result::ok)
        .collect();

    if private_keys.is_empty() {
        return Err(Error::NoPrivateKeys.into());
    }
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_keys.remove(0))?;

    Ok(config)
}

/// Loads a `RootCertStore` containing only the certificates from the given PEM file.
///
/// Used for certificate pinning: when configured, only the specified CA is trusted
/// rather than the full Mozilla root CA store.
///
/// # Errors
/// * Returns an error if the file cannot be read or contains no valid certificates
///
pub fn load_pinned_root_store(path: &Path) -> Result<RootCertStore> {
    let cert_file = &mut BufReader::new(File::open(path).with_context(|| Error::CertRead)?);
    let mut root_store = RootCertStore::empty();
    for cert in CertificateDer::pem_reader_iter(cert_file).flatten() {
        root_store.add(cert)?;
    }
    if root_store.is_empty() {
        return Err(Error::CertRead.into());
    }
    Ok(root_store)
}

#[cfg(test)]
mod test {
    use super::{TlsConfig, load_tls_config};

    struct MockTlsConfig;

    impl TlsConfig for MockTlsConfig {
        fn cert_file_path(&self) -> &'static str {
            "./testtls/onlytests.pem"
        }

        fn key_file_path(&self) -> &'static str {
            "./testtls/onlytests-key.pem"
        }
    }

    struct MockEmptyKeysTlsConfig;

    impl TlsConfig for MockEmptyKeysTlsConfig {
        fn cert_file_path(&self) -> &'static str {
            "./testtls/onlytests.pem"
        }

        fn key_file_path(&self) -> &'static str {
            "./testtls/empty-key.pem"
        }
    }

    #[test]
    fn test_load_tls_config() {
        let config = MockTlsConfig;
        let tls_config = load_tls_config(&config);
        assert!(tls_config.is_ok());
    }

    #[test]
    fn test_load_tls_config_empty_keys() {
        let config = MockEmptyKeysTlsConfig;
        let tls_config = load_tls_config(&config);
        assert!(tls_config.is_err());
    }
}
