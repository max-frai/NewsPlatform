use crate::state::State;
use crate::{
    card_fetcher::{CardFetcher, CardFetcherKind},
    modules,
};
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

#[get("/")]
async fn index(state: web::Data<State>) -> impl Responder {
    let index_cards = state.fetcher.fetch(CardFetcherKind::Index).await.unwrap();

    let news_list_tpl = state
        .tera
        .render(
            "modules/news_list/tpl.html",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последние новости")),
                cards: index_cards,
            })
            .unwrap(),
        )
        .unwrap();

    let mut context = Context::new();
    context.insert("center_content", &news_list_tpl);
    // context.insert("vat_rate", &0.20);
    let html = state.tera.render("routes/index.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(
        html
        // IndexTemplate {
        //     center_content: news_list_tpl,
        // }
        // .render()
        // .unwrap(),
    )
}
