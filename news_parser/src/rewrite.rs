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

use mongodb::bson::oid::ObjectId;

pub async fn rewrite_news(client: Arc<Client>, constants: Arc<AppConfig>) {
    let db = client.database(&constants.database_name);
    let news_collection = db.collection(&constants.cards_collection_name);

    let options = FindOptions::builder()
        .sort(doc! { "date" : -1 })
        .limit(30)
        .build();

    let news_cursor = news_collection
        .find(
            Some(doc! {
                "lang" : "rus",
                "tagged" : true,
                "categorised" : true,
                "rewritten" : false
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
        &constants.platform_hash,
        "0",
        "0",
        "",
        "9223"
    )
    .stdin_bytes(json_string.to_owned())
    .stdout_capture()
    .start()
    .expect("Failed to start rewritebinary");
    let parse_result = handle.wait().expect("Failed to wait rewritebinary");

    println!("Got result from rewrite module");

    // dbg!(parse_result);

    let response_json = std::str::from_utf8(&parse_result.stdout).unwrap();

    // dbg!(&response_json);

    if let Ok(json) = serde_json::from_str::<Value>(&response_json) {
        // println!("Rewrite finished, parse results");
        // dbg!(&json);

        for item in json.as_array().unwrap() {
            let tag = item.get("tag").unwrap().as_str().unwrap();
            let text = item.get("text").unwrap().as_str().unwrap();
            let service_text = item.get("service_text").unwrap().as_str().unwrap();
            let title = item.get("title").unwrap().as_str().unwrap();

            let object_id = ObjectId::with_string(&tag).unwrap();

            if text.contains("[...] [...]") {
                println!("Something wrong with this article rewrite [...]");
                continue;
            }

            // Fix whitespace in markdown image
            let rewritten_text = news_general::helper::uppercase_first_letter(&text);
            // rewritten_title = uppercase_first_letter(&rewritten_title).trim().to_string();

            dbg!(&title);
            if rewritten_text.trim().is_empty() {
                println!("\t EMPTY TEXT, skip for now this rewrite");
            }

            news_collection
                .find_one_and_update(
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
                )
                .await
                .expect("Failed to rewrite in db");
        }
    } else {
        println!("Failed to parse json result from rewrite");
        dbg!(response_json);
    }
}
