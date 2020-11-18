use duct::*;
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
use unicode_segmentation::UnicodeSegmentation;

use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelJsonRequest {
    pub x: Vec<String>,
}

pub fn tag_news(client: Arc<Client>) {
    let db = client.database("news");
    let news_collection = db.collection("news");

    let options = FindOptions::builder().limit(50).build();
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
        let text = item.get("markdown").unwrap().as_str().unwrap().to_string();
        let _id = item.get("_id").unwrap().as_object_id().unwrap();

        let json_req = ModelJsonRequest { x: vec![text] };
        let json_req_str = serde_json::to_string(&json_req).unwrap();
        let model_response: Value = client
            .post("http://127.0.0.1:5555/model")
            .body(json_req_str)
            .send()
            .unwrap()
            .json()
            .unwrap();

        let mut array_results = model_response
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_array()
            .unwrap()
            .iter();

        let mut words = array_results.next().unwrap().as_array().unwrap().iter();
        let mut tags = array_results.next().unwrap().as_array().unwrap().iter();

        let ok_tags = vec![
            "person",
            "norp",
            "organization",
            "gpe",
            "event",
            "law",
            "product",
        ];

        let mut current_word = String::new();
        let mut passed_words = vec![];
        let mut previous_tag = String::new();

        while let Some(word) = words.next() {
            let word = word.as_str().unwrap().to_lowercase();
            let tag = tags.next().unwrap().as_str().unwrap().to_lowercase();

            if tag == "o" {
                continue;
            }

            // println!("{} - {}", word, tag);

            for ok in &ok_tags {
                if tag.ends_with(&format!("-{}", ok)) {
                    if tag.starts_with("b-") {
                        if !current_word.is_empty() {
                            passed_words.push([current_word, previous_tag]);
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

        dbg!(passed_words);

        // dbg!(model_response);
        break;
    }
}
