pub mod db;
mod lobby;
pub mod logger;
pub mod message;
pub mod routes;
pub mod socket;

use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    get,
    web::Data,
    App, HttpResponse, HttpServer, Responder,
};
use db::Database;
use lobby::Lobby;
use logger::setup_logger;
use routes::{chat_route::chat_scope, user_route::user_scope};
use uuid::Uuid;

pub struct AppContext {
    db: Arc<Mutex<Database>>,
    auth_tokens: Arc<Mutex<HashMap<Uuid, i64>>>,
    chat_server: Addr<Lobby>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();
    if let Err(_) = dotenvy::dotenv() {
        panic!("Error loading dotenv");
    };
    let Ok(url_env) = env::var("URL") else {
        panic!("Variavel de url nao encontrada no env")
    };

    if let Err(err) = setup_logger() {
        panic!("Error setting up logger! {}", err);
    };
    let db = Arc::new(Mutex::new(db::get().unwrap()));
    let chat_server = Lobby::default().start();
    let auth_tokens = Arc::new(Mutex::new(HashMap::new()));
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
                db: db.clone(),
                auth_tokens: auth_tokens.clone(),
                chat_server: chat_server.clone(),
            }))
            // .app_data(Data::new(chat_server.clone()))
            .service(index)
            .service(get_uuid)
            .service(user_scope())
            .service(chat_scope())
    })
    .bind((url_env, 8080))?
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
