use crate::{helper::redirect, modules};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::card_queries::last_25;
use std::str::FromStr;
use tera::Context;

#[get("/authors")]
pub async fn authors_fix() -> HttpResponse {
    redirect("/authors/")
}

// ----------------------------------------------------------------

#[get("/authors/")]
pub async fn authors(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    let last_cards = state.fetcher.fetch(last_25(), true).await.unwrap();
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

    context.insert("right_content", &right_tpl);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/authors.tera", &context).unwrap())
}
