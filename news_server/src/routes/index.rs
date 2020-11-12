use crate::state::State;
use crate::{
    modules,
};
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;
use crate::card_queries::INDEX_CARDQUERY;

#[get("/")]
async fn index(state: web::Data<State>) -> impl Responder {
    let index_cards = state.fetcher.fetch(&INDEX_CARDQUERY).await.unwrap();

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

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/index.tera", &context).unwrap())
}
