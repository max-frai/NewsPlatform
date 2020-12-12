use futures::Future;
use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    Client,
};
use std::{pin::Pin, sync::Arc};
use tokio::main;
use tokio::sync::Mutex;
use tokio::time::{delay_for, Duration};

use news_general::constants::*;
use news_general::tag::*;

pub mod categorise;
pub mod parse;
pub mod rewrite;
pub mod tag;
pub mod translate;

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

    let parse_client = client.clone();
    let parse_constants = constants.clone();
    tokio::task::spawn(async move {
        loop {
            println!("!!!!!!!!!!!!!!!!!!!!!!! Parse news.......");
            let client = parse_client.clone();
            let constants = parse_constants.clone();
            tokio::task::spawn(async move {
                crate::parse::parse_news(client, constants).await;
            })
            .await;

            delay_for(Duration::from_secs(60)).await;
        }
    });

    let translate_client = client.clone();
    let translate_constants = constants.clone();
    tokio::task::spawn(async move {
        loop {
            println!("!!!!!!!!!!!!!!!!!!! Translate news.......");
            let client = translate_client.clone();
            let constants = translate_constants.clone();
            tokio::task::spawn(async move {
                crate::translate::translate_news(client, constants.clone()).await;
            })
            .await;

            delay_for(Duration::from_secs(60)).await;
        }
    });

    let categorise_client = client.clone();
    let categorise_constants = constants.clone();
    tokio::task::spawn(async move {
        loop {
            println!("!!!!!!!!!!!!!!!!!!! Categorise news.......");
            let client = categorise_client.clone();
            let constants = categorise_constants.clone();
            tokio::task::spawn(async move {
                crate::categorise::categorise_news(client, constants.clone()).await;
            })
            .await;
            delay_for(Duration::from_secs(60)).await;
        }
    });

    let tag_client = client.clone();
    let tag_constants = constants.clone();
    tokio::task::spawn(async move {
        loop {
            println!("!!!!!!!!!!!!!!!!!!! Tag news.......");
            let client = tag_client.clone();
            let constants = tag_constants.clone();
            let tags = tags_manager.clone();
            tokio::task::spawn(async move {
                crate::tag::tag_news(client, constants, tags).await;
            })
            .await;
            delay_for(Duration::from_secs(30)).await;
        }
    });

    let rewrite_client = client.clone();
    let rewrite_constants = constants.clone();
    tokio::task::spawn(async move {
        loop {
            println!("Rewrite news.......");
            let client = rewrite_client.clone();
            let constants = rewrite_constants.clone();
            tokio::task::spawn(async move {
                crate::rewrite::rewrite_news(client, constants.clone()).await;
            })
            .await;
            delay_for(Duration::from_secs(60)).await;
        }
    });

    std::future::pending::<()>().await;
}
