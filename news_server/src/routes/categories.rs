use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, http::header, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::category::Category;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/categories")]
async fn categories_fix() -> HttpResponse {
    HttpResponse::MovedPermanently()
        .header(actix_web::http::header::LOCATION, "/categories/")
        .finish()
}

#[get("/categories/")]
async fn categories(state: web::Data<State>) -> impl Responder {
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

    let last_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(15),
            sort: Some(doc! { "date" : -1 }),
            query: doc! {},
        })
        .await
        .unwrap();

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

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);
    context.insert("right_content", &right_tpl);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/categories.tera", &context)
            .unwrap(),
    )
}
