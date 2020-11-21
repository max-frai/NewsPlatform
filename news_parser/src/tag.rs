use duct::*;
use lazy_static::lazy_static;
use maplit::hashmap;
use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    sync::Client,
};
use regex::Regex;
use rsmorphy::Source;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*, rsmorphy_dict_ru};
use serde_json::{json, Value};
use slug::slugify;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use std::{env, sync::Mutex};
use strum::IntoEnumIterator;
use strum_macros::EnumString;
use three_set_compare::ThreeSetCompare;
use unicode_segmentation::UnicodeSegmentation;

use news_general::tag::*;

use bson::oid::ObjectId;
use chrono::Utc;
use scraper::Html;
use scraper::Selector;
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

fn _ner(chunks: Vec<String>) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let client = reqwest::blocking::Client::new();
    let result = client
        .post("http://localhost:5555/model")
        .json(&maplit::hashmap! {
            "x" => chunks
        })
        .send()?
        .json::<Vec<Vec<Vec<String>>>>()?;

    let mut final_words = vec![];
    let mut final_tags = vec![];

    for pairs in result.iter() {
        final_words.append(&mut pairs[0].to_owned());
        final_tags.append(&mut pairs[1].to_owned());
    }

    Ok((final_words, final_tags))
}

fn ner(text: &str) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let chars =
        unicode_segmentation::UnicodeSegmentation::graphemes(text, true).collect::<Vec<&str>>();

    // TODO: Don't split on words (use commas, dots, newlines)
    let mut chunks: Vec<String> = vec![];
    for chunk in chars.chunks(1000) {
        chunks.push(chunk.iter().map(|i| i.to_string()).collect());
    }

    _ner(chunks)
}

pub fn tag_news(client: Arc<Client>, tags_manager: Arc<Mutex<TagsManager>>) {
    let db = client.database("news");
    let news_collection = db.collection("news");

    let options = FindOptions::builder()
        .sort(doc! {"date" : 1})
        .limit(1)
        .build();
    let news = news_collection
        .find(
            Some(doc! {
                "rewritten" : true,
                "tagged" : false
            }),
            Some(options),
            // None,
        )
        .unwrap()
        .filter_map(|item| item.ok())
        .collect::<Vec<Document>>();

    if news.is_empty() {
        println!("News to tag is empty, return....");
        return;
    }

    println!("News found to tag: {}", news.len());

    let client = reqwest::blocking::Client::new();

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

        let (words, tags) = ner(&text.trim()).unwrap_or((vec![], vec![]));

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

        // let mut final_tags = vec![];

        for pair in &passed_words {
            let mut word = &pair.0;
            dbg!(&pair.1);
            let kind = TagKind::from_str(&pair.1).unwrap();

            let mut tags_manager_mut = tags_manager.lock().unwrap();
            let tag = tags_manager_mut.search_for_tag(word, kind);
            dbg!(&tag);
        }

        // dbg!(&tags);

        // news_collection.update_one(
        //     doc! {
        //         "_id" : _id
        //     },
        //     doc! {
        //         "$set" : doc!{ "tags" : tags, "tagged" : true }
        //     },
        //     None,
        // );

        // dbg!(model_response);
        // break;
    }
}
