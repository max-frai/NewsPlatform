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
    right_content: String,
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

    let center_content = modules::exact_card::ExactCardTpl {
        card: card.first().unwrap().clone(),
    }
    .render()
    .unwrap();

    let index_cards = state.fetcher.fetch(CardFetcherKind::Index).await.unwrap();
    let right_content = modules::compact_news_list::NewsListTpl {
        title: Some(String::from("Похожие новости")),
        cards: index_cards,
    }
    .render()
    .unwrap();

    HttpResponse::Ok().content_type("text/html").body(
        ExactTemplate {
            center_content,
            right_content,
        }
        .render()
        .unwrap(),
    )
}
