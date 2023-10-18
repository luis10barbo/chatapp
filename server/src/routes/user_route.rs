use actix_session::Session;
use actix_web::{
    get, post,
    web::Query,
    web::{self, Data},
    HttpResponse, Responder, Scope,
};
use rusqlite::ErrorCode;
use serde::Deserialize;

use crate::{db::user_db::UserTable, AppContext};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .service(create_user)
        .service(login_user)
        .service(my_user_info)
        .service(rota_sair)
        .service(user_info)
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
        let err = res.unwrap_err();
        if let Some(sqlite_err) = err.sqlite_error() {
            if sqlite_err.code == ErrorCode::ConstraintViolation {
                return HttpResponse::Conflict()
                    .body(format!("Usuario {} já existe", body.usuario));
            }
        }
        return HttpResponse::InternalServerError().body(err.to_string());
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

#[derive(Debug, Deserialize)]
struct UserInfoQuery {
    id: usize,
}

#[get("/info")]
async fn user_info(
    app_ctx: Data<AppContext>,
    session: Session,
    query: Query<UserInfoQuery>,
) -> impl Responder {
    let RespostaAdquirirIdSessao::Id(_) = get_user_id(&session) else {
        return HttpResponse::Unauthorized().body("Precisa estar logado para acessar informacao de outros usuarios");
    };

    let Ok(db) = app_ctx.db.lock() else {
        return HttpResponse::InternalServerError().body("Erro ao adquirir db");
    };
    println!("{:?}", query.id);

    let Ok(user) = db.get_user(query.id) else {
        return HttpResponse::NotFound().body("Usuario nao encontrado");
    };
    HttpResponse::Ok().json(user)
}

#[get("/me")]
async fn my_user_info(app_ctx: Data<AppContext>, session: Session) -> impl Responder {
    let user_id = session.get::<usize>(USER_ID_KEY).unwrap();
    println!("{:?}", session.entries());
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("Usuario nao logado");
    }
    let user = app_ctx.db.lock().unwrap().get_user(user_id.unwrap());
    let Ok(user) = user else {
        return HttpResponse::NotFound().body(user.unwrap_err().to_string());
    };
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

pub fn is_logged_in(session: &Session) -> Result<usize, HttpResponse> {
    let RespostaAdquirirIdSessao::Id(id) = get_user_id(session) else {
        return Err(HttpResponse::Unauthorized().body("E necessario estar autenticado para utilizar essa funcao"));
    };
    Ok(id)
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
