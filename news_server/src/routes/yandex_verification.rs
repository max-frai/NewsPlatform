use crate::state::State;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/yandex_72d0af7ac55abcfa.html")]
async fn yandex_verification(state: web::Data<State>) -> impl Responder {
    let body = format!("Verification: 72d0af7ac55abcfa ");
    HttpResponse::Ok().content_type("text/plain").body(&body)
}
