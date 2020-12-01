use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::{category::Category, tag::TagKind};
use std::str::FromStr;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/tags/{kind}/{slug}/")]
async fn exact_tag(
    state: web::Data<State>,
    web::Path((kind, slug)): web::Path<(String, String)>,
) -> impl Responder {
    let kind = TagKind::from_str(&kind).unwrap();
    let tag = state.tags_manager.find(kind, &slug).await.unwrap();

    let tag_cards = state
        .fetcher
        .fetch(CardQuery {
            lifetime: Duration::seconds(60),
            limit: Some(15),
            sort: Some(doc! { "date" : -1 }),
            query: doc! {
                "tags" : tag._id.to_owned()
            },
        })
        .await
        .unwrap();

    let title = format!("{}: все новости", tag.wiki_title);
    let news_list_tpl = state
        .tera
        .render(
            "modules/news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: None,
                cards: tag_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);
    context.insert("tag", tag);
    context.insert("title", &title);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/exact_tag.tera", &context)
            .unwrap(),
    )
}
