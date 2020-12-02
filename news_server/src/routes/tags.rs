use crate::{card_queries::CardQuery, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::tag::Tag;
use tera::Context;

#[get("/tags/")]
async fn tags(state: web::Data<State>) -> impl Responder {
    let tags: Vec<&Tag> = state
        .tags_manager
        .tags
        .iter()
        .take(50)
        .map(|(_, val)| val)
        .collect();

    let last_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(25),
            sort: Some(doc! { "date" : -1 }),
            query: doc! {},
        })
        .await
        .unwrap();

    let right_tpl = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&crate::modules::news_list::NewsListTpl {
                title: Some(String::from("Последнее")),
                cards: last_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let mut context = Context::new();
    context.insert("tags", &tags);
    context.insert("right_content", &right_tpl);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/tags.tera", &context).unwrap())
}
