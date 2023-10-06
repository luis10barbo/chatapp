mod lobby;
pub mod message;

use std::time::{Duration, Instant};

use actix::{
    fut, prelude::ContextFutureSpawner, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext,
    Handler, Running, StreamHandler, WrapFuture,
};
use actix_web::{
    get,
    web::{Data, Path, Payload},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use lobby::{ClientActorMessage, Connect, Disconnect, Lobby, WsMessage};
use serde::Deserialize;
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct ChatWs {
    id: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    room: Uuid,
}

impl ChatWs {
    pub fn new(room: Uuid, lobby_addr: Addr<Lobby>) -> ChatWs {
        ChatWs {
            id: Uuid::new_v4(),
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
                lobby_id: self.room,
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
            room_id: self.room,
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
                room_id: self.room,
            }),
            Err(e) => panic!("{}", e),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let chat_server = Lobby::default().start();
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(connect_to_chat)
            .service(get_uuid)
            .app_data(Data::new(chat_server.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

#[get("/uuid")]
pub async fn get_uuid() -> impl Responder {
    HttpResponse::Ok().body(Uuid::new_v4().to_string())
}

#[derive(Deserialize)]
pub struct ConnectChatInfo {
    pub uuid: Uuid,
}

#[get("/chats/{uuid}")]
pub async fn connect_to_chat(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Lobby>>,
    info: Path<ConnectChatInfo>,
) -> impl Responder {
    let ws = ChatWs::new(info.uuid, srv.get_ref().clone());
    ws::start(ws, &req, stream)
}
