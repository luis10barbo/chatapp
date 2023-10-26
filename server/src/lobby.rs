use crate::{
    db::{
        chat_message_db::{ChatMessage, ChatMessagesTable, InsertChatMessage},
        Database,
    },
    message::{format_date, SocketMessage},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::{
    prelude::{Message, Recipient},
    Actor, ActorContext, AsyncContext, Handler,
};
use chrono::Utc;
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<i64, Socket>,       //self id to self
    rooms: HashMap<String, HashSet<i64>>, //room id  to list of users id
    db: Arc<Mutex<Database>>,
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
    pub room_id: String,
    pub id: i64,
}

//WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub room_id: String,
    pub id: i64,
}

//client sends this to the lobby for the lobby to echo out.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: i64,
    pub msg: String,
    pub room_id: String,
}
impl ClientActorMessage {
    pub fn new_message(&self, msg: String) -> SocketMessage {
        SocketMessage {
            message_type: crate::message::MessageType::TEXT,
            message: msg,
            id: Some(self.id),
            ..Default::default()
        }
    }
}

impl Lobby {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            rooms: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
    fn send_message(&self, message: SocketMessage, target_id: &i64) {
        if let Some(scoket_recipient) = self.sessions.get(target_id) {
            let _ = scoket_recipient.do_send(WsMessage(serde_json::to_string(&message).unwrap()));
            return;
        }
        println!(
            "Attempting to send {:?} but couldn't find user {}",
            message,
            target_id.to_string()
        );
    }
}

// impl Default for Lobby {
//     fn default() -> Self {
//         Self {
//             rooms: HashMap::new(),
//             sessions: HashMap::new(),

//         }
//     }
// }

pub struct ChannelDeleted {
    pub user_id: i64,
    pub room_id: String,
}

// impl Handler<ChannelDeleted> for Lobby {
//     type Result;

//     fn handle(&mut self, msg: ChannelDeleted, ctx: &mut Self::Context) -> Self::Result {

//     }
// }

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Self::Context) -> Self::Result {
        let db = self.db.lock().unwrap();
        if let Err(err) = db.insert_message(InsertChatMessage {
            chat_id: msg.room_id.to_string(),
            date_created: format_date(Utc::now()),
            message: &msg.msg,
            user_id: msg.id,
        }) {
            log::error!("Error sending message to db {:?}", err);
            if let Some(code) = err.sqlite_error_code() {
                self.send_message(
                    SocketMessage {
                        message_type: crate::message::MessageType::CHAT_DELETED,
                        message: format!("Chat {:?} foi deletado!", msg.room_id),
                        id: None,
                        date: format_date(Utc::now()),
                    },
                    &msg.id,
                );
            }
            // self.send_message(ChannelDeleted, target_id)
            return ();
        };
        self.rooms
            .get(&msg.room_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|conn_id| {
                self.send_message(
                    SocketMessage {
                        message_type: crate::message::MessageType::TEXT,
                        message: msg.msg.clone(),
                        id: Some(msg.id),
                        ..Default::default()
                    },
                    conn_id,
                )
            });
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        println!("Disconnecting user");
        if self.sessions.remove(&msg.id).is_some() {
            let Some(room) =  self.rooms.get(&msg.room_id) else {
                    println!("Could not find lobby {}", &msg.room_id);
                    return ();
            };
            println!("Disconnecting");

            room.iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|conn_id| {
                    self.send_message(
                        SocketMessage {
                            message_type: crate::message::MessageType::LEAVE,
                            message: msg.id.to_string(),
                            id: Some(msg.id),
                            ..Default::default()
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
        println!("conectando {} ao lobby {}", msg.id, msg.room_id);
        self.rooms
            .entry(msg.room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.id);

        self.rooms
            .get(&msg.room_id)
            .unwrap()
            .iter()
            // .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|conn_id| {
                println!("curr conn: {}, msgid: {}", conn_id, &msg.id);
                if conn_id.to_owned() == msg.id {
                    self.sessions.keys();
                    // self.send_message(
                    //     SocketMessage {
                    //         message_type: crate::message::MessageType::DISCONNECTED,
                    //         message: "Logado de outra localizacao".into(),
                    //         id: Some(msg.id),
                    //         ..Default::default()
                    //     },
                    //     &msg.id,
                    // );
                    // ctx.cancel_future(ctx.handle());
                    return;
                }

                self.send_message(
                    SocketMessage {
                        message_type: crate::message::MessageType::JOIN,
                        message: msg.id.to_string(),
                        id: Some(msg.id),
                        ..Default::default()
                    },
                    conn_id,
                )
            });

        self.sessions.insert(msg.id, msg.addr);

        println!("{:?}", self.sessions.keys());
        self.send_message(
            SocketMessage {
                message_type: crate::message::MessageType::INIT,
                message: self.rooms.get(&msg.room_id).unwrap().len().to_string(),
                id: Some(msg.id),
                ..Default::default()
            },
            &msg.id,
        );
    }
}
