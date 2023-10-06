use crate::message::SocketMessage;
use std::collections::{HashMap, HashSet};

use actix::{
    prelude::{Message, Recipient},
    Actor, Handler,
};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     //self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>, //room id  to list of users id
}

impl Actor for Lobby {
    type Context = actix::Context<Lobby>;
}

//WsConn responds to this to pipe it through to the actual client
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

//WsConn sends this to the lobby to say "put me in please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid,
    pub id: Uuid,
}

//WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub room_id: Uuid,
    pub id: Uuid,
}

//client sends this to the lobby for the lobby to echo out.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid,
}
impl ClientActorMessage {
    pub fn new_message(&self, msg: String) -> SocketMessage {
        SocketMessage {
            message_type: crate::message::MessageTypes::TEXT,
            message: msg,
            id: Some(self.id),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: SocketMessage, target_id: &Uuid) {
        if let Some(scoket_recipient) = self.sessions.get(target_id) {
            let _ = scoket_recipient.do_send(WsMessage(serde_json::to_string(&message).unwrap()));
            return;
        }
        println!(
            "Attempting to send message but couldn't find user {}",
            target_id.to_string()
        );
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            rooms: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, ctx: &mut Self::Context) -> Self::Result {
        self.rooms
            .get(&msg.room_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|conn_id| {
                self.send_message(
                    SocketMessage {
                        message_type: crate::message::MessageTypes::TEXT,
                        message: msg.msg.clone(),
                        id: None,
                    },
                    conn_id,
                )
            });
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        if self.sessions.remove(&msg.id).is_some() {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|conn_id| {
                    self.send_message(
                        SocketMessage {
                            message_type: crate::message::MessageTypes::LEAVE,
                            message: msg.id.to_string(),
                            id: None,
                        },
                        conn_id,
                    );
                });
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                    return;
                }
                self.rooms.remove(&msg.room_id);
            }
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.id);

        self.rooms
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|conn_id| {
                self.send_message(
                    SocketMessage {
                        message_type: crate::message::MessageTypes::JOIN,
                        message: msg.id.to_string(),
                        id: None,
                    },
                    conn_id,
                )
            });

        self.sessions.insert(msg.id, msg.addr);
        self.send_message(
            SocketMessage {
                message_type: crate::message::MessageTypes::ID,
                message: msg.id.to_string(),
                id: Some(msg.id),
            },
            &msg.id,
        );
    }
}
