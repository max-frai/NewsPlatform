use crate::state::State;
use crate::{
    card_fetcher::{CardFetcher, CardFetcherKind},
    modules,
};
use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "routes/exact.html")]
struct ExactTemplate {
    center_content: String,
}

#[get("/general/{id}_{slug}")]
async fn exact(
    state: web::Data<State>,
    web::Path((id, slug)): web::Path<(String, String)>,
) -> impl Responder {
    let card = state
        .fetcher
        .fetch(CardFetcherKind::Exact(id))
        .await
        .unwrap();

    let exact_tpl = modules::exact_card::ExactCardTpl {
        card: card.first().unwrap().clone(),
    }
    .render()
    .unwrap();

    HttpResponse::Ok().content_type("text/html").body(
        ExactTemplate {
            center_content: exact_tpl,
        }
        .render()
        .unwrap(),
    )
}
