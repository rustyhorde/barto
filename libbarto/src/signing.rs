// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Signer, Verifier};
pub use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::Error;

/// Parse a base64-encoded Ed25519 signing (private) key.
///
/// The key must be a base64-encoded 32-byte seed.
///
/// # Errors
/// Returns [`Error::InvalidKey`] if the string is not valid base64 or not exactly 32 bytes.
///
pub fn parse_signing_key(b64: &str) -> Result<SigningKey> {
    let bytes = STANDARD.decode(b64).map_err(|_| Error::InvalidKey)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| Error::InvalidKey)?;
    Ok(SigningKey::from_bytes(&bytes))
}

/// Parse a base64-encoded Ed25519 verifying (public) key.
///
/// The key must be a base64-encoded 32-byte compressed point.
///
/// # Errors
/// Returns [`Error::InvalidKey`] if the string is not valid base64, not 32 bytes, or not a valid
/// point on the Ed25519 curve.
///
pub fn parse_verifying_key(b64: &str) -> Result<VerifyingKey> {
    let bytes = STANDARD.decode(b64).map_err(|_| Error::InvalidKey)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| Error::InvalidKey)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| Error::InvalidKey.into())
}

/// Return the base64-encoded public key corresponding to a signing key.
#[must_use]
pub fn public_key_b64(key: &SigningKey) -> String {
    STANDARD.encode(key.verifying_key().as_bytes())
}

/// Return a short fingerprint of a verifying key: first 8 bytes of its SHA-256 hash, hex-encoded.
///
/// Safe to log — reveals no key material.
#[must_use]
pub fn key_fingerprint(key: &VerifyingKey) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let hash = Sha256::digest(key.as_bytes());
    hash[..8]
        .iter()
        .fold(String::with_capacity(16), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// Sign a payload, returning `[64-byte signature][payload]` as a `Vec<u8>`.
///
/// The signature covers exactly the payload bytes.
#[must_use]
pub fn sign_payload(key: &SigningKey, payload: &[u8]) -> Vec<u8> {
    let signature = key.sign(payload);
    let mut result = Vec::with_capacity(64 + payload.len());
    result.extend_from_slice(&signature.to_bytes());
    result.extend_from_slice(payload);
    result
}

/// Verify the 64-byte signature prefix of `data` and return the payload slice.
///
/// The data must be at least 64 bytes. The first 64 bytes are the Ed25519 signature over
/// the remaining bytes.
///
/// # Errors
/// Returns [`Error::SignatureInvalid`] if the data is too short or the signature does not verify.
///
pub fn verify_and_extract(key: &VerifyingKey, data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 64 {
        return Err(Error::SignatureInvalid.into());
    }
    let (sig_bytes, payload) = data.split_at(64);
    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| Error::SignatureInvalid)?;
    let signature = Signature::from_bytes(&sig_arr);
    key.verify(payload, &signature)
        .map_err(|_| Error::SignatureInvalid)?;
    Ok(payload.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{
        parse_signing_key, parse_verifying_key, public_key_b64, sign_payload, verify_and_extract,
    };
    use base64::{Engine, engine::general_purpose::STANDARD};
    use ed25519_dalek::SigningKey;

    fn make_keypair(seed: u8) -> (SigningKey, String, String) {
        let signing_key = SigningKey::from_bytes(&[seed; 32]);
        let sk_b64 = STANDARD.encode(signing_key.as_bytes());
        let pk_b64 = public_key_b64(&signing_key);
        (signing_key, sk_b64, pk_b64)
    }

    #[test]
    fn test_parse_signing_key_roundtrip() {
        let (_, sk_b64, _) = make_keypair(1);
        assert!(parse_signing_key(&sk_b64).is_ok());
    }

    #[test]
    fn test_parse_verifying_key_roundtrip() {
        let (_, _, pk_b64) = make_keypair(1);
        assert!(parse_verifying_key(&pk_b64).is_ok());
    }

    #[test]
    fn test_parse_signing_key_invalid_base64() {
        assert!(parse_signing_key("not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_parse_signing_key_wrong_length() {
        let too_short = STANDARD.encode([0u8; 16]);
        assert!(parse_signing_key(&too_short).is_err());
    }

    #[test]
    fn test_parse_verifying_key_invalid_base64() {
        assert!(parse_verifying_key("not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_parse_verifying_key_wrong_length() {
        let too_short = STANDARD.encode([0u8; 16]);
        assert!(parse_verifying_key(&too_short).is_err());
    }

    #[test]
    fn test_sign_and_verify() {
        let (_, sk_b64, pk_b64) = make_keypair(1);
        let sk = parse_signing_key(&sk_b64).unwrap();
        let vk = parse_verifying_key(&pk_b64).unwrap();

        let payload = b"hello bartos";
        let signed = sign_payload(&sk, payload);
        assert_eq!(signed.len(), 64 + payload.len());

        let extracted = verify_and_extract(&vk, &signed).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_verify_wrong_key() {
        let (_, sk_b64, _) = make_keypair(1);
        let (_, _, pk_b64_other) = make_keypair(2); // different seed → different key pair
        let sk = parse_signing_key(&sk_b64).unwrap();
        let vk_wrong = parse_verifying_key(&pk_b64_other).unwrap();

        let signed = sign_payload(&sk, b"payload");
        assert!(verify_and_extract(&vk_wrong, &signed).is_err());
    }

    #[test]
    fn test_verify_tampered_payload() {
        let (_, sk_b64, pk_b64) = make_keypair(1);
        let sk = parse_signing_key(&sk_b64).unwrap();
        let vk = parse_verifying_key(&pk_b64).unwrap();

        let mut signed = sign_payload(&sk, b"original payload");
        // Flip a byte in the payload portion
        let last = signed.len() - 1;
        signed[last] ^= 0xFF;
        assert!(verify_and_extract(&vk, &signed).is_err());
    }

    #[test]
    fn test_verify_too_short() {
        let (_, _, pk_b64) = make_keypair(1);
        let vk = parse_verifying_key(&pk_b64).unwrap();
        assert!(verify_and_extract(&vk, &[0u8; 32]).is_err());
    }
}
