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

pub fn translate_news(client: Arc<Client>) {
    let db = client.database("news");
    let news_collection = db.collection("news");

    let options = FindOptions::builder().limit(1).build();
    let news = news_collection
        .find(
            Some(doc! {
                "rewritten" : false,
                "lang" : "ukr"
            }),
            Some(options),
        )
        .unwrap()
        .filter_map(|item| item.ok())
        .collect::<Vec<Document>>();

    if news.is_empty() {
        println!("News to translate is empty, return....");
        return;
    }

    println!("News found to translate: {}", news.len());

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
        let _id = item.get("_id").unwrap().as_object_id().unwrap().to_string();

        let data = format!("{}\n\n{}", title, markdown);

        rewrite_array.push(json! ({
            "tag" : _id,
            "text" : data,
            "title" : ""
        }));
    }

    let json_string =
        serde_json::to_string(&rewrite_array).expect("Failed to construct json for translate");

    // dbg!(&json_string);

    let handle = cmd!(
        format!("./rewritebinary_{}", env::consts::OS),
        "cfd724963e8336a0965bea0c0279cdab2ebb95de846e7019b62e1cd44292ebbcef5dba1efea6f351b8cbb9bb7bebc17ff3e13c35eba00c930cce494e25133724",
        "0",
        "0",
        "uk,ru"
    )
    .stdin_bytes(json_string.to_owned())
    .stdout_capture()
    .start()
    .expect("Failed to start rewritebinary");
    let parse_result = handle.wait().expect("Failed to wait rewritebinary");

    let response_json = std::str::from_utf8(&parse_result.stdout).unwrap();
    if let Ok(json) = serde_json::from_str::<Value>(&response_json) {
        println!("Translate finished, parse results");
        // dbg!(&json);

        for item in json.as_array().unwrap() {
            let tag = item.get("tag").unwrap().as_str().unwrap();
            let text = item.get("text").unwrap().as_str().unwrap();

            let chars = UnicodeSegmentation::graphemes(text, true).collect::<Vec<&str>>();

            let newline_pos_opt = chars.iter().position(|&character| character == "\n");
            if newline_pos_opt.is_none() {
                println!("[!] Newline pos is none");
                continue;
            }

            let newline_pos = newline_pos_opt.unwrap();
            let translated_title = chars
                .iter()
                .take(newline_pos)
                .map(|c| c.to_string())
                .collect::<String>()
                .trim()
                .replace("\"", "")
                .to_string();

            let translated_body = chars
                .iter()
                .skip(newline_pos + 1)
                .map(|c| c.to_string())
                .collect::<String>()
                .trim()
                .to_string();

            let new_slug = slug::slugify(&translated_title);

            let object_id = ObjectId::with_string(&tag).unwrap();

            dbg!(&translated_title);

            if text.trim().is_empty() {
                println!("\t EMPTY TEXT, skip for now this translate");
            }

            news_collection.find_one_and_update(
                doc! {
                    "_id": &object_id
                },
                doc! {
                    "$set" : {
                        "title" : translated_title,
                        "markdown" : translated_body,
                        "slug" : new_slug,
                        "lang" : "rus",
                        "rewritten" : true
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
