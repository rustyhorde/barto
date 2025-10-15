// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod stream;

#[cfg(unix)]
use std::env::var_os;
use std::{
    collections::HashMap,
    process::Stdio,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use bincode::{Decode, Encode, config::standard, encode_to_vec};
use bon::Builder;
use futures_util::{SinkExt as _, stream::SplitSink};
use libbarto::{
    Bartoc, BartocInfo, BartocWs, BartosToBartoc, Data, MissedTick, OffsetDataTimeWrapper, Output,
    OutputKind, Realtime, Status, UuidWrapper, parse_ts_ping, send_ts_ping,
};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    net::TcpStream,
    process::Command,
    select, spawn,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
    try_join,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, protocol::CloseFrame},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use uuid::Uuid;

use crate::error::Error;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Decode, Encode)]
pub(crate) enum BartocMessage {
    Close,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    BartocToBartos(BartocWs),
    BartosToBartoc(BartosToBartoc),
    Data(Data),
    RecordData(Data),
    ClientInfo(BartocInfo),
}

impl BartocMessage {
    pub(crate) fn ping(bytes: Vec<u8>) -> Self {
        Self::BartocToBartos(BartocWs::Ping(bytes))
    }

    pub(crate) fn close(cr: Option<(u16, String)>) -> Self {
        Self::BartocToBartos(BartocWs::Close(cr))
    }
}

impl TryInto<Message> for &'_ BartocMessage {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Message, Self::Error> {
        match self {
            BartocMessage::BartocToBartos(msg) => {
                let bytes = encode_to_vec(msg, standard())?;
                Ok(Message::Binary(bytes.into()))
            }
            _ => Err(Error::InvalidBartocMessage.into()),
        }
    }
}

#[derive(Builder, Debug)]
pub(crate) struct Handler {
    // Cancellation token for this session
    token: CancellationToken,
    // current heartbeat instant
    #[builder(default = Instant::now())]
    hb: Instant,
    // The start instant of this session
    #[builder(default = Instant::now())]
    origin: Instant,
    tx: UnboundedSender<BartocMessage>,
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    #[builder(default = HashMap::new())]
    rt_map: HashMap<Realtime, (String, Vec<String>)>,
    rt_monitor_handle: Option<JoinHandle<()>>,
    // the stdout queue
    data_tx: UnboundedSender<Data>,
    id: Option<UuidWrapper>,
    bartoc_name: String,
    missed_tick: Option<MissedTick>,
}

impl Handler {
    pub(crate) async fn handle_msg(&mut self, msg: BartocMessage) -> Result<()> {
        match &msg {
            BartocMessage::Close => {
                trace!("shutting down bartoc");
                Err(Error::Shutdown.into())
            }
            BartocMessage::Ping(bytes) => {
                trace!("handling ping message, sending pong");
                if let Some(dur) = parse_ts_ping(&bytes.clone().into()) {
                    trace!("ping duration: {}s", dur.as_secs_f64());
                }
                self.hb = Instant::now();
                if let Err(e) = self.send_message(Message::Pong(bytes.clone().into())).await {
                    error!("unable to send pong message to websocket: {e}");
                }
                Ok(())
            }
            BartocMessage::Pong(bytes) => {
                trace!("handling pong message");
                if let Some(dur) = parse_ts_ping(&bytes.clone().into()) {
                    trace!("pong duration: {}s", dur.as_secs_f64());
                }
                self.hb = Instant::now();
                Ok(())
            }
            BartocMessage::BartocToBartos(bts) => {
                let msg = match bts {
                    BartocWs::Close(close_reason) => {
                        trace!("sending close message to bartos");
                        Message::Close(close_reason.as_ref().map(|(code, reason)| CloseFrame {
                            code: (*code).into(),
                            reason: reason.clone().into(),
                        }))
                    }
                    BartocWs::Ping(items) => {
                        trace!("sending ping message to bartos");
                        Message::Ping(items.clone().into())
                    }
                    BartocWs::Pong(items) => {
                        trace!("sending pong message to bartos");
                        Message::Pong(items.clone().into())
                    }
                };
                if let Err(e) = self.send_message(msg).await {
                    error!("unable to send message to websocket: {e}");
                }
                Ok(())
            }
            BartocMessage::BartosToBartoc(btb) => {
                match btb {
                    BartosToBartoc::Initialize(initialize) => {
                        trace!("received initialize message from bartos");
                        let schedules = initialize.schedules().schedules();
                        let id = initialize.id().0;
                        info!("bartoc id: {id}");
                        self.id = Some(initialize.id());
                        for schedule in schedules {
                            if let Ok(rt) = Realtime::try_from(&schedule.on_calendar()[..]) {
                                info!("bartoc schedule: {rt} -> {}", schedule.cmds().join(", "));
                                *self.rt_map.entry(rt).or_default() =
                                    (schedule.name().clone(), schedule.cmds().clone());
                            } else {
                                error!(
                                    "unable to parse bartoc schedule: {}",
                                    schedule.on_calendar()
                                );
                            }
                        }
                        self.rt_monitor();
                    }
                }
                Ok(())
            }
            BartocMessage::Data(data) => {
                self.data_tx.send(data.clone())?;
                Ok(())
            }
            BartocMessage::RecordData(data) => {
                let bartoc_msg = Bartoc::Record(data.clone());
                let msg_bytes = encode_to_vec(&bartoc_msg, standard())?;
                let msg = Message::Binary(msg_bytes.into());
                if let Err(e) = self.send_message(msg).await {
                    error!("unable to send message to websocket: {e}");
                }
                Ok(())
            }
            BartocMessage::ClientInfo(ci) => {
                let bartoc_msg = Bartoc::ClientInfo(ci.clone());
                let msg_bytes = encode_to_vec(&bartoc_msg, standard())?;
                let msg = Message::Binary(msg_bytes.into());
                if let Err(e) = self.send_message(msg).await {
                    error!("unable to send message to websocket: {e}");
                }
                Ok(())
            }
        }
    }

