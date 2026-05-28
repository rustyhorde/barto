// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use bincode_next::{
    config::{Configuration, standard},
    decode_from_slice,
};
use bon::Builder;
use libbarto::{BartosToBartoc, VerifyingKey, verify_and_extract};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace, warn};

use crate::handler::BartocMessage;

#[derive(Builder, Clone, Debug)]
pub(crate) struct WsHandler {
    tx: UnboundedSender<BartocMessage>,
    token: CancellationToken,
    /// Optional Ed25519 verifying key — when set, incoming binary messages must carry a valid
    /// 64-byte signature prefix. Messages that fail verification are dropped and logged.
    verifying_key: Option<VerifyingKey>,
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
                        let decode_target: Option<Vec<u8>> = if let Some(vk) = &self.verifying_key {
                            match verify_and_extract(vk, &bytes) {
                                Ok(payload) => {
                                    trace!("binary message signature verified");
                                    Some(payload)
                                }
                                Err(e) => {
                                    warn!("message signature invalid, dropping: {e}");
                                    None
                                }
                            }
                        } else {
                            Some(bytes.to_vec())
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
}
