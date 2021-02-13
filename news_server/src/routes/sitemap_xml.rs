use crate::{state::State, templates::make_card_url_raw};
use actix_web::{get, web, HttpResponse, Responder};

use anyhow::{Context, Result};
use news_general::card_queries::all_sitemap;
use sitemap::writer::SiteMapWriter;

pub(crate) async fn generate_sitemap_xml(state: web::Data<State>) -> Result<String> {
    let cards = state.fetcher.fetch(all_sitemap(), false).await.unwrap();

    let mut output = Vec::<u8>::new();
    let sitemap_writer = SiteMapWriter::new(&mut output);
    let mut urlwriter = sitemap_writer.start_urlset()?;

    for card in cards {
        let url = format!(
            "{}{}",
            state.constants.full_domain,
            make_card_url_raw(&card, false)
        );

        urlwriter.url(url)?;
    }

    urlwriter.end()?;

    std::str::from_utf8(output.as_slice())
        .map(|data| data.to_string())
        .context("Failed to convert sitemap data")
}

#[get("/sitemap.xml")]
async fn sitemap_xml(state: web::Data<State>) -> actix_web::Result<impl Responder> {
    {
        let sitemap = state.sitemap.read().await;
        if !sitemap.is_empty() {
            return Ok(HttpResponse::Ok().content_type("text/xml").body(&*sitemap));
        }
    }

    let mut sitemap_mut = state.sitemap.write().await;
    *sitemap_mut = generate_sitemap_xml(state.clone()).await.unwrap();

    return Ok(HttpResponse::Ok()
        .content_type("text/javascript")
        .body(&*sitemap_mut));
}
