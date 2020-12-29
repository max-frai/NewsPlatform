use crate::{
    card_queries::{last_15, last_15_by_category, CardQuery},
    helper::redirect,
    modules,
};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::category::Category;
use std::str::FromStr;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/{category}")]
async fn exact_category_fix(web::Path(category): web::Path<String>) -> HttpResponse {
    redirect(&format!("/{}/", category))
}

#[get("/{category}/")]
async fn exact_category(
    state: web::Data<State>,
    web::Path(category): web::Path<String>,
    mut context: LayoutContext,
) -> impl Responder {
    let category = Category::from_str(&category).unwrap_or(Category::Other);
    let category_str = format!("{:?}", category);

    let category_cards = state
        .fetcher
        .fetch(last_15_by_category(&category_str), true)
        .await
        .unwrap();

    let title = format!("{}: все новости", category.to_description());
    let news_list_tpl = state
        .tera
        .render(
            "modules/news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: None,
                cards: category_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let last_cards = state.fetcher.fetch(last_15(), true).await.unwrap();
    let right_tpl = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последнее")),
                cards: last_cards,
            })
            .unwrap(),
        )
        .unwrap();

    context.insert("center_content", &news_list_tpl);
    context.insert("right_content", &right_tpl);
    context.insert("category", &category.to_string());
    context.insert("title", &title);
    context.insert("category_name", &category.to_description());

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/exact_category.tera", &context)
            .unwrap(),
    )
}
