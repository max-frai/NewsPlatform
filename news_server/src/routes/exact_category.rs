use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::category::Category;
use std::str::FromStr;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/categories/{category}")]
async fn exact_category(
    state: web::Data<State>,
    web::Path(category): web::Path<String>,
) -> impl Responder {
    let category = Category::from_str(&category).unwrap_or(Category::Other);
    let category_str = format!("{:?}", category);

    let category_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(15),
            sort: Some(doc! { "date" : -1 }),
            query: doc! {
                "category" : category_str,
            },
        })
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

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);
    context.insert("category", &category.to_string());
    context.insert("title", &title);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/exact_category.tera", &context)
            .unwrap(),
    )
}
