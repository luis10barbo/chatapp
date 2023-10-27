use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data},
    web::{Json, Query},
    HttpResponse, Responder, Scope,
};
use rusqlite::ErrorCode;
use serde::Deserialize;

use crate::{
    db::user_db::{User, UserTable},
    AppContext,
};

pub trait UserSession {
    fn insert_user_id(&self, user_id: i64) -> Result<(), HttpResponse>;
    fn get_user_id(&self) -> Result<Option<i64>, HttpResponse>;
}

impl UserSession for Session {
    fn insert_user_id(&self, user_id: i64) -> Result<(), HttpResponse> {
        if let Err(err) = self.insert(USER_ID_KEY, user_id) {
            return Err(HttpResponse::InternalServerError()
                .body(format!("Erro ao salvar sessão: {}", err.to_string())));
        };
        Ok(())
    }
    fn get_user_id(&self) -> Result<Option<i64>, HttpResponse> {
        let Ok(user_id) = self.get::<i64>(USER_ID_KEY) else {
            return Err(HttpResponse::InternalServerError().body("Erro ao adquirir id do usuario de sessao"));
        };
        Ok(user_id)
    }
}

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
async fn create_user(
    app_ctx: Data<AppContext>,
    body: web::Json<AuthUserBody>,
    session: Session,
) -> impl Responder {
    let db_ref = app_ctx.db.try_lock().unwrap();
    let res = db_ref.create_user(body.usuario.clone(), body.senha.clone());
    let Ok(user_id) = res else {
        let err = res.unwrap_err();
        if let Some(sqlite_err) = err.sqlite_error() {
            if sqlite_err.code == ErrorCode::ConstraintViolation {
                return HttpResponse::Conflict()
                    .body(format!("Usuario \"{}\" já existe", body.usuario));
            }
        }
        return HttpResponse::InternalServerError().body(err.to_string());
    };

    if let Err(err) = session.insert_user_id(user_id) {
        return err;
    };
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
    let Ok(user_id) = login_res else {
        return HttpResponse::NotFound().body("Usuario não encontrado");
    };

    if let Some(user_id) = user_id {
        if let Err(err) = session.insert_user_id(user_id) {
            return err;
        }

        if let Ok(user) = db.get_user(user_id) {
            return HttpResponse::Ok().json(user);
        }
        return HttpResponse::InternalServerError().body("Error fetching user");
    }
    return HttpResponse::Unauthorized().body("Senha incorreta");
}

#[derive(Debug, Deserialize)]
struct UserInfoQuery {
    id: i64,
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

    let Ok(user) = db.get_user(query.id) else {
        return HttpResponse::NotFound().body("Usuario nao encontrado");
    };
    HttpResponse::Ok().json(user)
}

#[get("/me")]
async fn my_user_info(app_ctx: Data<AppContext>, session: Session) -> impl Responder {
    let user_id = session.get_user_id();
    let Ok(user_id) = session.get_user_id() else {
        return user_id.unwrap_err()
    };

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
    Id(i64),
    Erro(HttpResponse),
}

pub fn get_user_id(session: &Session) -> RespostaAdquirirIdSessao {
    let res = session.get_user_id();
    let Ok(res) = res else {
        return RespostaAdquirirIdSessao::Erro(res.unwrap_err());
    };

    if res.is_none() {
        return RespostaAdquirirIdSessao::Erro(
            HttpResponse::Unauthorized().body("Usuario nao logado"),
        );
    }
    RespostaAdquirirIdSessao::Id(res.unwrap())
}

pub fn is_logged_in(session: &Session) -> Result<i64, HttpResponse> {
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

#[post("/update")]
async fn rota_update(
    session: Session,
    app_ctx: Data<AppContext>,
    user: Json<User>,
) -> impl Responder {
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

    if (user.user_id != user_id) {
        return HttpResponse::Unauthorized().body("Você só pode modificar suas informações.");
    }

    let res = db.update_user(User {
        user_id: user.user_id,
        user_nick: user.user_nick.clone(),
        user_name: user.user_name.clone(),
        user_status: user.user_status.clone(),
        user_email: user.user_email.clone(),
        user_image: user.user_image.clone(),
    });
    let Ok(modified) = res else {
        let err = res.unwrap_err();
        log::error!("{:?}", err);
        return HttpResponse::InternalServerError().body("Erro ao atualizar usuario.");
    };

    if modified < 1 {
        return HttpResponse::NotModified().body("Nada modificado");
    }

    HttpResponse::Ok().body("")
}
