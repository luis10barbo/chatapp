use std::time::Duration;

use actix::Message;

pub mod chat;
pub mod info;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);
