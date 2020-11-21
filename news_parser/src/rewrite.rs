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
use std::process::{Command, Stdio};
use std::{collections::HashMap, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;

use bson::oid::ObjectId;
use chrono::Utc;

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn rewrite_news(client: Arc<Client>) {
    let db = client.database("news");
    let news_collection = db.collection("news");

    let options = FindOptions::builder()
        .sort(doc! { "date" : 1 })
        .limit(200)
        .build();
    let news = news_collection
        .find(
            Some(doc! {
                "lang" : "rus",
                "rewritten" : false
            }),
            Some(options),
            // None,
        )
        .unwrap()
        .filter_map(|item| item.ok())
        .collect::<Vec<Document>>();

    if news.is_empty() {
        println!("News to rewrite is empty, return....");
        return;
    }

    println!("News found to rewrite: {}", news.len());

    let mut rewrite_array = vec![];

    for item in news {
        let title = item
            .get("title")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("«", "\"")
            .replace("»", "\"")
            .replace("\"", "");
        let markdown = item.get("markdown").unwrap().as_str().unwrap().to_string();
        // let markdown = format!("{}. \n\n{}", title, text);
        let _id = item.get("_id").unwrap().as_object_id().unwrap().to_string();

        rewrite_array.push(json! ({
            "tag" : _id,
            "title" : title,
            "text" : markdown
        }));
    }

    let json_string =
        serde_json::to_string(&rewrite_array).expect("Failed to construct json for rewrite");

    // dbg!(&json_string);

    let handle = cmd!(
        format!("./rewritebinary_{}", env::consts::OS),
        "cfd724963e8336a0965bea0c0279cdab2ebb95de846e7019b62e1cd44292ebbcef5dba1efea6f351b8cbb9bb7bebc17ff3e13c35eba00c930cce494e25133724",
        "1",
        "0",
        ""
    )
    .stdin_bytes(json_string.to_owned())
    .stdout_capture()
    .start()
    .expect("Failed to start rewritebinary");
    let parse_result = handle.wait().expect("Failed to wait rewritebinary");

    println!("Got result from rewrite module");

    // dbg!(parse_result);

    let response_json = std::str::from_utf8(&parse_result.stdout).unwrap();

    if let Ok(json) = serde_json::from_str::<Value>(&response_json) {
        println!("Rewrite finished, parse results");
        // dbg!(&json);

        for item in json.as_array().unwrap() {
            let tag = item.get("tag").unwrap().as_str().unwrap();
            let text = item.get("text").unwrap().as_str().unwrap();
            let service_text = item.get("service_text").unwrap().as_str().unwrap();
            let title = item.get("title").unwrap().as_str().unwrap();

            let object_id = ObjectId::with_string(&tag).unwrap();

            // Fix whitespace in markdown image
            let mut rewritten_text = uppercase_first_letter(&text);
            // rewritten_title = uppercase_first_letter(&rewritten_title).trim().to_string();

            dbg!(&title);
            if rewritten_text.trim().is_empty() {
                println!("\t EMPTY TEXT, skip for now this rewrite");
            }

            // Fix marks
            // let marks = &id2marks[tag];
            // let mark_re = regex::Regex::new(r"\{ ?(\d)+ ?\}").unwrap();
            // let multi_whitespace_re = regex::Regex::new(r" {2,}").unwrap();

            // rewritten_text = mark_re
            //     .replace_all(&rewritten_text, |caps: &regex::Captures| {
            //         if let Ok(index) = caps[1].parse::<usize>() {
            //             if index > marks.len() {
            //                 println!("index more than caps");
            //             }
            //             marks.get(index).cloned().unwrap_or_default()
            //         } else {
            //             String::default()
            //         }
            //     })
            //     .to_string();

            // rewritten_text = multi_whitespace_re
            //     .replace_all(&rewritten_text, " ")
            //     .to_string();

            // dbg!(&rewritten_title);

            news_collection.find_one_and_update(
                doc! {
                    "_id": &object_id
                },
                doc! {
                    "$set" : {
                        // "title" : rewritten_title.to_owned(),
                        // "slug" : rewritten_slug,
                        "markdown" : rewritten_text.to_owned(),
                        "service_markdown" : service_text.to_owned(),
                        "rewritten" : true,
                    }
                },
                None,
            );
        }
    } else {
        println!("Failed to parse json result from rewrite");
        dbg!(response_json);
    }
}