    async fn send_message(&mut self, msg: Message) -> Result<()> {
        self.sink.send(msg).await?;
        Ok(())
    }

    pub(crate) async fn bartoc_info(&mut self) -> Result<()> {
        let info = BartocInfo::builder().build();
        let bartoc_msg = Bartoc::ClientInfo(info);
        let msg_bytes = encode_to_vec(&bartoc_msg, standard())?;
        let msg = Message::Binary(msg_bytes.into());
        if let Err(e) = self.send_message(msg).await {
            error!("unable to send message to websocket: {e}");
        }
        Ok(())
    }

    pub(crate) fn heartbeat(&mut self, client_timeout_opt: Option<u64>) {
        let mut interval = interval(HEARTBEAT_INTERVAL);
        let client_timeout = if let Some(ct) = client_timeout_opt {
            Duration::from_secs(ct)
        } else {
            CLIENT_TIMEOUT
        };
        trace!("Starting worker session heartbeat");
        let origin_c = self.origin;
        let cloned_sender = self.tx.clone();
        let cloned_token = self.token.clone();
        let _blah = spawn(async move {
            loop {
                select! {
                    () = cloned_token.cancelled() => {
                        trace!("cancellation token triggered, shutting down heartbeat");
                        break;
                    }
                    // wait until the next interval tick
                    b = interval.tick() => {
                        trace!("checking heartbeat");
                        // check heartbeat
                        if Instant::now().duration_since(Instant::from(b)) > client_timeout {
                            // heartbeat timed out so we disconnect this bartoc
                            error!("heartbeat timed out, disconnecting!");
                            cloned_token.cancel();
                            break;
                        }
                        let bytes = send_ts_ping(origin_c);
                        if let Err(e) = cloned_sender.send(BartocMessage::ping(bytes.into())) {
                            error!("unable to send heartbeat ping: {e}");
                        }
                    }
                }
            }
        });
    }

    pub(crate) fn rt_monitor(&mut self) {
        trace!("Starting bartoc realtime monitor");
        let cloned_token = self.token.clone();
        let cloned_rt_map = self.rt_map.clone();
        let cloned_tx = self.tx.clone();
        let cloned_bartoc_name = self.bartoc_name.clone();
        let cloned_missed_tick = self.missed_tick;
        if let Some(bartoc_id) = self.id {
            if let Some(handle) = &self.rt_monitor_handle {
                handle.abort();
            }
            let rt_mon_handle = spawn(async move {
                let mut interval = interval(Duration::from_secs(1));
                if let Some(missed_tick) = &cloned_missed_tick {
                    match missed_tick {
                        MissedTick::Skip => {
                            info!("setting missed tick behavior to Skip");
                            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                        }
                        MissedTick::Delay => {
                            info!("setting missed tick behavior to Delay");
                            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                        }
                        MissedTick::Burst => {
                            info!("setting missed tick behavior to Burst");
                            interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
                        }
                    }
                }

                loop {
                    select! {
                        () = cloned_token.cancelled() => {
                            trace!("cancellation token triggered, shutting down realtime monitor");
                            break;
                        }
                        _ = interval.tick() => {
                            let start = Instant::now();
                            let now = OffsetDateTime::now_utc();
                            let run_nows: Vec<Realtime> = cloned_rt_map.keys().filter(|&rt| rt.should_run(now)).cloned().collect();
                            let cmds: HashMap<String, Vec<String>> = run_nows.into_iter().filter_map(|rt| {
                                cloned_rt_map.get(&rt).map(|(name, cmds)| (name.clone(), cmds.clone()))
                            }).collect();
                            let tx_c = cloned_tx.clone();
                            let bartoc_name_c = cloned_bartoc_name.clone();
                            if !cmds.is_empty() {
                                info!("spawning {} commands after {}ns", cmds.len(),start.elapsed().as_nanos());
                            }
                            let _handle = spawn(async move {
                                for (name, cmds) in cmds {
                                    let name_c = name.clone();
                                    for cmd in cmds {
                                        let id = Uuid::new_v4();
                                        info!("running command: {name_c} ({id})");
                                        Self::run_cmd(
                                            id,
                                            bartoc_id,
                                            &bartoc_name_c,
                                            &name,
                                            &cmd,
                                            tx_c.clone()
                                        ).await.unwrap_or_else(|e| error!("unable to run command: {e}"));
                                    }
                                }
                            });
                        }
                    }
                }
            });
            self.rt_monitor_handle = Some(rt_mon_handle);
        } else {
            error!("unable to start realtime monitor without bartoc id");
        }
    }

