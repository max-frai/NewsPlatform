use crate::state::State;
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

#[get("/robots.txt")]
async fn robots(state: web::Data<State>) -> impl Responder {
    let body = r#"#
    #██╗░░██╗██╗░░░██╗██████╗░  ██╗░░░░░░█████╗░██╗██████╗░
    #██║░░██║██║░░░██║██╔══██╗  ██║░░░░░██╔══██╗██║██╔══██╗
    #███████║██║░░░██║██████╦╝  ██║░░░░░██║░░██║██║██║░░██║
    #██╔══██║██║░░░██║██╔══██╗  ██║░░░░░██║░░██║██║██║░░██║
    #██║░░██║╚██████╔╝██████╦╝  ███████╗╚█████╔╝██║██████╔╝
    #╚═╝░░╚═╝░╚═════╝░╚═════╝░  ╚══════╝░╚════╝░╚═╝╚═════╝░

    User-agent: *
    Disallow: /"#;
    HttpResponse::Ok().content_type("text/plain").body(body)
}
