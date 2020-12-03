use crate::state::State;
use crate::{card_queries::CardQuery, modules};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::{category::Category, tag::TagKind};
use std::str::FromStr;
use tera::Context;

#[get("/tags/{kind}/{slug}/")]
async fn exact_tag(
    state: web::Data<State>,
    web::Path((kind, slug)): web::Path<(String, String)>,
) -> impl Responder {
    let kind = TagKind::from_str(&kind).unwrap();
    let tag = state.tags_manager.find(kind.clone(), &slug).await.unwrap();

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
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последнее")),
                cards: last_cards,
            })
            .unwrap(),
        )
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
    context.insert("right_content", &right_tpl);
    context.insert("tag", tag);
    context.insert("title", &title);

    let meta_title = match &kind {
        TagKind::Gpe => format!("{} последние новости - главное на сегодня", tag.wiki_title),
        TagKind::Person => format!(
            "{} последние новости - свежие статьи и информация",
            tag.wiki_title
        ),
        _ => format!(
            "Новости {} - последние и главные новости на сегодня",
            tag.wiki_title
        ),
    };

    let meta_desc = match kind {
        TagKind::Gpe => format!("HubLoid {} ➤ Главные и последние новости по {} ✔ Важные обновления каждый день", tag.wiki_title, tag.wiki_title),
        TagKind::Person => format!("HubLoid {} ➤ Последние новости и статьи по персоне {} ✔ Информация и все упоминания", tag.wiki_title, tag.wiki_title),
        _ => format!("HubLoid {} ➤ Последние новости по {} - вся важная информация ✔ Свежие обновления каждый день", tag.wiki_title, tag.wiki_title)
    };

    context.insert("meta_title", &meta_title);
    context.insert("meta_description", &meta_desc);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/exact_tag.tera", &context)
            .unwrap(),
    )
}
