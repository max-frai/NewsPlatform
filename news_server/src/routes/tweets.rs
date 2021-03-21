use crate::{helper::redirect, modules};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::{card_queries::last_15, category::Category};
use strum::IntoEnumIterator;
use tera::Context;

#[get("/tweets")]
async fn tweets_fix() -> HttpResponse {
    redirect("/tweets/")
}

#[get("/tweets/")]
async fn tweets(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    let last_cards = state.fetcher.fetch(last_15(), true).await.unwrap();

    let right_tpl = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последнее")),
                cards: last_cards,
                is_amp: false,
            })
            .unwrap(),
        )
        .unwrap();
    // context.insert("center_content", &news_list_tpl);
    context.insert("right_content", &right_tpl);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/tweets.tera", &context).unwrap())
}
