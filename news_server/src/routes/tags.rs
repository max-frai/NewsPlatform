use crate::{card_queries::CardQuery, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::tag::{Tag, TagKind};
use std::str::FromStr;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/tags/")]
async fn tags_all(state: web::Data<State>) -> impl Responder {
    tag_logic(state, None).await
}

#[get("/tags/{kind}")]
async fn tags_scope(state: web::Data<State>, web::Path(kind): web::Path<String>) -> impl Responder {
    tag_logic(state, Some(kind)).await
}

async fn tag_logic(state: web::Data<State>, kind: Option<String>) -> impl Responder {
    let all_tags: Vec<&Tag> = state
        .tags_manager
        .tags
        .iter()
        .filter(|tag| {
            if let Some(kind) = kind.as_ref().and_then(|kind| TagKind::from_str(&kind).ok()) {
                return tag.1.kind == kind;
            } else {
                return true;
            };
        })
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
    context.insert("tags", &all_tags);
    context.insert("right_content", &right_tpl);

    let mut group_buttons = vec![(String::new(), "Все")];
    for tag in TagKind::iter() {
        group_buttons.push((tag.to_string(), tag.to_description()));
    }
    context.insert("buttons_base_link", "/tags/");
    context.insert("buttons_active_tag", &kind.unwrap_or("".to_string()));
    context.insert("group_buttons", &group_buttons);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/tags.tera", &context).unwrap())
}
