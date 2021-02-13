use crate::{helper, modules};
use crate::{layout_context::LayoutContext, state::State};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::card_queries::{last_15, last_25_by_category};
use tera::Context;

async fn _exact_logic(
    is_amp: bool,
    state: web::Data<State>,
    web::Path((url_category, slug)): web::Path<(String, String)>,
    mut context: LayoutContext,
) -> impl Responder {
    let card = state.fetcher.fetch_exact(slug).await;

    if card.is_err() {
        return helper::redirect(&format!("/{}/", url_category));
    }

    let card = card.unwrap();

    let exact_card_tpl = if !is_amp {
        "modules/exact_card/tpl.tera"
    } else {
        "modules/exact_card_amp/tpl.tera"
    };

    let mut center_tpl = state
        .tera
        .render(exact_card_tpl, &Context::from_serialize(&card).unwrap())
        .unwrap();

    if is_amp {
        let root_node = state.dom_helper.parse_html(&center_tpl);
        for img in state.dom_helper.select(&root_node, "img") {
            state.dom_helper.set_node_attribute(&img, "layout", "fill");
            let wrapper = state.dom_helper.wrap_with_element(&img, "div");
            state
                .dom_helper
                .set_node_attribute(&wrapper, "class", "imgWrap");
            state.dom_helper.rename_node(&img, "amp-img");
        }

        let mut result_html_buf: Vec<u8> = vec![];
        root_node.serialize(&mut result_html_buf).unwrap();
        center_tpl = std::str::from_utf8(&result_html_buf)
            .unwrap_or_default()
            .to_string();
    }

    let last_cards = state.fetcher.fetch(last_15(), true).await.unwrap();
    let right_tpl = state
        .tera
        .render(
            "modules/compact_news_list/tpl.tera",
            &Context::from_serialize(&modules::news_list::NewsListTpl {
                title: Some(String::from("Последнее")),
                cards: last_cards,
                is_amp,
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
                is_amp,
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
    context.insert("is_amp", &is_amp);

    let main_tpl = if !is_amp {
        "routes/exact.tera"
    } else {
        "routes/exact_amp.tera"
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render(main_tpl, &context).unwrap())
}

#[get("/amp/{category}/{slug}.html")]
async fn exact_amp(
    state: web::Data<State>,
    web::Path((url_category, slug)): web::Path<(String, String)>,
    context: LayoutContext,
) -> impl Responder {
    _exact_logic(true, state, web::Path((url_category, slug)), context).await
}

#[get("/{category}/{slug}.html")]
async fn exact(
    state: web::Data<State>,
    web::Path((url_category, slug)): web::Path<(String, String)>,
    context: LayoutContext,
) -> impl Responder {
    _exact_logic(false, state, web::Path((url_category, slug)), context).await
}
