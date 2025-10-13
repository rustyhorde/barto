// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{sync::LazyLock, time::Duration};

use anyhow::Result;
use bincode::{config::standard, decode_from_slice};
use bon::Builder;
use console::Style;
use futures_util::{StreamExt as _, stream::SplitStream};
use libbarto::BartosToBartoCli;
use tokio::{net::TcpStream, select, time::sleep};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::{info, trace};
use vergen_pretty::PrettyExt;

use crate::error::Error;

pub(crate) static BOLD_BLUE: LazyLock<Style> = LazyLock::new(|| Style::new().bold().blue());
pub(crate) static BOLD_GREEN: LazyLock<Style> = LazyLock::new(|| Style::new().bold().green());
type WsMessage = Option<std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>;
type Stream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Builder, Debug)]
pub(crate) struct Handler {
    stream: Stream,
}

impl Handler {
    pub(crate) async fn handle(&mut self) -> Result<()> {
        select! {
            () = sleep(Duration::from_secs(5)) => {},
            msg_opt_res = self.stream.next() => {
                Self::handle_message(msg_opt_res)?;
            },
        }
        Ok(())
    }

    fn handle_message(msg_opt_res: WsMessage) -> Result<()> {
        let msg = msg_opt_res.ok_or(Error::InvalidMessage)??;
        if let Message::Binary(bytes) = &msg {
            Self::handle_binary(bytes);
            Ok(())
        } else {
            Err(Error::InvalidMessage.into())
        }
    }

    fn handle_binary(bytes: &[u8]) {
        match decode_from_slice(bytes, standard()) {
            Err(e) => trace!("unable to decode binary message: {e}"),
            Ok((msg, _)) => match msg {
                BartosToBartoCli::Info(pretty_ext) => {
                    let (max_category, max_label) = Self::maxes(&pretty_ext);
                    for (category, label, value) in pretty_ext.vars() {
                        let blah = format!("{label:>max_label$} ({category:>max_category$})");
                        let key = BOLD_BLUE.apply_to(&blah);
                        let value = BOLD_GREEN.apply_to(value);
                        info!("{key}: {value}");
                    }
                }
                BartosToBartoCli::Updates(updates) => {
                    for update in updates {
                        info!("{update}");
                    }
                }
                BartosToBartoCli::Cleanup(deleted) => {
                    info!("deleted {} output rows", deleted.0);
                    info!("deleted {} exit status rows", deleted.1);
                }
                BartosToBartoCli::Clients(clients) => {
                    for (id, description) in clients {
                        info!(
                            "client {}: {}",
                            BOLD_GREEN.apply_to(id),
                            BOLD_BLUE.apply_to(description)
                        );
                    }
                }
            },
        }
    }

    fn maxes(pretty_ext: &PrettyExt) -> (usize, usize) {
        let mut max_category = 0;
        let mut max_label = 0;
        for (category, label, _) in pretty_ext.vars() {
            if category.len() > max_category {
                max_category = category.len();
            }
            if label.len() > max_label {
                max_label = label.len();
            }
        }
        (max_category, max_label)
    }
}
