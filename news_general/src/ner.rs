use itertools::Itertools;
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::tag::TagKind;

async fn _ner(
    chunks: Vec<String>,
    service_url: &str,
) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let client = reqwest::Client::new();
    dbg!(&chunks);
    let result = client
        .post(service_url)
        //"http://localhost:5555/model"
        .json(&maplit::hashmap! {
            "x" => chunks
        })
        .send()
        .await
        .unwrap()
        .json::<Vec<Vec<Vec<String>>>>()
        .await?;

    let mut final_words = vec![];
    let mut final_tags = vec![];

    for pairs in result.iter() {
        final_words.append(&mut pairs[0].to_owned());
        final_tags.append(&mut pairs[1].to_owned());
    }

    Ok((final_words, final_tags))
}

async fn ner(mut text: String, service_url: &str) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    text = text
        .replace(">", " ")
        .replace("\n", " ")
        .replace("\u{301}", "");

    // dbg!(&text);

    text = format!("{}{}{}", text, text, text); // ? We get more tags with this

    let chars = unicode_segmentation::UnicodeSegmentation::graphemes(text.as_str(), true)
        .collect::<Vec<&str>>();

    let mut chunks = vec![];
    for chunk in &chars.into_iter().chunks(1100) {
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
    _ner(chunks, service_url).await
}

pub async fn ner_tags(text: String, service_url: &str) -> Option<Vec<(String, TagKind)>> {
    let (words, tags) = ner(text.trim().to_owned(), service_url)
        .await
        .unwrap_or((vec![], vec![]));

    // dbg!(&words);
    // dbg!(&tags);

    if words.is_empty() {
        // println!("No ner words, skip");
        return None;
    }

    let mut words = words.iter();
    let mut tags = tags.iter();

    let mut current_word = String::new();
    let mut passed_words = vec![];
    let mut previous_tag = String::new();

    fn maybe_push_ready_word(
        current_word: &mut String,
        previous_tag: &str,
        passed_words: &mut Vec<(String, TagKind)>,
    ) {
        if current_word.chars().count() > 4 {
            passed_words.push((
                current_word.to_owned(),
                TagKind::from_str(previous_tag).unwrap(),
            ));
            *current_word = String::new();
        }
    }

    while let Some(word) = words.next() {
        let word = word.as_str().to_lowercase();
        let tag = tags.next().unwrap().as_str().to_lowercase();

        // println!("{} - {}", word, tag);

        if tag == "o" {
            maybe_push_ready_word(&mut current_word, &previous_tag, &mut passed_words);
            continue;
        }

        for ok_tag in TagKind::iter() {
            if tag.ends_with(&format!("-{}", ok_tag.to_string().to_lowercase())) {
                // println!("\t tag ok");
                if tag.starts_with("b-") {
                    maybe_push_ready_word(&mut current_word, &previous_tag, &mut passed_words);
                    current_word = word.to_owned();
                } else if tag.starts_with("i-") && !current_word.is_empty() {
                    current_word = format!("{} {}", current_word, word);
                }

                previous_tag = tag.replace("i-", "").replace("b-", "");
                break;
            }
        }
    }

    Some(passed_words)
}
