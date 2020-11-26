use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    Client,
};
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use std::{env, sync::Mutex};
use strum::IntoEnumIterator;

use news_general::tag::*;

use futures::stream::StreamExt;
use news_general::constants::AppConfig;
use serde::{Deserialize, Serialize};
use wikipedia::iter::Category;

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

async fn _ner(chunks: Vec<&str>) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:5555/model")
        .json(&maplit::hashmap! {
            "x" => chunks
        })
        .send()
        .await
        .unwrap()
        .json::<Vec<Vec<Vec<String>>>>()
        .await
        .unwrap();

    let mut final_words = vec![];
    let mut final_tags = vec![];

    for pairs in result.iter() {
        final_words.append(&mut pairs[0].to_owned());
        final_tags.append(&mut pairs[1].to_owned());
    }

    Ok((final_words, final_tags))
}

async fn ner(text: &str) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    // let chars =
    //     unicode_segmentation::UnicodeSegmentation::graphemes(text, true).collect::<Vec<&str>>();

    // TODO: Don't split on words (use commas, dots, newlines)
    // let mut chunks: Vec<String> = vec![];
    // for chunk in chars.chunks(1000) {
    //     chunks.push(chunk.iter().map(|i| i.to_string()).collect());
    // }

    let chunks = text.split(". ").collect();
    _ner(chunks).await
}

pub async fn tag_news(
    client: Arc<Client>,
    constants: Arc<AppConfig>,
    tags_manager: Arc<Mutex<TagsManagerWriter>>,
) {
    let db = client.database(&constants.database_name);
    let news_collection = db.collection(&constants.cards_collection_name);

    let options = FindOptions::builder()
        .sort(doc! {"date" : 1})
        .limit(100)
        .build();

    let news_cursor = news_collection
        .find(
            Some(doc! {
                // "rewritten" : true,
                // "tagged" : false,
                "slug" : "ispolkom-zhitomirskogo-gorsoveta-razreshil-shkolam-samim-reshat-idti-li-na-karantin"
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
        let text = item
            .get("markdown")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("*", "")
            .replace("--", "-");

        let _id = item.get("_id").unwrap().as_object_id().unwrap();

        // println!("Text:\n{}", text.trim());

        let (words, tags) = ner(&text.trim()).await.unwrap_or((vec![], vec![]));

        // dbg!(&words);
        // dbg!(&tags);

        if words.is_empty() {
            println!("No ner words, skip");
            continue;
        }

        let mut words = words.iter();
        let mut tags = tags.iter();

        let mut current_word = String::new();
        let mut passed_words = vec![];
        let mut previous_tag = String::new();

        while let Some(word) = words.next() {
            let word = word.as_str().to_lowercase();
            let tag = tags.next().unwrap().as_str().to_lowercase();

            if tag == "o" {
                continue;
            }

            // println!("{} - {}", word, tag);

            for ok_tag in TagKind::iter() {
                if tag.ends_with(&format!("-{}", ok_tag.to_string().to_lowercase())) {
                    // println!("\t tag ok");
                    if tag.starts_with("b-") {
                        if !current_word.is_empty() {
                            if current_word.chars().count() > 3 {
                                passed_words.push((
                                    current_word,
                                    previous_tag.replace("i-", "").replace("b-", ""),
                                ));
                            }
                            current_word = String::new();
                        }
                        current_word = word.to_owned();
                    } else if tag.starts_with("i-") && !current_word.is_empty() {
                        current_word = format!("{} {}", current_word, word);
                    }

                    previous_tag = tag;
                    break;
                }
            }
        }

        // dbg!(passed_words);

        let mut final_tags = vec![];
        for pair in &passed_words {
            let word = &pair.0;
            let kind = TagKind::from_str(&pair.1).unwrap();

            // dbg!(word);
            // dbg!(&kind);

            let mut tags_manager_mut = tags_manager.lock().unwrap();
            if let Some(tag) = tags_manager_mut.search_for_tag(word, kind).await {
                if !final_tags.contains(&tag._id) {
                    final_tags.push(tag._id);
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
