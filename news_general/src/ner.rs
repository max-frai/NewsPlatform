use anyhow::*;
use duct::*;
// use itertools::Itertools;
use std::str::FromStr;
// use strum::IntoEnumIterator;

use crate::tag::TagKind;

async fn ner(mut text: String) -> anyhow::Result<Vec<(String, String)>> {
    text = text
        .replace(">", " ")
        .replace("\n", " ")
        .replace("\u{301}", "");

    let handle = cmd!(format!("python3"), "news_ner/process.py")
        .stdin_bytes(text)
        .stdout_capture()
        .start()
        .expect("Failed to start ner process");

    let parse_result = handle.wait().expect("Failed to wait ner process");
    let response_json = std::str::from_utf8(&parse_result.stdout).unwrap();

    // dbg!(&response_json);

    serde_json::from_str::<Vec<(String, String)>>(&response_json)
        .context("Failed to parse ner json")
}

pub async fn ner_tags(text: String) -> Option<Vec<(String, TagKind)>> {
    let pairs = ner(text.trim().to_owned())
        .await
        .unwrap_or(vec![])
        .iter()
        .map(|(word, tag)| {
            (
                word.to_lowercase(),
                TagKind::from_str(tag.to_lowercase().as_str()).unwrap(),
            )
        })
        .collect::<Vec<(String, TagKind)>>();

    dbg!(&pairs);

    if pairs.is_empty() {
        // println!("No ner words, skip");
        return None;
    }

    Some(pairs)
}
