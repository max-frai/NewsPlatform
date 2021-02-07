use mongodb::{
    bson::{doc, document::Document},
    options::FindOptions,
    Client,
};
use std::sync::Arc;

use news_general::tag::*;

use futures::stream::StreamExt;
use news_general::constants::AppConfig;
use serde::{Deserialize, Serialize};

use news_general;

#[derive(Serialize, Deserialize)]
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

pub async fn tag_news(
    client: Arc<Client>,
    constants: Arc<AppConfig>,
    tags_manager: Arc<tokio::sync::Mutex<TagsManagerWriter>>,
) {
    let db = client.database(&constants.database_name);
    let news_collection = db.collection(&constants.cards_collection_name);
    let tags_col = db.collection(&constants.tags_collection_name);

    let options = FindOptions::builder()
        .sort(doc! {"date" : -1})
        .limit(40)
        .build();

    let news_cursor = news_collection
        .find(
            Some(doc! {
                "tagged" : false,
                "lang" : "rus",
                // "slug" : "v-nikolaeva-na-rekonstruktsiiu-perekrestka-v-leskakh-potratiat-eshche-8-4-milliona-griven"
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
        println!("News to tag is empty, return....");
        return;
    }

    println!("News found to tag: {}", news.len());

    for item in news {
        let title = item.get("title").unwrap().as_str().unwrap();
        let text = item
            .get("markdown")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("*", "")
            .replace("--", "-");

        let _id = item.get("_id").unwrap().as_object_id().unwrap();

        // println!("Title:\n{}", title.trim());
        // println!("Text:\n{}", text.trim());

        let mut final_tags = vec![];
        if let Some(ner_tags) = news_general::ner::ner_tags(format!("{}. {}", title, text)).await {
            for pair in &ner_tags {
                let word = pair.0.trim();
                let kind = pair.1.to_owned();

                println!("----------------------");
                dbg!(word);
                dbg!(&kind);

                let mut tags_manager_mut = tags_manager.lock().await;
                // tags_manager_mut.search_for_tag_in_wiki(word, kind);
                if let Some(tag) = tags_manager_mut.search_for_tag_in_wiki(word, kind).await {
                    let tag_bson = bson::to_document(&tag).unwrap();
                    tags_col.insert_one(tag_bson, None).await;

                    if !final_tags.contains(&tag._id) {
                        final_tags.push(tag._id);
                    }
                }
            }
        }

        dbg!(&final_tags);

        news_collection
            .update_one(
                doc! {
                    "_id" : _id
                },
                doc! {
                    "$set" : doc!{ "tags" : final_tags, "tagged" : true }
                },
                None,
            )
            .await
            .expect("Failed to set tags");

        // dbg!(model_response);
        // break;
    }
}
