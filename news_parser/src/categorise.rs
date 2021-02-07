use duct::*;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, document::Document},
    options::FindOptions,
    Client,
};
use news_general::{category::Category, constants::AppConfig};
use std::io::Write;
use std::sync::Arc;
use std::{collections::HashSet, env};

use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct ClusteringItem {
    pub category: String,
    pub timestamp: i64,
    pub description: String,
    pub site_name: String,
    pub text: String,
    pub title: String,
    pub url: String,
    pub file_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusteringThread {
    pub articles: Vec<String>,
    pub category: String,
}

pub async fn categorise_news(client: Arc<Client>, constants: Arc<AppConfig>) {
    let db = client.database(&constants.database_name);
    let news_collection = db.collection(&constants.cards_collection_name);

    let options = FindOptions::builder()
        .sort(doc! {"date" : -1})
        .limit(400)
        .build();

    let news_cursor = news_collection
        .find(
            Some(doc! {
                "categorised" : false,
                "lang" : "rus"
            }),
            Some(options),
            // None,
        )
        .await
        .unwrap();

    let news_docs = news_cursor
        .collect::<Vec<Result<Document, mongodb::error::Error>>>()
        .await;

    let news = news_docs
        .iter()
        .filter_map(|item| item.as_ref().ok())
        .collect::<Vec<&Document>>();

    if news.is_empty() {
        println!("News to categorise is empty, return....");
        return;
    }

    println!("News found to categorise: {}", news.len());

    let mut items = vec![];
    let mut all_ids = HashSet::new();
    for item in news {
        let title = item.get("title").unwrap().as_str().unwrap().to_string();
        let text = item.get("markdown").unwrap().as_str().unwrap().to_string();
        let _id = item.get("_id").unwrap().as_object_id().unwrap();

        all_ids.insert(_id.to_owned());

        items.push(ClusteringItem {
            category: String::from(""),
            timestamp: Utc::now().timestamp(),
            description: text.to_string().replace("*", "").trim().to_string(),
            site_name: String::from(""),
            text,
            file_name: _id.to_string(),
            title,
            url: String::from(""),
        });
    }

    // dbg!(&items);

    let mut file = std::fs::File::create("categories.json").unwrap();
    let json_str = serde_json::to_string(&items).unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
    file.sync_all().unwrap();

    let handle = cmd!(
        format!("./nlp_{}", env::consts::OS),
        "categories",
        "categories.json"
    )
    .stdout_capture()
    .start()
    .expect("Failed to start nlp");
    let parse_result = handle.wait().expect("Failed to wait nlp");

    let response_json = std::str::from_utf8(&parse_result.stdout).unwrap();
    let threads = serde_json::from_str::<Vec<ClusteringThread>>(response_json).unwrap();

    dbg!(&threads);

    println!("Update threads categories...");
    for thread in threads {
        if !thread.articles.is_empty() {
            let articles_ids: Vec<ObjectId> = thread
                .articles
                .iter()
                .map(|_id| ObjectId::with_string(_id).unwrap())
                .collect();

            let category = Category::from_str(&thread.category).unwrap();
            let category_bson = bson::to_bson(&category).unwrap();

            println!("Set category for {} articles", articles_ids.len());
            dbg!(&category);

            for _id in &articles_ids {
                all_ids.remove(_id);
            }

            news_collection
                .update_many(
                    doc! {
                        "_id" : doc!{ "$in" : articles_ids }
                    },
                    doc! {
                        "$set" : doc!{ "category" : category_bson, "categorised" : true }
                    },
                    None,
                )
                .await
                .expect("Failed to set categories");
        }
    }

    println!("Mark left news as not proper: {}", all_ids.len());
    let unknown_bson = bson::to_bson(&Category::Unknown).unwrap();
    news_collection
        .update_many(
            doc! {
                "_id" : doc!{ "$in" : all_ids.into_iter().collect::<Vec<ObjectId>>() }
            },
            doc! {
                "$set" : doc!{ "category" : unknown_bson, "categorised" : true }
            },
            None,
        )
        .await
        .expect("Failed to set UNKNOWN categories");
}
