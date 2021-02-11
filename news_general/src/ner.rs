use anyhow::*;
// use duct::*;
// use itertools::Itertools;
use std::str::FromStr;
// use strum::IntoEnumIterator;

use crate::tag::TagKind;

async fn ner(ner_link: &str, mut text: String) -> anyhow::Result<Vec<(String, String)>> {
    text = text
        .replace(">", " ")
        .replace("\n", " ")
        .replace("\u{301}", "");

    let client = reqwest::Client::new();
    let response_json = client
        .post(ner_link)
        .body(text)
        .send()
        .await?
        .text()
        .await?;

    serde_json::from_str::<Vec<(String, String)>>(&response_json)
        .context("Failed to parse ner json")
}

pub async fn ner_tags(ner_link: &str, text: String) -> Option<Vec<(String, TagKind)>> {
    let pairs = ner(ner_link, text.trim().to_owned())
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

    // dbg!(&pairs);

    if pairs.is_empty() {
        // println!("No ner words, skip");
        return None;
    }

    Some(pairs)
}
