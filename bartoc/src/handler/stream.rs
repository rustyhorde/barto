// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use bincode_next::{
    config::{Configuration, standard},
    decode_from_slice,
};
use bon::Builder;
use libbarto::{BartosToBartoc, VerifyingKey, hmac_verify_and_extract, verify_and_extract};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace, warn};

use crate::handler::BartocMessage;

const DEFAULT_REPLAY_WINDOW_SECS: u64 = 60;

#[derive(Builder, Clone, Debug)]
pub(crate) struct WsHandler {
    tx: UnboundedSender<BartocMessage>,
    token: CancellationToken,
    /// Optional Ed25519 verifying key — when set, incoming binary messages must carry a valid
    /// 64-byte signature prefix. Messages that fail verification are dropped and logged.
    verifying_key: Option<VerifyingKey>,
    /// Optional HMAC-SHA256 key — when set, the payload (after any Ed25519 unwrap) must carry
    /// a valid authenticated envelope. Messages with bad MACs, expired timestamps, or replayed
    /// nonces are dropped and logged.
    hmac_key: Option<Vec<u8>>,
    /// Replay window in seconds. Defaults to 60.
    #[builder(default = DEFAULT_REPLAY_WINDOW_SECS)]
    replay_window_secs: u64,
    /// Seen nonces within the current replay window, keyed by nonce → message timestamp.
    #[builder(default)]
    seen_nonces: HashMap<u64, u64>,
}

impl WsHandler {
    pub(crate) fn handle_msg(
        &mut self,
        opt_res_msg: Option<std::result::Result<Message, Error>>,
    ) -> Result<()> {
        if let Some(msg_res) = opt_res_msg {
            match msg_res {
                Ok(msg) => match msg {
                    Message::Text(_utf8_bytes) => error!("text message received, ignoring"),
                    Message::Binary(bytes) => {
                        // Layer 5: Ed25519 verify (outermost layer).
                        let after_ed25519: Option<Vec<u8>> = if let Some(vk) = &self.verifying_key {
                            match verify_and_extract(vk, &bytes) {
                                Ok(payload) => {
                                    trace!("binary message Ed25519 signature verified");
                                    Some(payload)
                                }
                                Err(e) => {
                                    warn!("message Ed25519 signature invalid, dropping: {e}");
                                    None
                                }
                            }
                        } else {
                            Some(bytes.to_vec())
                        };

                        // Layer 4: HMAC-SHA256 verify and replay check.
                        let decode_target: Option<Vec<u8>> = if let Some(payload) = after_ed25519 {
                            if let Some(hmac_key) = &self.hmac_key.clone() {
                                match hmac_verify_and_extract(
                                    hmac_key,
                                    &payload,
                                    self.replay_window_secs,
                                ) {
                                    Ok((inner, ts, nonce)) => {
                                        if self.check_and_record_nonce(nonce, ts) {
                                            trace!("binary message HMAC verified");
                                            Some(inner)
                                        } else {
                                            warn!("replayed nonce detected, dropping message");
                                            None
                                        }
                                    }
                                    Err(e) => {
                                        warn!("message HMAC invalid, dropping: {e}");
                                        None
                                    }
                                }
                            } else {
                                Some(payload)
                            }
                        } else {
                            None
                        };

                        if let Some(payload) = decode_target {
                            if let Ok((btb, _)) = decode_from_slice::<BartosToBartoc, Configuration>(
                                &payload,
                                standard(),
                            ) {
                                trace!("binary message received");
                                let bm = BartocMessage::BartosToBartoc(btb);
                                if let Err(e) = self.tx.send(bm) {
                                    error!("unable to send binary message to handler: {e}");
                                }
                            } else {
                                error!("unable to decode binary message, ignoring");
                            }
                        }
                    }
                    Message::Ping(bytes) => {
                        trace!("ping message received, sending pong");
                        if let Err(e) = self.tx.send(BartocMessage::Ping(bytes.into())) {
                            error!("unable to send ping message to handler: {e}");
                        }
                    }
                    Message::Pong(bytes) => {
                        trace!("pong message received");
                        if let Err(e) = self.tx.send(BartocMessage::Pong(bytes.into())) {
                            error!("unable to send pong message to handler: {e}");
                        }
                    }
                    Message::Close(close_frame) => {
                        trace!("close message received, shutting down bartoc");
                        if let Some(cf) = &close_frame {
                            let code = u16::from(cf.code);
                            if cf.reason.is_empty() {
                                trace!("close reason: code={code} no reason given");
                            } else {
                                trace!("close reason: code={code} reason={}", cf.reason);
                            }
                        } else {
                            trace!("close reason: none");
                        }
                        if let Err(e) = self.tx.send(BartocMessage::Close) {
                            error!("unable to send close message to handler: {e}");
                        }
                        self.token.cancel();
                    }
                    Message::Frame(_frame) => error!("frame message received, ignoring"),
                },
                Err(e) => {
                    error!("websocket error: {e}");
                    self.token.cancel();
                    self.tx.send(BartocMessage::Close)?;
                }
            }
        }
        Ok(())
    }

    /// Returns `true` if the nonce is fresh (not seen before), recording it for future checks.
    /// Returns `false` if the nonce has already been seen (replay detected).
    /// Also prunes nonces whose timestamps have expired from the replay window.
    fn check_and_record_nonce(&mut self, nonce: u64, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let window = self.replay_window_secs;
        self.seen_nonces
            .retain(|_, &mut ts| now.abs_diff(ts) <= window);
        if self.seen_nonces.contains_key(&nonce) {
            return false;
        }
        let _ = self.seen_nonces.insert(nonce, timestamp);
        true
    }
}
