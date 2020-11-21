use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use tera::Context;

#[get("/general/{id}_{slug}")]
async fn exact(
    state: web::Data<State>,
    web::Path((id, _)): web::Path<(String, String)>,
) -> impl Responder {
    let card = state.fetcher.fetch_exact(id).await.unwrap();
    let center_tpl = state
        .tera
        .render(
            "modules/exact_card/tpl.tera",
            &Context::from_serialize(&card).unwrap(),
        )
        .unwrap();

    let last_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(10),
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
                title: Some(String::from("Последние новости")),
                cards: last_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let category_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(10),
            sort: Some(doc! { "date" : -1 }),
            query: doc! { "category" : card.category },
        })
        .await
        .unwrap();

    let right_tpl2 = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Новости Спорта")),
                cards: category_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let mut context = Context::new();
    context.insert("center_content", &center_tpl);
    context.insert("right_content", &format!("{}{}", right_tpl, right_tpl2));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/exact.tera", &context).unwrap())
}
