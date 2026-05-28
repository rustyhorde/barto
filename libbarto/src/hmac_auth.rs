// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hmac::{Hmac, KeyInit, Mac};
use rand::RngExt as _;
use sha2::Sha256;

use crate::Error;

type HmacSha256 = Hmac<Sha256>;

/// Number of bytes in the HMAC envelope header: 8 (timestamp) + 8 (nonce) + 32 (MAC).
pub const HMAC_HEADER_LEN: usize = 48;

/// Parse an HMAC key from a plain string — the raw UTF-8 bytes become the key.
///
/// HMAC-SHA256 accepts any key length; a sufficiently random string (e.g., from
/// `openssl rand -base64 32`) provides adequate entropy.
#[must_use]
pub fn parse_hmac_key(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Wrap `payload` in an HMAC-SHA256 authenticated envelope.
///
/// Wire format: `[8-byte timestamp_secs BE][8-byte nonce BE][32-byte HMAC-SHA256][payload]`.
/// The MAC covers `timestamp || nonce || payload`.
#[must_use]
pub fn hmac_sign(key: &[u8], payload: &[u8]) -> Vec<u8> {
    let timestamp = current_secs();
    let nonce: u64 = rand::rng().random();
    let mac = compute_mac(key, timestamp, nonce, payload);

    let mut out = Vec::with_capacity(HMAC_HEADER_LEN + payload.len());
    out.extend_from_slice(&timestamp.to_be_bytes());
    out.extend_from_slice(&nonce.to_be_bytes());
    out.extend_from_slice(&mac);
    out.extend_from_slice(payload);
    out
}

/// Verify an HMAC-SHA256 envelope and return `(payload, timestamp, nonce)`.
///
/// Rejects messages whose timestamp falls outside `window_secs` of now.
/// The MAC is verified with a constant-time comparison before the timestamp is checked.
///
/// # Errors
/// Returns [`Error::HmacInvalid`] if the data is too short or the MAC does not verify.
/// Returns [`Error::MessageExpired`] if the timestamp is outside the replay window.
pub fn hmac_verify_and_extract(
    key: &[u8],
    data: &[u8],
    window_secs: u64,
) -> Result<(Vec<u8>, u64, u64)> {
    if data.len() < HMAC_HEADER_LEN {
        return Err(Error::HmacInvalid.into());
    }
    let (ts_bytes, rest) = data.split_at(8);
    let (nonce_bytes, rest) = rest.split_at(8);
    let (mac_bytes, payload) = rest.split_at(32);

    let timestamp = u64::from_be_bytes(ts_bytes.try_into().map_err(|_| Error::HmacInvalid)?);
    let nonce = u64::from_be_bytes(nonce_bytes.try_into().map_err(|_| Error::HmacInvalid)?);

    // Verify MAC first (constant-time) before revealing timestamp information.
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| Error::HmacInvalid)?;
    mac.update(&timestamp.to_be_bytes());
    mac.update(&nonce.to_be_bytes());
    mac.update(payload);
    mac.verify_slice(mac_bytes)
        .map_err(|_| Error::HmacInvalid)?;

    if current_secs().abs_diff(timestamp) > window_secs {
        return Err(Error::MessageExpired.into());
    }

    Ok((payload.to_vec(), timestamp, nonce))
}

