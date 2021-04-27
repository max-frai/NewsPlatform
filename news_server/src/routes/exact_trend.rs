use crate::{helper::redirect, modules};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::card_queries::{last_25, last_40_by_trend};
use std::str::FromStr;
use tera::Context;

#[get("/trend/{slug}")]
async fn exact_trend_fix(slug: web::Path<String>) -> HttpResponse {
    redirect(&format!("/trend/{}/", slug))
}

#[get("/trend/{slug}/")]
async fn exact_trend(
    mut context: LayoutContext,
    state: web::Data<State>,
    slug: web::Path<String>,
) -> impl Responder {
    let trend_cards = state
        .fetcher
        .fetch(last_40_by_trend(&*slug), true)
        .await
        .unwrap();

    let last_cards = state.fetcher.fetch(last_25(), true).await.unwrap();

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

    let title = format!("{}: все новости", slug);
    let news_list_tpl = state
        .tera
        .render(
            "modules/news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: None,
                cards: trend_cards,
                is_amp: false,
            })
            .unwrap(),
        )
        .unwrap();

    context.insert("center_content", &news_list_tpl);
    context.insert("right_content", &right_tpl);
    // context.insert("tag", &tag);
    // context.insert("title", &title);

    // let meta_title = match &kind {
    //     TagKind::Loc => format!("{} последние новости - главное на сегодня", tag.wiki_title),
    //     TagKind::Per => format!(
    //         "{} последние новости - свежие статьи и информация",
    //         tag.wiki_title
    //     ),
    //     _ => format!(
    //         "Новости {} - последние и главные новости на сегодня",
    //         tag.wiki_title
    //     ),
    // };

    // let meta_desc = match kind {
    //     TagKind::Loc => format!("HubLoid {} ➤ Главные и последние новости по {} ✔ Важные обновления каждый день", tag.wiki_title, tag.wiki_title),
    //     TagKind::Per => format!("HubLoid {} ➤ Последние новости и статьи по персоне {} ✔ Информация и все упоминания", tag.wiki_title, tag.wiki_title),
    //     _ => format!("HubLoid {} ➤ Последние новости по {} - вся важная информация ✔ Свежие обновления каждый день", tag.wiki_title, tag.wiki_title)
    // };

    // context.insert("meta_title", &meta_title);
    // context.insert("meta_description", &meta_desc);

    HttpResponse::Ok().content_type("text/html").body(
        state
            .tera
            .render("routes/exact_trend.tera", &context)
            .unwrap(),
    )
}
