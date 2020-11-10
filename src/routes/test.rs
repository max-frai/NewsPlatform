use crate::state::State;
use crate::{
    card_fetcher::{CardFetcher, CardFetcherKind},
    modules,
};
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

#[get("/test")]
async fn test(state: web::Data<State>) -> impl Responder {
    let test_tpl = state
        .tera
        .render("layouts/test.tera", &Context::new())
        .unwrap();

    HttpResponse::Ok().content_type("text/html").body(test_tpl)
}
