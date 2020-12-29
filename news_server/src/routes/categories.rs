use crate::{
    card_queries::{last_15, CardQuery},
    helper::redirect,
    modules,
};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, http::header, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::category::Category;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/categories")]
async fn categories_fix() -> HttpResponse {
    redirect("/categories/")
}

#[get("/categories/")]
async fn categories(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    let categories = Category::iter()
        .map(|item| (item.to_string(), item.to_description().to_owned()))
        .collect();

    let news_list_tpl = state
        .tera
        .render(
            "modules/categories_list/tpl.tera",
            &Context::from_serialize(&modules::categories_list::CategoriesListTpl { categories })
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

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/categories.tera", &context)
            .unwrap(),
    )
}
