use crate::state::State;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/google7b7f46877ad77582.html")]
async fn search_console(_: web::Data<State>) -> impl Responder {
    let body = "google-site-verification: google7b7f46877ad77582.html";
    HttpResponse::Ok().content_type("text/plain").body(body)
}
