use crate::{
    card_queries::{last_15, last_25_by_category},
    helper, modules,
};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

#[get("/{category}/{slug}.html")]
async fn exact(
    state: web::Data<State>,
    web::Path((url_category, slug)): web::Path<(String, String)>,
    mut context: LayoutContext,
) -> impl Responder {
    let card = state.fetcher.fetch_exact(slug).await;

    if card.is_err() {
        return helper::redirect(&format!("/{}/", url_category));
    }

    let card = card.unwrap();

    let center_tpl = state
        .tera
        .render(
            "modules/exact_card/tpl.tera",
            &Context::from_serialize(&card).unwrap(),
        )
        .unwrap();

    let last_cards = state.fetcher.fetch(last_15(), true).await.unwrap();
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

    let card_category = format!("{:?}", card.category);
    let category_cards = state
        .fetcher
        .fetch(last_25_by_category(&card_category), true)
        .await
        .unwrap();

    let right_tpl2 = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(card.category.to_description().to_string()),
                cards: category_cards,
            })
            .unwrap(),
        )
        .unwrap();

    context.insert("center_content", &center_tpl);
    context.insert("right_content", &format!("{}{}", right_tpl, right_tpl2));
    context.insert("article_category", &card.category.to_description());
    context.insert("card", &card);
    context.insert("article_name", &card.title);
    context.insert("article_description", &card.description);
    context.insert("og_image", &card.og_image);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/exact.tera", &context).unwrap())
}