fn current_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn compute_mac(key: &[u8], timestamp: u64, nonce: u64, payload: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(&timestamp.to_be_bytes());
    mac.update(&nonce.to_be_bytes());
    mac.update(payload);
    mac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
    use super::{HMAC_HEADER_LEN, compute_mac, hmac_sign, hmac_verify_and_extract, parse_hmac_key};
    use crate::Error;

    fn key() -> Vec<u8> {
        parse_hmac_key("test-shared-secret")
    }

    fn make_envelope(key: &[u8], timestamp: u64, nonce: u64, payload: &[u8]) -> Vec<u8> {
        let mac = compute_mac(key, timestamp, nonce, payload);
        let mut env = Vec::new();
        env.extend_from_slice(&timestamp.to_be_bytes());
        env.extend_from_slice(&nonce.to_be_bytes());
        env.extend_from_slice(&mac);
        env.extend_from_slice(payload);
        env
    }

    #[test]
    fn test_sign_and_verify() {
        let k = key();
        let payload = b"hello bartos";
        let envelope = hmac_sign(&k, payload);
        assert_eq!(envelope.len(), HMAC_HEADER_LEN + payload.len());
        let (extracted, _ts, _nonce) = hmac_verify_and_extract(&k, &envelope, 60).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_wrong_key() {
        let k1 = parse_hmac_key("key-one");
        let k2 = parse_hmac_key("key-two");
        let envelope = hmac_sign(&k1, b"payload");
        assert!(hmac_verify_and_extract(&k2, &envelope, 60).is_err());
    }

    #[test]
    fn test_tampered_payload() {
        let k = key();
        let mut envelope = hmac_sign(&k, b"original");
        let last = envelope.len() - 1;
        envelope[last] ^= 0xFF;
        assert!(hmac_verify_and_extract(&k, &envelope, 60).is_err());
    }

    #[test]
    fn test_tampered_nonce() {
        let k = key();
        let mut envelope = hmac_sign(&k, b"payload");
        envelope[8] ^= 0xFF;
        assert!(hmac_verify_and_extract(&k, &envelope, 60).is_err());
    }

    #[test]
    fn test_tampered_mac() {
        let k = key();
        let mut envelope = hmac_sign(&k, b"payload");
        envelope[16] ^= 0xFF;
        assert!(hmac_verify_and_extract(&k, &envelope, 60).is_err());
    }

    #[test]
    fn test_tampered_timestamp() {
        let k = key();
        let mut envelope = hmac_sign(&k, b"payload");
        envelope[0] ^= 0xFF;
        assert!(hmac_verify_and_extract(&k, &envelope, 60).is_err());
    }

    #[test]
    fn test_too_short() {
        let k = key();
        assert!(hmac_verify_and_extract(&k, &[0u8; 10], 60).is_err());
    }

    #[test]
    fn test_exactly_header_len() {
        let k = key();
        // Empty payload is valid
        let envelope = hmac_sign(&k, b"");
        assert_eq!(envelope.len(), HMAC_HEADER_LEN);
        let (extracted, _ts, _nonce) = hmac_verify_and_extract(&k, &envelope, 60).unwrap();
        assert_eq!(extracted, b"");
    }

    #[test]
    fn test_expired() {
        let k = key();
        // Craft a valid envelope with a timestamp in the distant past.
        let timestamp: u64 = 1_000_000; // far in the past
        let nonce: u64 = 0xDEAD_BEEF;
        let payload = b"expired";
        let envelope = make_envelope(&k, timestamp, nonce, payload);
        let err = hmac_verify_and_extract(&k, &envelope, 60).unwrap_err();
        assert!(matches!(
            err.downcast_ref::<Error>(),
            Some(Error::MessageExpired)
        ));
    }

    #[test]
    fn test_nonce_returned() {
        let k = key();
        let envelope = hmac_sign(&k, b"data");
        let nonce_from_env = u64::from_be_bytes(envelope[8..16].try_into().unwrap());
        let (_payload, _ts, nonce) = hmac_verify_and_extract(&k, &envelope, 60).unwrap();
        assert_eq!(nonce, nonce_from_env);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let k = key();
        let e1 = hmac_sign(&k, b"same");
        let e2 = hmac_sign(&k, b"same");
        let n1 = u64::from_be_bytes(e1[8..16].try_into().unwrap());
        let n2 = u64::from_be_bytes(e2[8..16].try_into().unwrap());
        // Collision probability ≈ 2⁻⁶⁴
        assert_ne!(n1, n2);
    }
}
