// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
    time::Duration,
};

use anyhow::Result;
use bincode::{config::standard, decode_from_slice};
use bon::Builder;
use console::{Key, Style, Term};
use count_digits::CountDigits;
use futures_util::{StreamExt as _, stream::SplitStream};
use libbarto::{
    BartosToBartoCli, ClientData, FailedOutput, Garuda, ListOutput, UpdateKind, UuidWrapper,
    clean_output_string,
};
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
                BartosToBartoCli::Info(pretty_ext) => Self::handle_info(&pretty_ext),
                BartosToBartoCli::InfoJson(json) => print!("{json}"),
                BartosToBartoCli::Updates(updates) => Self::handle_updates(updates),
                BartosToBartoCli::Cleanup(deleted) => Self::handle_cleanup(deleted),
                BartosToBartoCli::Clients(clients) => Self::handle_clients(&clients),
                BartosToBartoCli::Query(map) => Self::handle_query(map),
                BartosToBartoCli::List(list) => Self::handle_list(&list, false),
                BartosToBartoCli::Failed(failed_output) => Self::handle_failed(&failed_output),
                BartosToBartoCli::ListCommands(cmds) => Self::handle_list_commands(&cmds),
                BartosToBartoCli::Cmd(cmd_output) => Self::handle_cmd_output(&cmd_output),
            },
        }
    }

    fn handle_info(pretty_ext: &PrettyExt) {
        let (max_category, max_label) = Self::maxes(pretty_ext);
        for (category, label, value) in pretty_ext.vars() {
            let blah = format!("{label:>max_label$} ({category:>max_category$})");
            let key = BOLD_BLUE.apply_to(&blah);
            let value = BOLD_GREEN.apply_to(value);
            println!("{key}: {value}");
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
            UpdateKind::Pacman(pacman) | UpdateKind::Cachyos(pacman) => {
                let packages = pacman.packages().join(", ");
                println!(
                    "{} ({}) {}",
                    BOLD_GREEN.apply_to("Packages"),
                    BOLD_GREEN.apply_to(pacman.update_count()),
                    BOLD_BLUE.apply_to(packages)
                );
                println!();
                println!(
                    "{}   {:<4.2} {}",
                    BOLD_GREEN.apply_to("Total Download Size:"),
                    BOLD_BLUE.apply_to(pacman.download_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
                println!(
                    "{}  {:<4.2} {}",
                    BOLD_GREEN.apply_to("Total Installed Size:"),
                    BOLD_BLUE.apply_to(pacman.install_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
                println!(
                    "{}      {:<4.2} {}",
                    BOLD_GREEN.apply_to("Net Upgrade Size:"),
                    BOLD_BLUE.apply_to(pacman.net_size()),
                    BOLD_BLUE.apply_to("MiB")
                );
            }
            UpdateKind::Apt(apt) => {
                for line in apt {
                    println!("{}", BOLD_BLUE.apply_to(line));
                }
            }
        }
    }

    fn handle_cleanup(deleted: (u64, u64)) {
        println!(
            "{} {} {}",
            BOLD_GREEN.apply_to("deleted"),
            BOLD_YELLOW.apply_to(deleted.0),
            BOLD_GREEN.apply_to("output rows")
        );
        println!(
            "{} {} {}",
            BOLD_GREEN.apply_to("deleted"),
            BOLD_YELLOW.apply_to(deleted.1),
            BOLD_GREEN.apply_to("exit status rows")
        );
    }

    fn handle_clients(clients: &HashMap<UuidWrapper, ClientData>) {
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

    fn handle_query(results: BTreeMap<usize, BTreeMap<String, String>>) {
        let (max_col_label, _max_val_label) = Self::maxes_query(&results);
        println!(
            "{} {}",
            BOLD_GREEN.apply_to("Total outputs:"),
            BOLD_YELLOW.apply_to(results.len())
        );
        println!();
        let total = results.len();
        let digits = total.count_digits();
        let term = Term::stdout();
        let (height, width) = term.size_checked().unwrap_or((80, 24));
        let print_height = usize::from(height) - 8;
        'outer: for (idx, row) in results {
            let known_width = digits + max_col_label + 10;

            for (col, data) in row {
                let (mut final_data, data_uw) = clean_output_string(&data);
                let disp_data = if data_uw <= usize::from(width) - known_width {
                    final_data
                } else {
                    final_data.truncate(usize::from(width) - known_width);
                    final_data.push_str(" ...");
                    final_data
                };
                println!(
                    "{:>digits$} - {:>max_col_label$}: {}",
                    BOLD_GREEN.apply_to(idx + 1),
                    BOLD_GREEN.apply_to(col),
                    BOLD_BLUE.apply_to(disp_data)
                );
            }
            if idx > 0 && (idx + 1) % print_height == 0 {
                println!();
                println!(
                    "{}",
                    BOLD_YELLOW.apply_to("Press any key to continue, 'x' to exit...")
                );
                match term.read_key() {
                    Ok(key) => {
                        if key == Key::Char('x') {
                            let _res = term.clear_last_lines(1);
                            println!("{}", BOLD_YELLOW.apply_to("Exiting..."));
                            break 'outer;
                        }
                        let _res = term.clear_last_lines(print_height + 2);
                    }
                    Err(_) => todo!(),
                }
            }
        }
    }

    fn handle_list(list: &[ListOutput], extra: bool) {
        if list.is_empty() {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total outputs:"),
                BOLD_YELLOW.apply_to(0)
            );
        } else {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total outputs:"),
                BOLD_YELLOW.apply_to(list.len())
            );
            println!(
                "{}: {}\n{}: {}",
                BOLD_GREEN.apply_to("Exit Status"),
                BOLD_BLUE.apply_to(list[0].exit_code()),
                BOLD_GREEN.apply_to("Success"),
                BOLD_BLUE.apply_to(list[0].success())
            );
            println!();
            let total = list.len();
            let digits = total.count_digits();
            let term = Term::stdout();
            let (height, width) = term.size_checked().unwrap_or((80, 24));
            let print_height = if extra {
                usize::from(height) - 13
            } else {
                usize::from(height) - 8
            };
            'outer: for (idx, output) in list.iter().enumerate() {
                let output = output.timestamp().zip(output.data().clone()).map_or_else(
                    String::new,
                    |(timestamp, data)| {
                        let known_width = digits + timestamp.to_string().len() + 10;
                        let (mut final_data, data_uw) = clean_output_string(&data);
                        let disp_data = if data_uw <= usize::from(width) - known_width {
                            final_data
                        } else {
                            final_data.truncate(usize::from(width) - known_width);
                            final_data.push_str(" ...");
                            final_data
                        };
                        format!(
                            "{:>digits$} - {}: {}",
                            BOLD_GREEN.apply_to(idx + 1),
                            BOLD_GREEN.apply_to(timestamp),
                            BOLD_BLUE.apply_to(disp_data)
                        )
                    },
                );
                println!("{output}");
                if idx > 0 && (idx + 1) % print_height == 0 {
                    println!();
                    if extra {
                        println!(
                            "{}",
                            BOLD_YELLOW.apply_to(
                                "Press any key to continue, 'x' to move to next client..."
                            )
                        );
                    } else {
                        println!(
                            "{}",
                            BOLD_YELLOW.apply_to("Press any key to continue, 'x' to exit...")
                        );
                    }
                    match term.read_key() {
                        Ok(key) => {
                            if key == Key::Char('x') {
                                let _res = term.clear_last_lines(1);
                                println!("{}", BOLD_YELLOW.apply_to("Exiting..."));
                                break 'outer;
                            }
                            let _res = term.clear_last_lines(print_height + 2);
                        }
                        Err(_) => todo!(),
                    }
                }
            }
        }
    }

    fn handle_failed(failed_output: &[FailedOutput]) {
        let (max_bartoc_name, max_cmd_name) = {
            let mut max_bartoc_name = 0;
            let mut max_cmd_name = 0;
            for output in failed_output {
                if let Some(bartoc_name) = output.bartoc_name()
                    && bartoc_name.len() > max_bartoc_name
                {
                    max_bartoc_name = bartoc_name.len();
                }
                if let Some(cmd_name) = output.cmd_name()
                    && cmd_name.len() > max_cmd_name
                {
                    max_cmd_name = cmd_name.len();
                }
            }
            (max_bartoc_name, max_cmd_name)
        };
        if failed_output.is_empty() {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total failed outputs:"),
                BOLD_YELLOW.apply_to(0)
            );
        } else {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total failed outputs:"),
                BOLD_YELLOW.apply_to(failed_output.len())
            );
            println!();
            let total = failed_output.len();
            let digits = total.count_digits();
            let term = Term::stdout();
            let (height, width) = term.size_checked().unwrap_or((80, 24));
            let print_height = usize::from(height) - 8;
            'outer: for (idx, output) in failed_output.iter().enumerate() {
                let timestamp = output
                    .timestamp()
                    .as_ref()
                    .map_or("None".to_string(), |t| t.0.to_string());
                let bartoc_name = output.bartoc_name().as_ref().map_or("None", String::as_str);
                let cmd_name = output.cmd_name().as_ref().map_or("None", String::as_str);
                let data = output
                    .data()
                    .as_ref()
                    .map_or("None", String::as_str)
                    .to_string();
                let _exit_code = output.exit_code();
                let _success = output.success();

                let known_width = digits + timestamp.len() + max_bartoc_name + max_cmd_name + 12;
                let (mut final_data, data_uw) = clean_output_string(&data);
                let disp_data = if data_uw <= usize::from(width) - known_width {
                    final_data
                } else {
                    final_data.truncate(usize::from(width) - known_width);
                    final_data.push_str(" ...");
                    final_data
                };
                println!(
                    "{:>digits$} - {}: {:<max_bartoc_name$} {:<max_cmd_name$} {}",
                    BOLD_GREEN.apply_to(idx + 1),
                    BOLD_GREEN.apply_to(timestamp),
                    BOLD_YELLOW.apply_to(bartoc_name),
                    BOLD_YELLOW.apply_to(cmd_name),
                    BOLD_BLUE.apply_to(disp_data),
                );
                if idx > 0 && (idx + 1) % print_height == 0 {
                    println!();
                    println!(
                        "{}",
                        BOLD_YELLOW.apply_to("Press any key to continue, 'x' to exit...")
                    );
                    match term.read_key() {
                        Ok(key) => {
                            if key == Key::Char('x') {
                                let _res = term.clear_last_lines(1);
                                println!("{}", BOLD_YELLOW.apply_to("Exiting..."));
                                break 'outer;
                            }
                            let _res = term.clear_last_lines(print_height + 2);
                        }
                        Err(_) => todo!(),
                    }
                }
            }
        }
    }

    fn handle_list_commands(cmds: &[String]) {
        if cmds.is_empty() {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total commands:"),
                BOLD_YELLOW.apply_to(0)
            );
        } else {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total commands:"),
                BOLD_YELLOW.apply_to(cmds.len())
            );
            println!();
            for cmd in cmds {
                println!("{}", BOLD_BLUE.apply_to(cmd));
            }
        }
    }

    fn handle_cmd_output(cmd_output: &BTreeMap<String, Vec<ListOutput>>) {
        if cmd_output.is_empty() {
            println!(
                "{} {}",
                BOLD_GREEN.apply_to("Total outputs:"),
                BOLD_YELLOW.apply_to(0)
            );
        } else {
            for (bartoc_name, list) in cmd_output {
                println!("{}",
                    BOLD_BLUE.apply_to("################################################################################")
                );
                println!("{}", BOLD_BLUE.apply_to("#"));
                println!(
                    "#  {} {}",
                    BOLD_GREEN.apply_to("Bartoc Name:"),
                    BOLD_YELLOW.apply_to(bartoc_name)
                );
                println!("{}", BOLD_BLUE.apply_to("#"));
                println!("{}",
                    BOLD_BLUE.apply_to("################################################################################")
                );
                println!();
                Self::handle_list(list, true);
                let term = Term::stdout();
                let (height, _width) = term.size_checked().unwrap_or((80, 24));
                let print_height = usize::from(height) - 13;
                let _res = term.clear_last_lines(1);
                println!(
                    "{}",
                    BOLD_YELLOW.apply_to("Press any key to continue, 'x' to exit...")
                );
                match term.read_key() {
                    Ok(key) => {
                        if key == Key::Char('x') {
                            let _res = term.clear_last_lines(1);
                            println!("{}", BOLD_YELLOW.apply_to("Exiting..."));
                            break;
                        }
                        let _res = term.clear_last_lines(print_height + 2);
                    }
                    Err(_) => todo!(),
                }
            }
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
