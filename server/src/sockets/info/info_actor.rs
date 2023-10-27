use std::collections::HashMap;

use actix::{Actor, Handler, Message, Recipient};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{message::format_date, sockets::WsMessage};

type Socket = Recipient<WsMessage>;

#[derive(Debug, Clone)]
pub struct Info {
    sessions: HashMap<i64, Socket>,
}
impl Info {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for Info {
    type Context = actix::Context<Info>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    ChatCreated,
    ChatRemoved,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoMessage {
    pub message_type: MessageType,
    pub message: String,
    pub id: Option<i64>,
    pub date: String,
}

impl Info {
    fn send_message(&self, message: InfoMessage, target_id: &i64) {
        if let Some(scoket_recipient) = self.sessions.get(target_id) {
            let _ = scoket_recipient.do_send(WsMessage(serde_json::to_string(&message).unwrap()));
            return;
        }
        log::error!(
            "Attempting to send {:?} but couldn't find user {}",
            message,
            target_id.to_string()
        );
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatCreated {
    pub user_id: i64,
    pub room_id: String,
}

impl Handler<ChatCreated> for Info {
    type Result = ();

    fn handle(&mut self, msg: ChatCreated, _: &mut Self::Context) -> Self::Result {
        self.sessions
            .iter()
            .filter(|(user_id, _)| (*user_id).to_owned() != msg.user_id)
            .for_each(|(user_id, _)| {
                self.send_message(
                    InfoMessage {
                        message_type: MessageType::ChatCreated,
                        message: msg.room_id.clone(),
                        id: None,
                        date: format_date(Utc::now()),
                    },
                    &user_id,
                )
            })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatDeleted {
    pub user_id: i64,
    pub room_id: String,
}

impl Handler<ChatDeleted> for Info {
    type Result = ();

    fn handle(&mut self, msg: ChatDeleted, _: &mut Self::Context) -> Self::Result {
        self.sessions
            .iter()
            .filter(|(user_id, _)| (*user_id).to_owned() != msg.user_id)
            .for_each(|(user_id, _)| {
                self.send_message(
                    InfoMessage {
                        message_type: MessageType::ChatRemoved,
                        message: msg.room_id.clone(),
                        id: None,
                        date: format_date(Utc::now()),
                    },
                    &user_id,
                )
            });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id: i64,
    pub addr: Recipient<WsMessage>,
}
impl Handler<Connect> for Info {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i64,
}
impl Handler<Disconnect> for Info {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}
