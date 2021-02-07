use duct::*;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, document::Document},
    options::FindOptions,
    Client,
};
use news_general::constants::AppConfig;
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

use bson::oid::ObjectId;

pub async fn translate_news(client: Arc<Client>, constants: Arc<AppConfig>) {
    let db = client.database(&constants.database_name);
    let news_collection = db.collection(&constants.cards_collection_name);

    let options = FindOptions::builder()
        .sort(doc! {"date" : -1})
        .limit(100)
        .build();

    let news_cursor = news_collection
        .find(
            Some(doc! {
                "rewritten" : false,
                "lang" : "ukr"
            }),
            Some(options),
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

    if news.is_empty() || news.len() < 40 {
        println!("News to translate or it's less than 40 or it is empty, return....");
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
        &constants.platform_hash,
        "0",
        "0",
        "uk,ru",
        "9224"
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

            let new_slug = str_slug::slug(&translated_title);

            let object_id = ObjectId::with_string(&tag).unwrap();

            dbg!(&translated_title);

            if text.trim().is_empty() {
                println!("\t EMPTY TEXT, skip for now this translate");
            }

            news_collection
                .find_one_and_update(
                    doc! {
                        "_id": &object_id
                    },
                    doc! {
                        "$set" : {
                            "title" : translated_title,
                            "markdown" : translated_body,
                            "slug" : new_slug,
                            "lang" : "rus"
                        }
                    },
                    None,
                )
                .await
                .expect("Failed to translate");
        }
    } else {
        println!("Failed to parse json result from rewrite");
        dbg!(response_json);
    }
}
