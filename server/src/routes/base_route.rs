use actix_session::Session;
use actix_web::{
    get,
    web::{Data, Payload},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;

use crate::{sockets::info::info_socket::InfoWS, AppContext};

use super::user_route::{get_user_id, RespostaAdquirirIdSessao};

pub fn base_scope() -> Scope {
    Scope::new("/").service(info_route)
}

#[get("/")]
pub async fn index_route() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

#[get("/info")]
pub async fn info_route(
    session: Session,
    app_ctx: Data<AppContext>,
    req: HttpRequest,
    stream: Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = get_user_id(&session);
    let RespostaAdquirirIdSessao::Id(user_id) = user_id else {
        let RespostaAdquirirIdSessao::Erro(err) = user_id else {
            panic!("")
        };
        return Ok(err);
    };
    let actor = InfoWS::new(user_id, app_ctx.info_server.clone());
    ws::start(actor, &req, stream)
}
