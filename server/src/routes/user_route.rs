use actix_session::{Session, SessionGetError};
use actix_web::{
    get, post,
    web::Payload,
    web::{self, Data},
    HttpRequest, HttpResponse, Responder, Scope,
};
use serde::Deserialize;

use crate::{db::user_db::UserTable, lobby::Lobby, socket::ChatWs, AppContext};

use super::chat_route::connect_to_chat;

pub fn user_scope() -> Scope {
    web::scope("/user")
        .service(create_user)
        .service(login_user)
        .service(user_info)
        .service(rota_sair)
}

#[derive(Debug, Deserialize)]
struct AuthUserBody {
    usuario: String,
    senha: String,
}

#[post("/registrar")]
async fn create_user(app_ctx: Data<AppContext>, body: web::Json<AuthUserBody>) -> impl Responder {
    let db_ref = app_ctx.db.try_lock().unwrap();
    let res = db_ref.create_user(body.usuario.clone(), body.senha.clone());
    if res.is_err() {
        return HttpResponse::InternalServerError().body(res.unwrap_err().to_string());
    }
    HttpResponse::Ok().body(format!("Usuario {} criado", res.unwrap()))
}

const USER_ID_KEY: &str = "user_id";

#[post("/login")]
async fn login_user(
    app_ctx: Data<AppContext>,
    body: web::Json<AuthUserBody>,
    session: Session,
) -> impl Responder {
    let db = app_ctx.db.lock().unwrap();
    let login_res = db.login_user(body.usuario.clone(), body.senha.clone());
    if login_res.is_err() {
        return HttpResponse::NotFound().body("Usuario não encontrado");
    }
    let user_id = login_res.unwrap();
    if let Some(user_id) = user_id {
        if let Err(err) = session.insert(USER_ID_KEY, user_id) {
            return HttpResponse::InternalServerError()
                .body(format!("Erro ao salvar sessão: {}", err.to_string()));
        };

        if let Ok(user) = db.get_user(user_id) {
            return HttpResponse::Ok().json(user);
        }
        return HttpResponse::InternalServerError().body("Error fetching user");
    }
    return HttpResponse::Unauthorized().body("Senha incorreta");
}

#[get("/")]
async fn user_info(app_ctx: Data<AppContext>, session: Session) -> impl Responder {
    let user_id = session.get::<usize>(USER_ID_KEY).unwrap();
    println!("{:?}", session.entries());
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("Usuario nao logado");
    }
    let user = app_ctx
        .db
        .lock()
        .unwrap()
        .get_user(user_id.unwrap())
        .unwrap();
    HttpResponse::Ok().json(user)
}

pub enum RespostaAdquirirIdSessao {
    Id(usize),
    Erro(HttpResponse),
}

pub fn get_user_id(session: &Session) -> RespostaAdquirirIdSessao {
    let res = session.get::<usize>(USER_ID_KEY);
    if res.is_err() {
        return RespostaAdquirirIdSessao::Erro(
            HttpResponse::InternalServerError().body("Erro ao adquirir ID na sessao"),
        );
    }
    let res = res.unwrap();
    if res.is_none() {
        return RespostaAdquirirIdSessao::Erro(
            HttpResponse::Unauthorized().body("Usuario nao logado"),
        );
    }
    RespostaAdquirirIdSessao::Id(res.unwrap())
}

#[post("/sair")]
async fn rota_sair(session: Session) -> impl Responder {
    let user_id = session.get::<usize>(USER_ID_KEY);
    println!("Deslogando!!!");
    if user_id.is_err() {
        return HttpResponse::InternalServerError().body("Erro ao adquirir a sessao");
    }

    session.remove(USER_ID_KEY);
    HttpResponse::Ok().body("Deslogado com sucesso!")
}
