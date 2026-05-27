// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use anyhow::{Context, Result};
use rustls::{
    RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    server::WebPkiClientVerifier,
};
use tracing::trace;

use crate::Error;

/// A trait for types that provide TLS configuration details.
pub trait TlsConfig {
    /// The certificate file path.
    fn cert_file_path(&self) -> &str;
    /// The private key file path.
    fn key_file_path(&self) -> &str;
    /// Optional path to a CA certificate whose signatures are required on client certificates.
    /// When `Some`, the server enables mutual TLS and rejects clients that do not present a
    /// valid certificate signed by this CA. Defaults to `None` (no client auth required).
    fn client_ca_cert_path(&self) -> Option<&Path> {
        None
    }
}

/// Generates a `ServerConfig` for TLS using the provided configuration.
///
/// When `config.client_ca_cert_path()` is `Some`, mutual TLS is enabled and clients must
/// present a certificate signed by the specified CA.
///
/// # Errors
/// * Returns an error if the certificate or key files cannot be read
/// * Returns an error if no valid private keys are found in the key file
/// * Returns an error if the client CA cert file cannot be read (when mTLS is enabled)
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

    let server_config = if let Some(ca_path) = config.client_ca_cert_path() {
        trace!("mTLS enabled, client CA: {}", ca_path.display());
        let client_ca_store = load_pinned_root_store(ca_path)?;
        let client_verifier = WebPkiClientVerifier::builder(Arc::new(client_ca_store)).build()?;
        ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(cert_chain, private_keys.remove(0))?
    } else {
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_keys.remove(0))?
    };

    Ok(server_config)
}

/// Loads a client certificate chain and private key from PEM files.
///
/// Used for mutual TLS: the returned values are passed to `ClientConfig::with_client_auth_cert`.
///
/// # Errors
/// * Returns an error if either file cannot be read or contains no valid data
///
pub fn load_client_cert_and_key(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_file = &mut BufReader::new(File::open(cert_path).with_context(|| Error::CertRead)?);
    let cert_chain: Vec<CertificateDer<'static>> = CertificateDer::pem_reader_iter(cert_file)
        .flatten()
        .map(CertificateDer::into_owned)
        .collect();
    if cert_chain.is_empty() {
        return Err(Error::CertRead.into());
    }

    let key_file = &mut BufReader::new(File::open(key_path).with_context(|| Error::KeyRead)?);
    let mut keys: Vec<PrivateKeyDer<'static>> = PrivateKeyDer::pem_reader_iter(key_file)
        .filter_map(Result::ok)
        .map(|k| k.clone_key())
        .collect();
    if keys.is_empty() {
        return Err(Error::NoPrivateKeys.into());
    }

    Ok((cert_chain, keys.remove(0)))
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
    use std::path::Path;

    use super::{TlsConfig, load_client_cert_and_key, load_pinned_root_store, load_tls_config};

    struct MockTlsConfig;
    struct MockEmptyKeysTlsConfig;
    struct MockMtlsTlsConfig;

    impl TlsConfig for MockTlsConfig {
        fn cert_file_path(&self) -> &'static str {
            "./testtls/onlytests.pem"
        }

        fn key_file_path(&self) -> &'static str {
            "./testtls/onlytests-key.pem"
        }
    }

    impl TlsConfig for MockEmptyKeysTlsConfig {
        fn cert_file_path(&self) -> &'static str {
            "./testtls/onlytests.pem"
        }

        fn key_file_path(&self) -> &'static str {
            "./testtls/empty-key.pem"
        }
    }

    impl TlsConfig for MockMtlsTlsConfig {
        fn cert_file_path(&self) -> &'static str {
            "./testtls/onlytests.pem"
        }

        fn key_file_path(&self) -> &'static str {
            "./testtls/onlytests-key.pem"
        }

        fn client_ca_cert_path(&self) -> Option<&Path> {
            Some(Path::new("./testtls/test-ca.pem"))
        }
    }

    #[test]
    fn test_load_tls_config() {
        assert!(load_tls_config(&MockTlsConfig).is_ok());
    }

    #[test]
    fn test_load_tls_config_empty_keys() {
        assert!(load_tls_config(&MockEmptyKeysTlsConfig).is_err());
    }

    #[test]
    fn test_load_tls_config_with_client_auth() {
        assert!(load_tls_config(&MockMtlsTlsConfig).is_ok());
    }

    #[test]
    fn test_load_pinned_root_store() {
        let store = load_pinned_root_store(Path::new("./testtls/test-ca.pem"));
        assert!(store.is_ok());
        assert!(!store.unwrap().is_empty());
    }

    #[test]
    fn test_load_pinned_root_store_missing_file() {
        assert!(load_pinned_root_store(Path::new("./testtls/nonexistent.pem")).is_err());
    }

    #[test]
    fn test_load_pinned_root_store_empty_cert() {
        // empty-key.pem has no certificates, so the store ends up empty → error
        assert!(load_pinned_root_store(Path::new("./testtls/empty-key.pem")).is_err());
    }

    #[test]
    fn test_load_client_cert_and_key() {
        let result = load_client_cert_and_key(
            Path::new("./testtls/test-client.pem"),
            Path::new("./testtls/test-client.key"),
        );
        assert!(result.is_ok());
        let (chain, _key) = result.unwrap();
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_load_client_cert_and_key_missing_cert() {
        assert!(
            load_client_cert_and_key(
                Path::new("./testtls/nonexistent.pem"),
                Path::new("./testtls/test-client.key"),
            )
            .is_err()
        );
    }

    #[test]
    fn test_load_client_cert_and_key_empty_cert() {
        assert!(
            load_client_cert_and_key(
                Path::new("./testtls/empty-key.pem"),
                Path::new("./testtls/test-client.key"),
            )
            .is_err()
        );
    }

    #[test]
    fn test_load_client_cert_and_key_missing_key() {
        assert!(
            load_client_cert_and_key(
                Path::new("./testtls/test-client.pem"),
                Path::new("./testtls/nonexistent.key"),
            )
            .is_err()
        );
    }

    #[test]
    fn test_load_client_cert_and_key_empty_key() {
        assert!(
            load_client_cert_and_key(
                Path::new("./testtls/test-client.pem"),
                Path::new("./testtls/empty-key.pem"),
            )
            .is_err()
        );
    }
}
