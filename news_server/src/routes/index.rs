use crate::{
    card_queries::{last_25, last_n, CardQuery},
    modules,
};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use tera::Context;

#[get("/")]
async fn index(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    let index_cards = state.fetcher.fetch(last_n(35)).await.unwrap();
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

    let top_persons = state.top_persons.read().await;
    context.insert("top_persons", &*top_persons);

    let top_gpe = state.top_gpe.read().await;
    context.insert("top_gpe", &*top_gpe);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/index.tera", &context).unwrap())
}
