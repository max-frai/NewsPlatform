use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    Client,
};
use regex::Regex;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use std::{env, sync::Mutex};
use strum::IntoEnumIterator;

use news_general::tag::*;

use futures::stream::StreamExt;
use news_general::constants::AppConfig;
use serde::{Deserialize, Serialize};
use wikipedia::iter::Category;

use itertools::Itertools;
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

async fn _ner(chunks: Vec<String>) -> anyhow::Result<(Vec<String>, Vec<String>)> {
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

async fn ner(mut text: String) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    text = text.replace(">", " ").replace("\n", " ");
    text = format!("{}{}{}", text, text, text); // ? We get more tags with this

    let chars = unicode_segmentation::UnicodeSegmentation::graphemes(text.as_str(), true)
        .collect::<Vec<&str>>();

    let mut chunks = vec![];
    for chunk in &chars.into_iter().chunks(1500) {
        chunks.push(chunk.collect::<Vec<&str>>().concat());
    }

    // while consumed < chars.len() {
    //     println!("Iteration ----------------");
    //     if chars.len() >= MAX_CHUNK_LEN {
    //         let (local_index, symbol) = chars
    //             .iter()
    //             .enumerate()
    //             .skip(consumed + MAX_CHUNK_LEN)
    //             .rev()
    //             .take_while(|(index, letter)| **letter == ".")
    //             .next()
    //             .unwrap();

    //         index = local_index;
    //     }

    //     dbg!(index);

    //     let chunk = chars
    //         .iter()
    //         .skip(consumed)
    //         .take(index)
    //         .cloned()
    //         .collect::<Vec<&str>>();

    //     consumed += chunk.len();
    //     dbg!(consumed);
    //     chunks.push(chunk.concat());
    // }

    // if chars_iter.len() <= MAX_CHUNK_LEN {
    //     chunks.push(chars_iter.map(|s| *s).collect::<Vec<&str>>().concat());
    // } else {
    //     let chunk: Vec<&str> = chars_iter
    //         .skip(MAX_CHUNK_LEN)
    //         .rev()
    //         .take_while(|letter| **letter == ".")
    //         .map(|s| *s)
    //         .collect::<Vec<&str>>();

    //     chunks.push(chunk.concat());
    //     chars_iter.drain(0..chunk.len());
    //     chars_iter.
    // }
    // }

    // dbg!(&chunks);
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
                "slug" : "zhitomirskii-raisovet-soberetsia-na-pervoe-zasedanie-uzhe-1-dekabria"
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

        let (words, tags) = ner(text.trim().to_owned())
            .await
            .unwrap_or((vec![], vec![]));

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

            // println!("{} - {}", word, tag);

            if tag == "o" {
                if current_word.chars().count() > 4 {
                    passed_words.push((current_word.to_owned(), previous_tag.to_owned()));
                    current_word = String::new();
                }
                continue;
            }

            for ok_tag in TagKind::iter() {
                if tag.ends_with(&format!("-{}", ok_tag.to_string().to_lowercase())) {
                    // println!("\t tag ok");
                    if tag.starts_with("b-") {
                        if !current_word.is_empty() {
                            // println!("\tCurrent word is not empty, append");
                            if current_word.chars().count() > 4 {
                                passed_words.push((current_word, previous_tag));
                            }
                            current_word = String::new();
                        }
                        current_word = word.to_owned();
                    } else if tag.starts_with("i-") && !current_word.is_empty() {
                        current_word = format!("{} {}", current_word, word);
                        dbg!(&current_word);
                    }

                    previous_tag = tag.replace("i-", "").replace("b-", "");
                    break;
                }
            }
        }

        // dbg!(&passed_words);

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
