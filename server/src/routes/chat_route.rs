use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data, Json, Path, Payload, Query},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{
        chat_db::{Chat, ChatTable, ChatTypes},
        chat_message_db::ChatMessagesTable,
    },
    routes::user_route::RespostaAdquirirIdSessao,
    sockets::{
        chat::{lobby_actor::ChatDeleted, lobby_socket::ChatWs},
        info::info_actor,
    },
    AppContext,
};

use super::user_route::{get_user_id, is_logged_in};

pub fn chat_scope() -> Scope {
    web::scope("/chat")
        // .service(chat_auth_route)
        .service(connect_to_chat)
        .service(create_chat_route)
        .service(get_chats_router)
        .service(get_messages)
        .service(remove_chat)
        .service(get_chat_router)
        .service(rota_update)
}

#[get("/auth")]
async fn chat_auth_route(session: Session, app_ctx: Data<AppContext>) -> impl Responder {
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
    }
    HttpResponse::Ok().body(uuid.to_string())
}

#[derive(Deserialize)]
pub struct ConnectChatInfo {
    pub uuid: String,
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

        let auth_token: i64;
        {
            let Some(auth_token_local) = auth_tokens.get(&uuid).clone() else {
                return AuthTokenResponse::Ok(None);
            };
            auth_token = *auth_token_local;
        }
        (*auth_tokens).remove(&uuid);

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
    let chats = db.get_chats();
    let Ok(chats) = chats else {
        println!("{:?}", chats.unwrap_err());
        return HttpResponse::InternalServerError().body("Erro adquirindo chats");
    };
    HttpResponse::Ok().json(chats)
}

#[derive(Debug, Deserialize)]
pub struct GetChatQuery {
    pub id: String,
}
#[get("/get")]
pub async fn get_chat_router(
    app_ctx: Data<AppContext>,
    query: Query<GetChatQuery>,
) -> impl Responder {
    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Erro adquirindo db");
    };

    let chat = db.get_chat(&query.id, ChatTypes::GROUP);
    let Ok(chat) = chat else {
        log::error!("{:?}", chat.unwrap_err());
        return HttpResponse::InternalServerError().body("Erro adquirindo chat");
    };

    HttpResponse::Ok().json(chat)
}

#[post("/create")]
pub async fn create_chat_route(
    session: Session,
    app_ctx: Data<AppContext>,
    body: Json<CreateChatRoute>,
) -> impl Responder {
    let is_logged_in = is_logged_in(&session);
    let Ok(user_id) = is_logged_in else {
        return is_logged_in.unwrap_err();
    };
    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Error fetching db from context");
    };

    let res = db.create_chat(&body.nome, user_id);
    let Ok(chat_id) = res else {
        log::error!("Error creating chat, {:?}", res.unwrap_err());
        return HttpResponse::InternalServerError().body("Erro ao criar grupo de chat.")
    };

    let chat = db.get_chat(&chat_id, ChatTypes::GROUP);
    let Ok(chat) = chat else {
        let err = chat.unwrap_err();
        return HttpResponse::InternalServerError().body(err.to_string())
    };

    if let Err(err) = app_ctx
        .info_server
        .send(info_actor::ChatCreated {
            room_id: chat_id.clone(),
            user_id: user_id,
        })
        .await
    {
        log::error!("Error sending message to user {:?}", err)
    };

    HttpResponse::Ok().json(chat)
}

#[derive(Debug, Deserialize)]
pub struct QueryConnectChat {
    pub t: ChatTypes,
}

#[get("/connect/{uuid}")]
pub async fn connect_to_chat(
    req: HttpRequest,
    stream: Payload,
    // srv: Data<Addr<Lobby>>,
    info: Path<ConnectChatInfo>,
    session: Session,
    app_ctx: Data<AppContext>,
    query: Query<QueryConnectChat>,
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

        if db.get_chat(&info.uuid, query.t).is_err() {
            return Ok(
                HttpResponse::BadRequest().body(format!("Chat {} nao encontrado", &info.uuid))
            );
        };
    }

    let ws = ChatWs::new(info.uuid.clone(), app_ctx.chat_server.clone(), user_id);
    ws::start(ws, &req, stream)
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    offset: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesPath {
    uuid: Uuid,
}

#[get("/messages/{uuid}")]
pub async fn get_messages(
    app_ctx: Data<AppContext>,
    query: Query<GetMessagesQuery>,
    path: Path<GetMessagesPath>,
) -> impl Responder {
    let db = app_ctx.db.lock().unwrap();
    let res = db.get_chat_messages(path.uuid.to_string().clone(), query.offset);
    let Ok(messages) = res else {
        return HttpResponse::InternalServerError().body("Undocumented error getting messages");
    };
    HttpResponse::Ok().json(messages)
}

#[derive(Debug, Deserialize)]
pub struct DeleteBody {
    chat_id: String,
}

#[post("/remove")]
pub async fn remove_chat(
    body: Json<DeleteBody>,
    session: Session,
    app_ctx: Data<AppContext>,
) -> impl Responder {
    let user_id = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(user_id) = user_id else {
        let RespostaAdquirirIdSessao::Erro(err) = user_id else {
            panic!()
        };
        return err;
    };

    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Falha ao adquirir db");
    };

    if let Err(err) = db.remove_chat(&body.chat_id) {
        log::error!("{:?}", err);
        return HttpResponse::InternalServerError().body("Erro ao deletar chat");
    };

    if let Err(err) = app_ctx
        .chat_server
        .send(ChatDeleted {
            chat_id: body.chat_id.clone(),
        })
        .await
    {
        log::error!("Error sending message to user: {:?}", err)
    }

    if let Err(err) = app_ctx
        .info_server
        .send(info_actor::ChatDeleted {
            room_id: body.chat_id.clone(),
            user_id: user_id,
        })
        .await
    {
        log::error!("Error sending message to user {:?}", err)
    };

    HttpResponse::Ok().body(format!("Chat {} deletado", body.chat_id))
}

#[post("/update")]
async fn rota_update(
    session: Session,
    app_ctx: Data<AppContext>,
    chat: Json<Chat>,
) -> impl Responder {
    println!("{:?}", chat);

    let user_id = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(user_id) = user_id else {
        let RespostaAdquirirIdSessao::Erro(err) = user_id else {
            panic!();
        };
        return err;
    };
    let Ok(db) = app_ctx.db.lock() else {
        log::error!("Error getting db, maybe it's poisoned?");
        return HttpResponse::InternalServerError().body("Erro adquirindo db.");
    };

    if chat.creator_id != user_id {
        return HttpResponse::Unauthorized().body("Você só pode modificar suas informações.");
    }

    let res = db.update_chat(Chat {
        chat_desc: chat.chat_desc.clone(),
        chat_id: chat.chat_id.clone(),
        chat_image: chat.chat_image.clone(),
        chat_name: chat.chat_name.clone(),
        date_created: chat.date_created.clone(),
        chat_type: chat.chat_type,
        creator_id: chat.creator_id,
        last_message: None,
    });
    let Ok(modified) = res else {
        let err = res.unwrap_err();
        log::error!("{:?}", err);
        return HttpResponse::InternalServerError().body("Erro ao atualizar usuario.");
    };

    if modified < 1 {
        return HttpResponse::NotModified().body("Nada modificado");
    }

    HttpResponse::Ok().body(format!("{:?}", modified))
}
