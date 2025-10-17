// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{collections::BTreeMap, sync::LazyLock, time::Duration};

use anyhow::Result;
use bincode::{config::standard, decode_from_slice};
use bon::Builder;
use console::Style;
use futures_util::{StreamExt as _, stream::SplitStream};
use libbarto::{BartosToBartoCli, ClientData, Garuda, UpdateKind};
use tokio::{net::TcpStream, select, time::sleep};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::trace;
use vergen_pretty::PrettyExt;

use crate::error::Error;

pub(crate) static BOLD_BLUE: LazyLock<Style> = LazyLock::new(|| Style::new().bold().blue());
pub(crate) static BOLD_GREEN: LazyLock<Style> = LazyLock::new(|| Style::new().bold().green());
pub(crate) static BOLD_YELLOW: LazyLock<Style> = LazyLock::new(|| Style::new().bold().yellow());
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
                        println!("{key}: {value}");
                    }
                }
                BartosToBartoCli::InfoJson(json) => {
                    print!("{json}");
                }
                BartosToBartoCli::Updates(updates) => Self::handle_updates(updates),
                BartosToBartoCli::Cleanup(deleted) => {
                    println!("deleted {} output rows", deleted.0);
                    println!("deleted {} exit status rows", deleted.1);
                }
                BartosToBartoCli::Clients(clients) => {
                    let mut client_datas = clients.values().cloned().collect::<Vec<ClientData>>();
                    client_datas.sort_by(|a, b| a.name().cmp(b.name()));
                    let (max_name_label, max_ip_label) = Self::maxes_client_data(&client_datas);
                    let client_count = client_datas.len();
                    for cd in client_datas {
                        println!(
                            "{:>max_name_label$} ({:>max_ip_label$}): {}",
                            BOLD_GREEN.apply_to(cd.name().clone()),
                            BOLD_GREEN.apply_to(cd.ip().clone()),
                            BOLD_BLUE.apply_to(cd)
                        );
                    }
                    println!();
                    println!(
                        "{} {}",
                        BOLD_GREEN.apply_to("Total clients:"),
                        BOLD_YELLOW.apply_to(client_count)
                    );
                }
                BartosToBartoCli::Query(map) => {
                    let (max_col_label, _max_val_label) = Self::maxes_query(&map);
                    for (i, row) in map {
                        let row_num = i + 1;
                        println!(
                            "{} {}",
                            BOLD_YELLOW.apply_to("Row"),
                            BOLD_YELLOW.apply_to(row_num)
                        );
                        for (col, val) in row {
                            println!(
                                "{:>max_col_label$}: {}",
                                BOLD_GREEN.apply_to(col),
                                BOLD_BLUE.apply_to(val)
                            );
                        }
                    }
                }
            },
        }
    }

    fn handle_updates(updates: UpdateKind) {
        match updates {
            UpdateKind::Garuda(garudas) => {
                let (
                    max_channel,
                    max_package,
                    max_old_version,
                    max_new_version,
                    max_size_change,
                    max_download_size,
                ) = Self::maxes_garuda(&garudas);
                for garuda in &garudas {
                    println!(
                        "{:<max_channel$} ({:<max_package$}): {:<max_old_version$} -> {:<max_new_version$} ({:>max_size_change$}, {:>max_download_size$})",
                        BOLD_BLUE.apply_to(garuda.package()),
                        BOLD_BLUE.apply_to(garuda.channel()),
                        BOLD_GREEN.apply_to(garuda.old_version()),
                        BOLD_GREEN.apply_to(garuda.new_version()),
                        BOLD_GREEN.apply_to(garuda.size_change()),
                        BOLD_GREEN.apply_to(garuda.download_size())
                    );
                }
            }
            UpdateKind::Pacman(pacman) => {
                let packages = pacman.packages().join(", ");
                println!(
                    "{} ({}) {}",
                    BOLD_GREEN.apply_to("Packages"),
                    BOLD_GREEN.apply_to(pacman.update_count()),
                    BOLD_BLUE.apply_to(packages)
                );
                println!();
                println!(
                    "{}   {} {}",
                    BOLD_GREEN.apply_to("Total Download Size:"),
                    BOLD_BLUE.apply_to(pacman.download_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
                println!(
                    "{}  {} {}",
                    BOLD_GREEN.apply_to("Total Installed Size:"),
                    BOLD_BLUE.apply_to(pacman.install_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
                println!(
                    "{}      {} {}",
                    BOLD_GREEN.apply_to("Net Upgrade Size:"),
                    BOLD_BLUE.apply_to(pacman.net_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
            }
            UpdateKind::Other => {}
        }
    }

    fn maxes_garuda(garudas: &[Garuda]) -> (usize, usize, usize, usize, usize, usize) {
        let mut max_package_label = 0;
        let mut max_channel_label = 0;
        let mut max_old_version_label = 0;
        let mut max_new_version_label = 0;
        let mut max_size_change_label = 0;
        let mut max_download_size_label = 0;
        for garuda in garudas {
            if garuda.package().len() > max_package_label {
                max_package_label = garuda.package().len();
            }
            if garuda.channel().len() > max_channel_label {
                max_channel_label = garuda.channel().len();
            }
            if garuda.old_version().len() > max_old_version_label {
                max_old_version_label = garuda.old_version().len();
            }
            if garuda.new_version().len() > max_new_version_label {
                max_new_version_label = garuda.new_version().len();
            }
            if garuda.size_change().len() > max_size_change_label {
                max_size_change_label = garuda.size_change().len();
            }
            if garuda.download_size().len() > max_download_size_label {
                max_download_size_label = garuda.download_size().len();
            }
        }
        (
            max_package_label,
            max_channel_label,
            max_old_version_label,
            max_new_version_label,
            max_size_change_label,
            max_download_size_label,
        )
    }

    fn maxes_query(map: &BTreeMap<usize, BTreeMap<String, String>>) -> (usize, usize) {
        let mut max_col_label = 0;
        let mut max_val_label = 0;
        for row in map.values() {
            for (col, val) in row {
                if col.len() > max_col_label {
                    max_col_label = col.len();
                }
                if val.len() > max_val_label {
                    max_val_label = val.len();
                }
            }
        }
        (max_col_label, max_val_label)
    }

    fn maxes_client_data(client_data: &[ClientData]) -> (usize, usize) {
        let mut max_name_label = 0;
        let mut max_ip_label = 0;
        for cd in client_data {
            if cd.name().len() > max_name_label {
                max_name_label = cd.name().len();
            }
            if cd.ip().len() > max_ip_label {
                max_ip_label = cd.ip().len();
            }
        }
        (max_name_label, max_ip_label)
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
