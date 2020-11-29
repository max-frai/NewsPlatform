use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::category::Category;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/categories")]
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

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/categories.tera", &context)
            .unwrap(),
    )
}
