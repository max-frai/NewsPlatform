use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    sync::Client,
};
use std::sync::Arc;

pub mod categorise;
pub mod parse;
pub mod rewrite;
pub mod translate;

fn main() {
    println!("Connect to mongodb");
    let client = Arc::new(
        Client::with_uri_str("mongodb://127.0.0.1:27017").expect("Failed to connect mongodb"),
    );

    println!("Parse news");
    // crate::translate::translate_news(client);
    // crate::parse::parse_news(client);
    // crate::rewrite::rewrite_news(client);
    crate::categorise::categorise_news(client);
}
