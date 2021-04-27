use crate::{helper::redirect, modules};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::{
    card::Card,
    card_queries::{last_15, last_40_by_category},
    category::Category,
};
use std::str::FromStr;
use tera::Context;

#[get("/society")]
pub async fn society_category_fix() -> HttpResponse {
    redirect("/society/")
}

#[get("/entertainment")]
pub async fn entertainment_category_fix() -> HttpResponse {
    redirect("/entertainment/")
}

#[get("/economy")]
pub async fn economy_category_fix() -> HttpResponse {
    redirect("/economy/")
}

#[get("/technology")]
pub async fn technology_category_fix() -> HttpResponse {
    redirect("/technology/")
}

#[get("/sports")]
pub async fn sports_category_fix() -> HttpResponse {
    redirect("/sports/")
}

#[get("/science")]
pub async fn science_category_fix() -> HttpResponse {
    redirect("/science/")
}

#[get("/other")]
pub async fn other_category_fix() -> HttpResponse {
    redirect("/other/")
}

// ----------------------------------------------------------------

#[get("/society/")]
pub async fn society_category(state: web::Data<State>, context: LayoutContext) -> impl Responder {
    _exact_category(state, "society", context).await
}

#[get("/entertainment/")]
pub async fn entertainment_category(
    state: web::Data<State>,
    context: LayoutContext,
) -> impl Responder {
    _exact_category(state, "entertainment", context).await
}

#[get("/economy/")]
pub async fn economy_category(state: web::Data<State>, context: LayoutContext) -> impl Responder {
    _exact_category(state, "economy", context).await
}

#[get("/technology/")]
pub async fn technology_category(
    state: web::Data<State>,
    context: LayoutContext,
) -> impl Responder {
    _exact_category(state, "technology", context).await
}

#[get("/sports/")]
pub async fn sports_category(state: web::Data<State>, context: LayoutContext) -> impl Responder {
    _exact_category(state, "sports", context).await
}

#[get("/science/")]
pub async fn science_category(state: web::Data<State>, context: LayoutContext) -> impl Responder {
    _exact_category(state, "science", context).await
}

#[get("/other/")]
pub async fn other_category(state: web::Data<State>, context: LayoutContext) -> impl Responder {
    _exact_category(state, "other", context).await
}

async fn _exact_category(
    state: web::Data<State>,
    category: &str,
    mut context: LayoutContext,
) -> impl Responder {
    let category = Category::from_str(category).unwrap_or(Category::Other);
    let category_str = format!("{:?}", category);

    let category_cards = state
        .fetcher
        .fetch(last_40_by_category(&category_str), true)
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
                is_amp: false,
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
                is_amp: false,
            })
            .unwrap(),
        )
        .unwrap();

    let mut category_cards = Vec::<Card>::new();
    let card_category = category.to_string();
    let clusters = &state.popular_clusters.read().await.clusters;
    for cluster in clusters {
        if cluster.category == card_category {
            category_cards = cluster
                .threads
                .iter()
                .take(6)
                .map(|thread| thread.main_item.clone())
                .collect();
            break;
        }
    }

    let right_tpl2 = state
        .tera
        .render(
            "modules/image_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(format!(
                    "{} - Важное",
                    category.to_description().to_string()
                )),
                cards: category_cards,
                is_amp: false,
            })
            .unwrap(),
        )
        .unwrap();

    context.insert("center_content", &news_list_tpl);
    context.insert("right_content", &format!("{}{}", right_tpl2, right_tpl));
    // context.insert("right_content", &right_tpl);
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
