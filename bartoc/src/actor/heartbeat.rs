// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::time::{Duration, Instant};

use actix::{ActorContext as _, AsyncContext as _, Context};
use awc::ws::Message;
use bytes::Bytes;
use libbarto::send_ts_ping;
use tracing::{error, trace};

use crate::actor::Worker;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// Heartbeat that sends ping to the server every HEARTBEAT_INTERVAL seconds (5)
// Also check for activity from the worker in the past CLIENT_TIMEOUT seconds (10)
pub(crate) fn heart_beat(origin_instant: Instant, ctx: &mut Context<Worker>) {
    trace!("Starting worker session heartbeat");
    _ = ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
        trace!("checking heartbeat");
        // check heartbeat
        if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
            // heartbeat timed out
            error!("heartbeat timed out, disconnecting!");

            // stop actor
            ctx.stop();

            // don't try to send a ping
            return;
        }
        trace!("sending heartbeat ping");
        let bytes = send_ts_ping(origin_instant);
        if let Err(e) = act
            .addr
            .write(Message::Ping(Bytes::copy_from_slice(&bytes)))
        {
            error!("unable to send ping: {e:?}");
        }
    });
}
