use std::thread;

use actix_session::Session;
use actix_web::{
    get,
    web::{self, Data, Path, Payload},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::{routes::user_route::RespostaAdquirirIdSessao, socket::ChatWs, AppContext};

use super::user_route::get_user_id;

pub fn chat_scope() -> Scope {
    web::scope("/chat")
        // .service(chat_auth_route)
        .service(connect_to_chat)
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
    Ok(Option<usize>),
    Err(HttpResponse),
}

pub fn get_auth_token(app_ctx: Data<AppContext>, uuid: Uuid) -> AuthTokenResponse {
    {
        let Ok(mut auth_tokens) = app_ctx.auth_tokens.lock() else {
            return AuthTokenResponse::Err(HttpResponse::InternalServerError().body("Nao foi possivel adquirir auth_tokens"));
        };
        println!("antiga -> {:?}", auth_tokens.keys());
        let auth_token: usize;
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

#[get("/{uuid}")]
pub async fn connect_to_chat(
    req: HttpRequest,
    stream: Payload,
    // srv: Data<Addr<Lobby>>,
    info: Path<ConnectChatInfo>,
    session: Session,
    app_ctx: Data<AppContext>,
) -> Result<HttpResponse, actix_web::Error> {
    {
        println!(
            "route -> {:?}, session -> {:?}, authid -> {:?}, thread -> {:?}",
            app_ctx.auth_tokens.lock().unwrap().keys(),
            session.entries(),
            -1,
            thread::current().id()
        );
    }
    // let res = get_auth_token(app_ctx, query.auth);

    // let AuthTokenResponse::Ok(user_id) = res else {
    //     let AuthTokenResponse::Err(error) = res else {
    //         todo!()
    //     };
    //     return Ok(error);
    // };

    // let Some(user_id) = user_id else {
    //     return Ok(HttpResponse::Unauthorized().body("Not authorized..."));
    // };
    let res = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(user_id) = res else {
        let RespostaAdquirirIdSessao::Erro(err) =  res else {
            todo!();
        };
        return Ok(err);
    };

    let ws = ChatWs::new(info.uuid, app_ctx.chat_server.clone(), user_id);
    ws::start(ws, &req, stream)
}
