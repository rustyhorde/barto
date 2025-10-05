// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

pub(crate) mod stream;

use std::{
    collections::HashMap,
    env::var_os,
    process::Stdio,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use bincode::{Decode, Encode, config::standard, encode_to_vec};
use bon::Builder;
use futures_util::{SinkExt as _, stream::SplitSink};
use libbarto::{BartocToBartos, BartosToBartoc, Realtime, parse_ts_ping, send_ts_ping};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    net::TcpStream,
    process::Command,
    select, spawn,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
    time::interval,
    try_join,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, protocol::CloseFrame},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use uuid::Uuid;

use crate::{
    db::data::{
        odt::OffsetDataTimeWrapper,
        output::{Output, OutputKind},
        uuid::UuidWrapper,
    },
    error::Error,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Decode, Encode)]
pub(crate) enum BartocMessage {
    Close,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    BartocToBartos(BartocToBartos),
    BartosToBartoc(BartosToBartoc),
    Output(Output),
}

impl BartocMessage {
    pub(crate) fn ping(bytes: Vec<u8>) -> Self {
        Self::BartocToBartos(BartocToBartos::Ping(bytes))
    }

    pub(crate) fn close(cr: Option<(u16, String)>) -> Self {
        Self::BartocToBartos(BartocToBartos::Close(cr))
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
    rt_map: HashMap<Realtime, Vec<String>>,
    rt_monitor_handle: Option<JoinHandle<()>>,
    // the stdout queue
    output_tx: UnboundedSender<Output>,
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
                    BartocToBartos::Close(close_reason) => {
                        trace!("sending close message to bartos");
                        Message::Close(close_reason.as_ref().map(|(code, reason)| CloseFrame {
                            code: (*code).into(),
                            reason: reason.clone().into(),
                        }))
                    }
                    BartocToBartos::Ping(items) => {
                        trace!("sending ping message to bartos");
                        Message::Ping(items.clone().into())
                    }
                    BartocToBartos::Pong(items) => {
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
                    BartosToBartoc::Initialize(schedules) => {
                        trace!("received initialize message from bartos");
                        schedules.schedules().iter().for_each(|s| {
                            if let Ok(rt) = Realtime::try_from(&s.on_calendar()[..]) {
                                info!(
                                    "bartoc schedule: {} -> {}",
                                    s.on_calendar(),
                                    s.cmds().join(", ")
                                );
                                self.rt_map.entry(rt).or_default().clone_from(s.cmds());
                            } else {
                                error!("unable to parse bartoc schedule: {}", s.on_calendar());
                            }
                        });
                        let count = schedules.schedules().len();
                        info!("bartoc {} schedules", count);
                        self.rt_monitor();
                    }
                }
                Ok(())
            }
            BartocMessage::Output(output) => {
                self.output_tx.send(output.clone())?;
                Ok(())
            }
        }
    }

    async fn send_message(&mut self, msg: Message) -> Result<()> {
        self.sink.send(msg).await?;
        Ok(())
    }

    pub(crate) fn heartbeat(&mut self) {
        let mut interval = interval(HEARTBEAT_INTERVAL);
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
                        if Instant::now().duration_since(Instant::from(b)) > CLIENT_TIMEOUT {
                            // heartbeat timed out
                            error!("heartbeat timed out, disconnecting!");

                            // TODO: notify the bartoc about the timeout and shutdown

                            // don't try to send a ping
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
        let mut interval = interval(Duration::from_secs(1));
        trace!("Starting bartoc realtime monitor");
        let _cloned_sender = self.tx.clone();
        let cloned_token = self.token.clone();
        let cloned_rt_map = self.rt_map.clone();
        let cloned_tx = self.tx.clone();
        if let Some(handle) = &self.rt_monitor_handle {
            handle.abort();
        }
        let rt_mon_handle = spawn(async move {
            loop {
                select! {
                    () = cloned_token.cancelled() => {
                        trace!("cancellation token triggered, shutting down realtime monitor");
                        break;
                    }
                    _ = interval.tick() => {
                        let now = OffsetDateTime::now_utc();
                        for (rt, cmds) in &cloned_rt_map {
                            if rt.should_run(now) {
                                info!("running commands: {}", cmds.join(", "));
                                for cmd in cmds {
                                    Self::run_cmd(cmd, cloned_tx.clone()).await.unwrap_or_else(|e| error!("unable to run command: {e}"));
                                }
                            }
                        }
                    }
                }
            }
        });
        self.rt_monitor_handle = Some(rt_mon_handle);
    }

    pub(crate) async fn run_cmd(cmd_str: &str, tx: UnboundedSender<BartocMessage>) -> Result<()> {
        if let Some(shell_path) = var_os("SHELL") {
            let uuid = Uuid::new_v4();
            let mut cmd = Command::new(shell_path);
            let _ = cmd.arg("-c");
            let _ = cmd.arg(cmd_str);
            let _ = cmd.stdout(Stdio::piped());
            let _ = cmd.stderr(Stdio::piped());
            let mut child = cmd.spawn()?;
            let stdout = child.stdout.take().ok_or(Error::StdoutHandle)?;
            let stderr = child.stderr.take().ok_or(Error::StderrHandle)?;
            let cmd_handle = spawn(async move { child.wait().await.map_err(Into::into) });

            let cloned_tx = tx.clone();
            let stdout_handle = spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Some(line) = reader.next_line().await.unwrap_or(None) {
                    let output = Output::builder()
                        .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
                        .uuid(UuidWrapper(uuid))
                        .kind(OutputKind::Stdout)
                        .data(line)
                        .build();
                    cloned_tx.send(BartocMessage::Output(output))?;
                }
                Ok(())
            });
            let stderr_handle = spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Some(line) = reader.next_line().await.unwrap_or(None) {
                    let output = Output::builder()
                        .timestamp(OffsetDataTimeWrapper(OffsetDateTime::now_utc()))
                        .uuid(UuidWrapper(uuid))
                        .kind(OutputKind::Stderr)
                        .data(line)
                        .build();
                    tx.send(BartocMessage::Output(output))?;
                }
                Ok(())
            });

            match try_join!(
                flatten(cmd_handle),
                flatten(stdout_handle),
                flatten(stderr_handle)
            ) {
                Ok((status, _stdout_res, _stderr_res)) => {
                    info!("{status}");
                }
                Err(e) => error!("command handling failed: {e}"),
            }
        } else {
            error!("no SHELL environment variable set");
        }
        Ok(())
    }
}

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_err) => Err(anyhow!("handling failed")),
    }
}
