use crate::state::State;
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

#[get("/robots.txt")]
async fn robots(state: web::Data<State>) -> impl Responder {
    let sitemap_url = format!("{}/sitemap.xml", state.constants.full_domain);
    let body = format!("User-agent: *\nDisallow:\nSitemap: {}", sitemap_url);
    HttpResponse::Ok().content_type("text/plain").body(&body)
}
