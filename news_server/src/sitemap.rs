use crate::{state::State, templates::make_card_url_raw};
use actix_web::{get, web, HttpResponse, Responder};
use sitemap::{structs::SiteMapEntry, writer::SiteMapWriter};

use anyhow::{Context, Result};
use news_general::{card_queries::all_sitemap, category::Category};
use strum::IntoEnumIterator;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn generate_head_sitemap(state: web::Data<State>) -> anyhow::Result<()> {
    let mut output = Vec::<u8>::new();
    let sitemap_writer = SiteMapWriter::new(&mut output);

    let mut writer = sitemap_writer.start_sitemapindex()?;

    writer.sitemap(format!("{}/posts_sitemap.xml", state.constants.full_domain))?;
    writer.sitemap(format!("{}/tags_sitemap.xml", state.constants.full_domain))?;
    writer.sitemap(format!(
        "{}/category_sitemap.xml",
        state.constants.full_domain
    ))?;

    writer.end()?;

    write_sitemap(&output, "head_sitemap.xml").await
}

pub async fn write_sitemap(output: &Vec<u8>, file_name: &str) -> anyhow::Result<()> {
    let content = std::str::from_utf8(output.as_slice())
        .map(|data| data.to_string())
        .context("Failed to convert sitemap data")?;

    let mut file = File::create(format!("news_templates/static/{}", file_name)).await?;
    file.write_all(&content.as_bytes()).await?;

    Ok(())
}

pub async fn generate_tags_sitemap(state: web::Data<State>) -> anyhow::Result<()> {
    let mut output = Vec::<u8>::new();
    let sitemap_writer = SiteMapWriter::new(&mut output);
    let mut urlwriter = sitemap_writer.start_urlset()?;

    for (_, tag) in state.tags_manager.read().await.tags.iter().take(20000) {
        let url = format!(
            "{}/{}/{}/",
            state.constants.full_domain,
            tag.kind.to_string().to_lowercase(),
            tag.slug
        );

        urlwriter.url(url)?;
    }

    urlwriter.end()?;
    write_sitemap(&output, "tags_sitemap.xml").await
}

pub async fn generate_categories_sitemap(state: web::Data<State>) -> anyhow::Result<()> {
    let mut output = Vec::<u8>::new();
    let sitemap_writer = SiteMapWriter::new(&mut output);
    let mut urlwriter = sitemap_writer.start_urlset()?;

    for category in Category::iter() {
        if category == Category::Unknown {
            continue;
        }

        let url = format!(
            "{}/{}/",
            state.constants.full_domain,
            category.to_string().to_lowercase()
        );

        urlwriter.url(url)?;
    }

    urlwriter.end()?;
    write_sitemap(&output, "category_sitemap.xml").await
}

pub async fn generate_posts_sitemap(state: web::Data<State>) -> anyhow::Result<()> {
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
    write_sitemap(&output, "posts_sitemap.xml").await
}
