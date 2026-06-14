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
use bincode_next::{config::standard, decode_from_slice};
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
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) async fn handle(&mut self) -> Result<()> {
        select! {
            () = sleep(Duration::from_secs(5)) => {},
            msg_opt_res = self.stream.next() => {
                Self::handle_message(msg_opt_res)?;
            },
        }
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) async fn wait_for_close(&mut self) {
        select! {
            () = sleep(Duration::from_millis(200)) => {
                trace!("close ack timeout");
            }
            () = async {
                while let Some(msg) = self.stream.next().await {
                    match msg {
                        Ok(Message::Close(_)) | Err(_) => break,
                        _ => {}
                    }
                }
            } => {
                trace!("close acknowledged");
            }
        }
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
                BartosToBartoCli::ClientVersions(versions) => {
                    Self::handle_client_versions(&versions);
                }
                BartosToBartoCli::Query(map) => Self::handle_query(map),
                BartosToBartoCli::List(list) => {
                    let _ = Self::handle_list(&list, false);
                }
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

    fn handle_cleanup(deleted: (u64, u64, usize)) {
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
        println!(
            "{} {} {}",
            BOLD_GREEN.apply_to("signaled"),
            BOLD_YELLOW.apply_to(deleted.2),
            BOLD_GREEN.apply_to("connected clients")
        );
    }

    fn handle_clients(clients: &HashMap<UuidWrapper, ClientData>) {
        let mut client_datas = clients.values().cloned().collect::<Vec<ClientData>>();
        client_datas.sort_by(|a, b| a.name().cmp(b.name()));
        let (max_name_label, max_ip_label, max_os_name, max_os_version, max_kernel, max_version) =
            Self::maxes_client_data(&client_datas);
        let client_count = client_datas.len();
        for cd in client_datas {
            let info_str = if let Some(info) = cd.bartoc_info() {
                format!(
                    "{:<max_os_name$} {:<max_os_version$} {:<max_kernel$} {:<max_version$}",
                    info.name(),
                    info.os_version(),
                    info.kernel_version(),
                    format!("v{}", info.version()),
                )
            } else {
                cd.name().clone()
            };
            println!(
                "{:>max_name_label$} ({:>max_ip_label$}): {}",
                BOLD_GREEN.apply_to(cd.name().clone()),
                BOLD_GREEN.apply_to(cd.ip().clone()),
                BOLD_BLUE.apply_to(info_str)
            );
        }
        println!();
        println!(
            "{} {}",
            BOLD_GREEN.apply_to("Total clients:"),
            BOLD_YELLOW.apply_to(client_count)
        );
    }

    fn handle_client_versions(versions: &BTreeMap<String, String>) {
        let max_name = versions.keys().map(String::len).max().unwrap_or(0);
        let client_count = versions.len();
        for (name, version) in versions {
            println!(
                "{:>max_name$}: {}",
                BOLD_GREEN.apply_to(name),
                BOLD_BLUE.apply_to(version)
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
        let print_height = usize::from(height).saturating_sub(8).max(1);
        'outer: for (idx, row) in results {
            let known_width = digits + max_col_label + 10;

            for (col, data) in row {
                let (mut final_data, data_uw) = clean_output_string(&data);
                let disp_data = if data_uw <= usize::from(width).saturating_sub(known_width) {
                    final_data
                } else {
                    final_data.truncate(usize::from(width).saturating_sub(known_width));
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

    fn handle_list(list: &[ListOutput], extra: bool) -> bool {
        let mut early = false;
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
                usize::from(height).saturating_sub(13).max(1)
            } else {
                usize::from(height).saturating_sub(8).max(1)
            };
            'outer: for (idx, output) in list.iter().enumerate() {
                let output = output.timestamp().zip(output.data().clone()).map_or_else(
                    String::new,
                    |(timestamp, data)| {
                        let known_width = digits + timestamp.to_string().len() + 10;
                        let (mut final_data, data_uw) = clean_output_string(&data);
                        let disp_data = if data_uw <= usize::from(width).saturating_sub(known_width)
                        {
                            final_data
                        } else {
                            final_data.truncate(usize::from(width).saturating_sub(known_width));
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
                                early = true;
                                break 'outer;
                            }
                            let _res = term.clear_last_lines(print_height + 2);
                        }
                        Err(_) => todo!(),
                    }
                }
            }
        }
        early
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
            let print_height = usize::from(height).saturating_sub(8).max(1);
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
                let disp_data = if data_uw <= usize::from(width).saturating_sub(known_width) {
                    final_data
                } else {
                    final_data.truncate(usize::from(width).saturating_sub(known_width));
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
                let early = Self::handle_list(list, true);
                let term = Term::stdout();
                let _res = term.clear_last_lines(1);
                if !early {
                    println!();
                }
                println!(
                    "{}",
                    BOLD_YELLOW
                        .apply_to("Press any key to continue to next client, 'x' to exit...")
                );
                match term.read_key() {
                    Ok(key) => {
                        if key == Key::Char('x') {
                            let _res = term.clear_last_lines(1);
                            println!("{}", BOLD_YELLOW.apply_to("Exiting..."));
                            break;
                        }
                        let _res = term.clear_screen();
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

    fn maxes_client_data(client_data: &[ClientData]) -> (usize, usize, usize, usize, usize, usize) {
        let (mut max_name_label, mut max_ip_label) = (0, 0);
        let (mut max_os_name, mut max_os_version, mut max_kernel, mut max_version) = (0, 0, 0, 0);
        for cd in client_data {
            max_name_label = max_name_label.max(cd.name().len());
            max_ip_label = max_ip_label.max(cd.ip().len());
            if let Some(info) = cd.bartoc_info() {
                max_os_name = max_os_name.max(info.name().len());
                max_os_version = max_os_version.max(info.os_version().len());
                max_kernel = max_kernel.max(info.kernel_version().len());
                max_version = max_version.max(info.version().len() + 1);
            }
        }
        (
            max_name_label,
            max_ip_label,
            max_os_name,
            max_os_version,
            max_kernel,
            max_version,
        )
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use bincode_next::{config::standard, encode_to_vec};
    use libbarto::{
        BartosToBartoCli, ClientData, FailedOutput, Garuda, ListOutput, UpdateKind, UuidWrapper,
    };
    use tokio_tungstenite::tungstenite::Message;
    use uuid::Uuid;

    use super::Handler;
    use crate::error::Error;

    fn garuda(channel: &str, package: &str) -> Garuda {
        Garuda::builder()
            .channel(channel)
            .package(package)
            .old_version("1.0")
            .new_version("2.0")
            .size_change("+1")
            .download_size("10")
            .build()
    }

    fn list_output() -> ListOutput {
        ListOutput::builder()
            .data("hello".to_string())
            .exit_code(0)
            .success(1)
            .build()
    }

    // Kept intentionally narrow: the display fns fall back to a 24-column
    // terminal width when run headless. The width math uses `saturating_sub`,
    // so wider content no longer panics, but narrow data keeps the printed
    // output predictable.
    fn failed_output() -> FailedOutput {
        FailedOutput::builder()
            .bartoc_name("h".to_string())
            .cmd_name("c".to_string())
            .data("d".to_string())
            .exit_code(1)
            .success(0)
            .build()
    }

    #[test]
    fn maxes_garuda_widths() {
        let garudas = vec![garuda("ch", "pkgname")];
        let (max_package, max_channel, ..) = Handler::maxes_garuda(&garudas);
        assert_eq!(max_package, "pkgname".len());
        assert_eq!(max_channel, "ch".len());
    }

    #[test]
    fn maxes_query_widths() {
        let mut row = BTreeMap::new();
        let _old = row.insert("column".to_string(), "value123".to_string());
        let mut map = BTreeMap::new();
        let _old = map.insert(0, row);
        let (max_col, max_val) = Handler::maxes_query(&map);
        assert_eq!(max_col, "column".len());
        assert_eq!(max_val, "value123".len());
    }

    #[test]
    fn maxes_client_data_widths() {
        let cd = ClientData::builder()
            .name("client-name".to_string())
            .ip("10.0.0.1".to_string())
            .build();
        let (max_name, max_ip, max_os_name, ..) = Handler::maxes_client_data(&[cd]);
        assert_eq!(max_name, "client-name".len());
        assert_eq!(max_ip, "10.0.0.1".len());
        assert_eq!(max_os_name, 0);
    }

    #[test]
    fn handle_message_none_is_err() {
        let res = Handler::handle_message(None);
        assert!(matches!(
            res.unwrap_err().downcast_ref::<Error>(),
            Some(Error::InvalidMessage)
        ));
    }

    #[test]
    fn handle_message_non_binary_is_err() {
        let res = Handler::handle_message(Some(Ok(Message::Text("nope".into()))));
        assert!(matches!(
            res.unwrap_err().downcast_ref::<Error>(),
            Some(Error::InvalidMessage)
        ));
    }

    #[test]
    fn handle_message_binary_is_ok() {
        let payload = encode_to_vec(BartosToBartoCli::Cleanup((1, 2, 3)), standard()).unwrap();
        let res = Handler::handle_message(Some(Ok(Message::Binary(payload.into()))));
        assert!(res.is_ok());
    }

    #[test]
    fn handle_binary_garbage_does_not_panic() {
        Handler::handle_binary(&[0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn handle_binary_dispatches_variants() {
        let mut clients = HashMap::new();
        let _old = clients.insert(
            UuidWrapper(Uuid::new_v4()),
            ClientData::builder()
                .name("host1".to_string())
                .ip("10.0.0.1".to_string())
                .build(),
        );
        let mut versions = BTreeMap::new();
        let _old = versions.insert("host1".to_string(), "1.5.11".to_string());
        let mut query = BTreeMap::new();
        let mut row = BTreeMap::new();
        let _old = row.insert("col".to_string(), "val".to_string());
        let _old = query.insert(0, row);

        let messages = vec![
            BartosToBartoCli::Cleanup((1, 2, 3)),
            BartosToBartoCli::Clients(clients),
            BartosToBartoCli::ClientVersions(versions),
            BartosToBartoCli::Query(query),
            BartosToBartoCli::List(vec![list_output()]),
            BartosToBartoCli::Failed(vec![failed_output()]),
            BartosToBartoCli::ListCommands(vec!["backup".to_string(), "restore".to_string()]),
            BartosToBartoCli::Updates(UpdateKind::Garuda(vec![garuda("ch", "pkg")])),
        ];
        for msg in messages {
            let bytes = encode_to_vec(msg, standard()).unwrap();
            Handler::handle_binary(&bytes);
        }
    }

    #[test]
    fn handle_empty_collections_do_not_panic() {
        Handler::handle_binary(&encode_to_vec(BartosToBartoCli::List(vec![]), standard()).unwrap());
        Handler::handle_binary(
            &encode_to_vec(BartosToBartoCli::Failed(vec![]), standard()).unwrap(),
        );
        Handler::handle_binary(
            &encode_to_vec(BartosToBartoCli::ListCommands(vec![]), standard()).unwrap(),
        );
    }
}
