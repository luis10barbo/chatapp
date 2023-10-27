use std::time::Instant;

use actix::{
    fut, prelude::ContextFutureSpawner, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext,
    Handler, Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;

use crate::sockets::{WsMessage, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};

use super::lobby_actor::{ClientActorMessage, Connect, Disconnect, Lobby};

#[derive(Debug)]
pub struct ChatWs {
    id: i64,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    room: String,
}

impl ChatWs {
    pub fn new(room: String, lobby_addr: Addr<Lobby>, id: i64) -> ChatWs {
        ChatWs {
            id,
            lobby_addr,
            hb: Instant::now(),
            room,
        }
    }
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

impl Handler<WsMessage> for ChatWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}

impl Actor for ChatWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                room_id: self.room.clone(),
                id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room.clone(),
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWs {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
                id: self.id,
                msg: s.to_string(),
                room_id: self.room.clone(),
            }),
            Err(e) => panic!("{}", e),
        }
    }
}
