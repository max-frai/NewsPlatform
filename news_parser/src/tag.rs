use duct::*;
use lazy_static::lazy_static;
use maplit::hashmap;
use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    sync::Client,
};
use regex::Regex;
use serde_json::{json, Value};
use slug::slugify;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::{collections::HashMap, sync::Arc};
use three_set_compare::ThreeSetCompare;
use unicode_segmentation::UnicodeSegmentation;

use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use wikipedia::iter::Category;

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

lazy_static! {
    static ref OK_TAGS: Vec<&'static str> = vec![
        "person",
        "norp",
        "organization",
        "gpe",
        "event",
        "law",
        "product",
        "facility",
    ];
    // static ref TAG_TO_DESC: HashMap<&'static str, &'static str> = hashmap! {
    //     "gpe" => "страна",
    //     "person" => "человек",
    //     "organization" => "организация"
    // };
}

pub fn tag_news(client: Arc<Client>) {
    let db = client.database("news");
    let news_collection = db.collection("news");

    let options = FindOptions::builder().limit(3).build();
    let news = news_collection
        .find(
            Some(doc! {
                "rewritten" : true,
                "tags" : doc!{ "$eq" : Vec::<String>::new() }
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
            .replace("*", "");

        let _id = item.get("_id").unwrap().as_object_id().unwrap();

        println!("Text:\n{}", text.trim());

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

            for ok in OK_TAGS.iter() {
                if tag.ends_with(&format!("-{}", ok)) {
                    if tag.starts_with("b-") {
                        if !current_word.is_empty() {
                            passed_words.push((
                                current_word,
                                previous_tag.replace("i-", "").replace("b-", ""),
                            ));
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

        let mut wiki = Wiki::default();
        wiki.language = "ru".to_owned();
        wiki.search_results = 1;

        pub type Wiki = wikipedia::Wikipedia<wikipedia::http::default::Client>;

        let brackets_re = regex::Regex::new(r"\(.*?\)").unwrap();
        let mut tags = vec![];
        let mut already_searched = vec![];

        let comparator = ThreeSetCompare::new();

        for pair in &passed_words {
            let word = &pair.0;
            let tag = &pair.1;

            // let helper = TAG_TO_DESC.get(tag.as_str()).unwrap_or(&"");
            // let query = if helper.is_empty() {
            //     word.to_owned()
            // } else {
            //     format!("{} {}", word, helper)
            // };

            if already_searched.contains(&word) {
                continue;
            }

            // println!("Search wiki for: {}; {}", word, tag);
            let search_result = wiki.search(&word).unwrap();
            if let Some(found) = search_result.first() {
                let mut found = found.to_owned();
                found = found.replace(",", "");
                found = brackets_re.replace_all(&found, "").to_string();
                println!("Found: {}; Original: {}", found, word);
                let similarity = comparator.similarity(&found, word);
                // dbg!(similarity);
                // println!("---------");

                if !tags.contains(&found) && similarity > 0.5 {
                    tags.push(found);
                }
            }

            already_searched.push(word);
        }

        dbg!(&tags);

        // news_collection.update_one(
        //     doc! {
        //         "_id" : _id
        //     },
        //     doc! {
        //         "$set" : doc!{ "tags" : tags }
        //     },
        //     None,
        // );

        // dbg!(model_response);
        // break;
    }
}
