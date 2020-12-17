use crate::{
    card_queries::{last_25, CardQuery},
    helper::redirect,
    layout_context::LayoutContext,
    state::State,
};
use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use chrono::Duration;
use news_general::tag::{Tag, TagKind};
use std::str::FromStr;
use strum::IntoEnumIterator;
use tera::Context;

#[get("/tags")]
async fn tags_all_fix() -> HttpResponse {
    redirect("/tags/")
}

#[get("/tags/")]
async fn tags_all(state: web::Data<State>, mut context: LayoutContext) -> impl Responder {
    tag_logic(state, None, context).await
}

#[get("/tags/{kind}")]
async fn tags_scope_fix(web::Path(kind): web::Path<String>) -> HttpResponse {
    redirect(&format!("/tags/{}/", kind))
}

#[get("/tags/{kind}/")]
async fn tags_scope(
    state: web::Data<State>,
    web::Path(kind): web::Path<String>,
    mut context: LayoutContext,
) -> impl Responder {
    tag_logic(state, Some(kind), context).await
}

async fn tag_logic(
    state: web::Data<State>,
    kind: Option<String>,
    mut context: LayoutContext,
) -> impl Responder {
    let tag_kind = kind.as_ref().and_then(|kind| TagKind::from_str(&kind).ok());
    let all_tags: Vec<&Tag> = state
        .tags_manager
        .tags
        .iter()
        .filter(|tag| {
            if let Some(kind) = &tag_kind {
                return &tag.1.kind == kind;
            } else {
                return true;
            };
        })
        .take(50)
        .map(|(_, val)| val)
        .collect();

    let last_cards = state.fetcher.fetch(last_25()).await.unwrap();
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

    context.insert("tags", &all_tags);
    context.insert("right_content", &right_tpl);

    let mut group_buttons = vec![(String::new(), String::new(), "Все")];
    for tag in TagKind::iter() {
        group_buttons.push((
            tag.to_string(),
            format!("{}/", tag.to_string()),
            tag.to_description(),
        ));
    }
    context.insert("buttons_base_link", "/tags/");
    context.insert("buttons_active_tag", &kind.unwrap_or("".to_string()));
    context.insert("group_buttons", &group_buttons);

    let meta_title = if let Some(ref tag) = tag_kind {
        match tag {
            TagKind::Event => "Новости фестивалей и ивентов",
            TagKind::Person => "Новости популярных личностей и персон",
            TagKind::Norp => "Новости политических, религиозных и этнических групп",
            TagKind::Org => "Новости организаций и компаний",
            TagKind::Gpe => "Новости стран и регионов",
            TagKind::Product => "Новости и обновления товаров",
            TagKind::Facility => "Новости и обновления объектов",
        }
    } else {
        "Популярные теги на"
    };

    let meta_desc = if let Some(ref tag) = tag_kind {
        match tag {
            TagKind::Event => {
                "HubLoid Ивенты и Фестивали ➤ Последние новости по ивентам и фестивалям"
            }
            TagKind::Person => "HubLoid Топ персон ➤ Последние новости по топовым личностям",
            TagKind::Norp => {
                "HubLoid ➤ Последние новости по политическим, религиозным и этническим группам"
            }
            TagKind::Org => {
                "HubLoid Компании и организации ➤ Последние новости по компаниям и организациям"
            }
            TagKind::Gpe => "HubLoid Страны и регионы ➤ Последние новости по странам и регионам",
            TagKind::Product => "HubLoid Товары и продукты ➤ Последние новости по новым товарам",
            TagKind::Facility => "HubLoid Объекты ➤ Последние новости по объектам",
        }
    } else {
        "HubLoid Популярные теги ➤ Последние новости по популярным тегам"
    };

    context.insert("meta_title", meta_title);
    context.insert("meta_description", &meta_desc);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(state.tera.render("routes/tags.tera", &context).unwrap())
}
