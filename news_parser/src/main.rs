use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    sync::Client,
};
use std::sync::{Arc, Mutex};

use news_general::constants::*;
use news_general::tag::*;

pub mod categorise;
pub mod parse;
pub mod rewrite;
pub mod tag;
pub mod translate;

fn main() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    println!("Connect to mongodb");
    let client = Arc::new(
        Client::with_uri_str("mongodb://127.0.0.1:27017").expect("Failed to connect mongodb"),
    );

    let db = client.database(&constants.database_name);
    let tags_col = db.collection(&constants.tags_collection_name);

    let tags_manager = Arc::new(Mutex::new(TagsManager::new(tags_col)));

    println!("Parse news");
    // crate::parse::parse_news(client);
    // crate::translate::translate_news(client);
    // crate::rewrite::rewrite_news(client);
    // crate::categorise::categorise_news(client);
    crate::tag::tag_news(client, tags_manager);
}
