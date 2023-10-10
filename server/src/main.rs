pub mod db;
mod lobby;
pub mod logger;
pub mod message;
pub mod routes;
pub mod socket;

use std::sync::{Arc, Mutex};

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key},
    get,
    web::{Data, Path, Payload},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use db::Database;
use lobby::Lobby;
use logger::setup_logger;
use routes::user_route::user_scope;
use serde::Deserialize;
use uuid::Uuid;

use crate::{routes::user_route::adquirir_id_sessao, socket::ChatWs};

struct AppContext {
    db: Arc<Mutex<Database>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();
    if let Err(err) = setup_logger() {
        panic!("Error setting up logger! {}", err);
    };
    let chat_server = Lobby::default().start();

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(2)))
                    .cookie_name("ssid".into())
                    .cookie_secure(false)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .cookie_http_only(true)
                    .build(),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(actix_web::http::header::ACCEPT)
                    .allowed_header(actix_web::http::header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(Data::new(AppContext {
                db: Arc::new(Mutex::new(db::get().unwrap())),
            }))
            .app_data(Data::new(chat_server.clone()))
            .service(index)
            .service(connect_to_chat)
            .service(get_uuid)
            .service(user_scope())
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
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let id_usuario = adquirir_id_sessao(&session);
    println!("{:?}", session.entries());
    println!("{:?}", req.cookies());
    if id_usuario.is_err() {
        return Ok(HttpResponse::NotFound().body("Sessao nao encontrada!"));
    }
    let id_usuario = id_usuario.unwrap();
    if id_usuario.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Usuario nao logado"));
    }
    let ws = ChatWs::new(info.uuid, srv.get_ref().clone(), id_usuario.unwrap());
    ws::start(ws, &req, stream)
}
