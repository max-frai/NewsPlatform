use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    Client,
};
use std::sync::{Arc, Mutex};
use tokio::main;

use news_general::constants::*;
use news_general::tag::*;

pub mod categorise;
pub mod parse;
pub mod rewrite;
pub mod translate;
// pub mod tag;

#[tokio::main]
async fn main() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    println!("Connect to mongodb");
    let client = Arc::new(
        Client::with_uri_str(&constants.mongodb_url)
            .await
            .expect("Failed to connect mongodb"),
    );

    let db = client.database(&constants.database_name);
    let tags_col = db.collection(&constants.tags_collection_name);

    let tags_manager = Arc::new(Mutex::new(TagsManagerWriter::new(tags_col).await));

    println!("Parse news");
    // crate::parse::parse_news(client, constants.clone()).await;
    // crate::translate::translate_news(client, constants.clone()).await;
    // crate::rewrite::rewrite_news(client, constants.clone()).await;
    crate::categorise::categorise_news(client, constants.clone()).await;
    // crate::tag::tag_news(client, tags_manager);
}
