use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data, Json, Path, Payload},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::chat_db::ChatTable, routes::user_route::RespostaAdquirirIdSessao, socket::ChatWs,
    AppContext,
};

use super::user_route::{get_user_id, is_logged_in};

pub fn chat_scope() -> Scope {
    web::scope("/chat")
        // .service(chat_auth_route)
        .service(connect_to_chat)
        .service(create_chat_route)
        .service(get_chats_router)
}

#[get("/auth")]
async fn chat_auth_route(session: Session, app_ctx: Data<AppContext>) -> impl Responder {
    // println!("auth thread -> {:?}", thread::current().id());
    let res = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(id) = res else {
        let RespostaAdquirirIdSessao::Erro(erro) = res else {
            todo!()
        };
        return erro;
    };
    let uuid = Uuid::new_v4();

    {
        let auth_tokens = app_ctx.auth_tokens.lock();
        let Ok(mut auth_tokens) = auth_tokens else {
            return HttpResponse::InternalServerError().body("");
        };
        auth_tokens.insert(uuid, id);
        // println!("setting -> {:?}", auth_tokens.keys());
        // println!("chat auth tokens -> {:?}", auth_tokens.keys());
    }
    HttpResponse::Ok().body(uuid.to_string())
}

#[derive(Deserialize)]
pub struct ConnectChatInfo {
    pub uuid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ConnectChatQuery {
    pub auth: Uuid,
}

pub enum AuthTokenResponse {
    Ok(Option<i64>),
    Err(HttpResponse),
}

pub fn get_auth_token(app_ctx: Data<AppContext>, uuid: Uuid) -> AuthTokenResponse {
    {
        let Ok(mut auth_tokens) = app_ctx.auth_tokens.lock() else {
            return AuthTokenResponse::Err(HttpResponse::InternalServerError().body("Nao foi possivel adquirir auth_tokens"));
        };
        println!("antiga -> {:?}", auth_tokens.keys());
        let auth_token: i64;
        {
            let Some(auth_token_local) = auth_tokens.get(&uuid).clone() else {
                return AuthTokenResponse::Ok(None);
            };
            auth_token = *auth_token_local;
        }
        (*auth_tokens).remove(&uuid);
        println!("nova -> {:?}", auth_tokens.keys());
        AuthTokenResponse::Ok(Some(auth_token.clone()))
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRoute {
    pub nome: String,
}

#[get("/")]
pub async fn get_chats_router(app_ctx: Data<AppContext>) -> impl Responder {
    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Erro adquirindo db");
    };
    let Ok(chats) = db.get_chats() else {
        return HttpResponse::InternalServerError().body("Erro adquirindo chats");
    };
    HttpResponse::Ok().json(chats)
}

#[post("/create")]
pub async fn create_chat_route(
    session: Session,
    app_ctx: Data<AppContext>,
    body: Json<CreateChatRoute>,
) -> impl Responder {
    if let Err(err) = is_logged_in(&session) {
        return err;
    };
    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Error fetching db from context");
    };

    let res = db.create_chat(&body.nome);
    let Ok(chat_id) = res else {
        log::error!("Error creating chat, {:?}", res.unwrap_err());
        return HttpResponse::InternalServerError().body("Erro ao criar grupo de chat.")
    };

    let chat = db.get_chat(chat_id);
    let Ok(chat) = chat else {
        let err = chat.unwrap_err();
        return HttpResponse::InternalServerError().body(err.to_string())
    };
    HttpResponse::Ok().json(chat)
}

#[get("/connect/{uuid}")]
pub async fn connect_to_chat(
    req: HttpRequest,
    stream: Payload,
    // srv: Data<Addr<Lobby>>,
    info: Path<ConnectChatInfo>,
    session: Session,
    app_ctx: Data<AppContext>,
) -> Result<HttpResponse, actix_web::Error> {
    let res = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(user_id) = res else {
        let RespostaAdquirirIdSessao::Erro(err) =  res else {
            todo!();
        };
        return Ok(err);
    };

    {
        let Ok(db) = app_ctx.db.lock() else {
            return Ok(HttpResponse::InternalServerError().body("Erro adquirindo db do contexto"));
        };

        if db.get_chat(info.uuid).is_err() {
            return Ok(
                HttpResponse::BadRequest().body(format!("Chat {} nao encontrado", info.uuid))
            );
        };
    }

    let ws = ChatWs::new(info.uuid, app_ctx.chat_server.clone(), user_id);
    ws::start(ws, &req, stream)
}
