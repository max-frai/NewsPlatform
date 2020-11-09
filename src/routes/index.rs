use crate::state::State;
use crate::{
    card_fetcher::{CardFetcher, CardFetcherKind},
    modules,
};
use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    center_content: String,
}

#[get("/")]
async fn index(state: web::Data<State>) -> impl Responder {
    let index_cards = state.fetcher.fetch(CardFetcherKind::Index).await.unwrap();

    let news_list_tpl = modules::news_list::NewsListTpl {
        title: Some(String::from("Последние новости")),
        cards: index_cards,
    }
    .render()
    .unwrap();

    HttpResponse::Ok().content_type("text/html").body(
        IndexTemplate {
            center_content: news_list_tpl,
        }
        .render()
        .unwrap(),
    )
}