    pub(crate) async fn run_cmd(
        id: Uuid,
        bartoc_id: UuidWrapper,
        bartoc_name: &str,
        cmd_name: &str,
        cmd_str: &str,
        tx: UnboundedSender<BartocMessage>,
    ) -> Result<()> {
        let mut cmd = Self::setup_cmd(cmd_str)?;
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or(Error::StdoutHandle)?;
        let stderr = child.stderr.take().ok_or(Error::StderrHandle)?;
        let cmd_handle = spawn(async move { child.wait().await.map_err(Into::into) });

        let stdout_tx = tx.clone();
        let bartoc_name = bartoc_name.to_string();
        let cmd_name = cmd_name.to_string();
        let bartoc_name_c = bartoc_name.clone();
        let cmd_name_c = cmd_name.clone();
        let stdout_handle = spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                let output = Output::builder()
                    .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
                    .bartoc_uuid(bartoc_id)
                    .bartoc_name(bartoc_name_c.clone())
                    .cmd_uuid(UuidWrapper(id))
                    .cmd_name(cmd_name_c.clone())
                    .kind(OutputKind::Stdout)
                    .data(line)
                    .build();
                stdout_tx.send(BartocMessage::Data(Data::Output(output)))?;
            }
            Ok(())
        });
        let stderr_tx = tx.clone();
        let stderr_handle = spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                let output = Output::builder()
                    .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
                    .bartoc_uuid(bartoc_id)
                    .bartoc_name(bartoc_name.clone())
                    .cmd_uuid(UuidWrapper(id))
                    .cmd_name(cmd_name.clone())
                    .kind(OutputKind::Stderr)
                    .data(line)
                    .build();
                stderr_tx.send(BartocMessage::Data(Data::Output(output)))?;
            }
            Ok(())
        });

        match try_join!(
            flatten(cmd_handle),
            flatten(stdout_handle),
            flatten(stderr_handle)
        ) {
            Ok((status, _stdout_res, _stderr_res)) => {
                if let Some(code) = status.code() {
                    if status.success() {
                        info!("command {id} exited successfully with code: {code}");
                    } else {
                        error!("command {id} exited with failure code: {code}");
                    }
                } else if status.success() {
                    info!("command {id} exited successfully without a code");
                } else {
                    error!("command {id} exited with failure without a code");
                }
                let status = Status::builder()
                    .cmd_uuid(UuidWrapper(id))
                    .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
                    .exit_code(status.code())
                    .success(status.success())
                    .build();
                tx.send(BartocMessage::Data(Data::Status(status)))?;
            }
            Err(e) => error!("command handling failed: {e}"),
        }
        Ok(())
    }

    #[cfg(unix)]
    fn setup_cmd(cmd_str: &str) -> Result<Command> {
        if let Some(shell_path) = var_os("SHELL") {
            let mut cmd = Command::new(shell_path);
            let _ = cmd.arg("-c");
            let _ = cmd.arg(cmd_str);
            let _ = cmd.stdout(Stdio::piped());
            let _ = cmd.stderr(Stdio::piped());
            Ok(cmd)
        } else {
            Err(Error::NoShell.into())
        }
    }

    #[cfg(windows)]
    #[allow(clippy::unnecessary_wraps)]
    fn setup_cmd(cmd_str: &str) -> Result<Command> {
        let mut cmd = Command::new("cmd");
        let _ = cmd.arg("/C");
        let _ = cmd.arg(cmd_str);
        let _ = cmd.stdout(Stdio::piped());
        let _ = cmd.stderr(Stdio::piped());
        Ok(cmd)
    }
}

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_err) => Err(anyhow!("handling failed")),
    }
}
