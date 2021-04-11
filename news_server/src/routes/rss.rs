use crate::{state::State, templates::make_card_url_raw};
use actix_web::{get, web, HttpResponse, Responder};
use news_general::card_queries::last_25;
use rss::ChannelBuilder;

#[get("/feed")]
async fn feed(state: web::Data<State>) -> impl Responder {
    let last_cards = state.fetcher.fetch(last_25(), true).await.unwrap();

    let mut items = vec![];
    for card in last_cards {
        items.push(
            rss::ItemBuilder::default()
                .title(card.title.to_owned())
                .link(format!(
                    "{}{}",
                    state.constants.full_domain,
                    make_card_url_raw(&card, false)
                ))
                .build()
                .unwrap(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("Hubloid")
        .link(state.constants.full_domain.to_owned())
        .description("HubLoid ➤ Все свежие и главные новости Украины и мира на сегодня ✔ Обновление каждую минуту ≡ Ваш персональный хаб новостей!")
        .items(items)
        .build()
        .unwrap();

    HttpResponse::Ok()
        .content_type("text/xml")
        .body(channel.to_string())
}
