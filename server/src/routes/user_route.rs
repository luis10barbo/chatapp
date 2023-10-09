use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder, Scope,
};
use serde::Deserialize;

use crate::{db::user::UserTable, AppContext};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .service(create_user)
        .service(login_user)
        .service(user_info)
}

#[derive(Debug, Deserialize)]
struct CreateUserBody {
    nickname: String,
    name: Option<String>,
    password: String,
}

#[post("/register")]
async fn create_user(app_ctx: Data<AppContext>, body: web::Json<CreateUserBody>) -> impl Responder {
    let db_ref = app_ctx.db.try_lock().unwrap();
    let res = db_ref.create_user(
        body.nickname.clone(),
        body.name.clone(),
        body.password.clone(),
    );
    if res.is_err() {
        return HttpResponse::InternalServerError().body(res.unwrap_err().to_string());
    }
    HttpResponse::Ok().body(format!("Usuario {} criado", res.unwrap()))
}
#[derive(Debug, Deserialize)]
struct LoginUserBody {
    nickname: String,
    password: String,
}
const USER_ID_KEY: &str = "user_id";

#[post("/login")]
async fn login_user(
    app_ctx: Data<AppContext>,
    body: web::Json<LoginUserBody>,
    session: Session,
) -> impl Responder {
    let mut res = false;
    {
        let user_id = app_ctx
            .db
            .lock()
            .unwrap()
            .login_user(body.nickname.clone(), body.password.clone())
            .unwrap();

        if user_id != 0 {
            session.insert(USER_ID_KEY, user_id);
            res = true;
        }
    }

    HttpResponse::Ok().body(res.to_string())
}

#[get("/")]
async fn user_info(app_ctx: Data<AppContext>, session: Session) -> impl Responder {
    println!("{:?}", session.entries());
    let user_id = session.get::<usize>(USER_ID_KEY).unwrap();
    if user_id.is_none() {
        return HttpResponse::Ok().body(());
    }
    let user = app_ctx
        .db
        .lock()
        .unwrap()
        .get_user(user_id.unwrap())
        .unwrap();
    HttpResponse::Ok().json(user)
}
