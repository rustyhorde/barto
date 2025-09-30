// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod heartbeat;

use std::time::Instant;

use actix::{
    Actor, ActorContext, Context, Handler, StreamHandler,
    io::{SinkWrite, WriteHandler},
};
use actix_codec::Framed;
use actix_http::ws::Item;
use actix_web::web::Bytes;
use awc::{
    BoxedSocket,
    error::WsProtocolError,
    ws::{CloseReason, Codec, Frame, Message},
};
use bincode::{config::standard, encode_to_vec};
use bon::Builder;
use futures_util::stream::SplitSink;
use libbarto::{WorkerClientToWorkerSession, parse_ts_ping};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, trace};

use crate::actor::heartbeat::heart_beat;

#[derive(Builder)]
#[allow(dead_code)]
pub(crate) struct Worker {
    // current heartbeat instant
    #[builder(default = Instant::now())]
    hb: Instant,
    // The start instant of this session
    #[builder(default = Instant::now())]
    origin: Instant,
    // The addr used to send messages back to the worker session
    addr: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    // the sender for the worker client to worker session messages
    tx: UnboundedSender<WorkerClientToWorkerSession>,
}

impl Worker {
    #[allow(clippy::unused_self)]
    fn handle_binary(&mut self, _ctx: &mut Context<Self>, _bytes: &Bytes) {}

    #[allow(clippy::unused_self)]
    fn handle_text(&mut self, _bytes: &Bytes) {}

    fn handle_ping(&mut self, bytes: Bytes) {
        trace!("handling ping message");
        if let Some(dur) = parse_ts_ping(&bytes) {
            trace!("ping duration: {}s", dur.as_secs_f64());
        }
        self.hb = Instant::now();
        if let Err(e) = self.addr.write(Message::Pong(bytes)) {
            error!("unable to send pong: {e:?}");
        }
    }

    fn handle_pong(&mut self, bytes: &Bytes) {
        trace!("handling pong message");
        if let Some(dur) = parse_ts_ping(bytes) {
            trace!("pong duration: {}s", dur.as_secs_f64());
        }
        self.hb = Instant::now();
    }

    #[allow(clippy::unused_self)]
    fn handle_close(&mut self, ctx: &mut Context<Self>, _reason: Option<CloseReason>) {
        info!("server disconnected");
        ctx.stop();
    }

    #[allow(clippy::unused_self)]
    fn handle_continuation(&mut self, ctx: &mut Context<Self>, _item: Item) {
        ctx.stop();
    }
}

impl Actor for Worker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("worker actor started");
        // start heartbeat otherwise server will disconnect after 10 seconds
        heart_beat(self.origin, ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("worker actor stopped");
    }
}

impl WriteHandler<WsProtocolError> for Worker {}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for Worker {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, ctx: &mut Self::Context) {
        if let Ok(message) = msg {
            match message {
                Frame::Binary(bytes) => self.handle_binary(ctx, &bytes),
                Frame::Text(bytes) => self.handle_text(&bytes),
                Frame::Ping(bytes) => self.handle_ping(bytes),
                Frame::Pong(bytes) => self.handle_pong(&bytes),
                Frame::Close(reason) => self.handle_close(ctx, reason),
                Frame::Continuation(item) => self.handle_continuation(ctx, item),
            }
        }
    }

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("worker stream handler started");
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        info!("worker stream handler finished");
        ctx.stop();
    }
}

impl Handler<WorkerClientToWorkerSession> for Worker {
    type Result = ();

    fn handle(&mut self, msg: WorkerClientToWorkerSession, _ctx: &mut Context<Self>) {
        match encode_to_vec(&msg, standard()) {
            Ok(msg_bytes) => info!("encoded message to vec: {:?}", msg_bytes),
            Err(e) => error!("{e}"),
        }
    }
}
