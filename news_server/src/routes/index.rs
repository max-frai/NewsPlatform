use crate::{
    card_queries::{last_25, last_n, CardQuery},
    modules,
    tag_cache::TagCache,
};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::tag::TagKind;
use tera::Context;

#[get("/")]
async fn index(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    let index_cards = state.fetcher.fetch(last_n(35), true).await.unwrap();
    let news_list_tpl = state
        .tera
        .render(
            "modules/news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последние новости")),
                cards: index_cards,
            })
            .unwrap(),
        )
        .unwrap();

    context.insert("center_content", &news_list_tpl);

    let tags_cache = state.tags_cache.read().await;

    let top_persons = tags_cache.get(&TagCache::DayExactTop(TagKind::Person));
    context.insert("top_persons", top_persons.unwrap_or(&vec![]));

    let top_gpe = tags_cache.get(&TagCache::DayExactTop(TagKind::Gpe));
    context.insert("top_gpe", top_gpe.unwrap_or(&vec![]));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/index.tera", &context).unwrap())
}
