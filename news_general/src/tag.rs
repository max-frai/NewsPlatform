use crate::tag::bson::oid::ObjectId;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use mongodb::bson;
use mongodb::Collection;
use regex::*;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*, rsmorphy_dict_ru, Source};
use scraper::Html;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use three_set_compare::ThreeSetCompare;

pub type Wiki = wikipedia::Wikipedia<wikipedia::http::default::Client>;

#[derive(
    Display, EnumString, EnumIter, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone,
)]
#[strum(serialize_all = "snake_case")]
pub enum TagKind {
    Person,
    Norp,
    Organization,
    Gpe,
    Event,
    Product,
    Facility,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub _id: ObjectId,
    pub kind: TagKind,
    pub summary: String,
    pub sentence: String,
    pub wiki_title: String,
    pub title: String,
    pub image: String,
}

impl Tag {
    pub fn kind(&self) -> String {
        self.kind.to_string().to_lowercase()
    }
}

pub struct TagsManager {
    // (Kind, Title) -> Tag
    tags: HashMap<(TagKind, String), Tag>,
}

pub struct TagsManagerWriter {
    // (Kind, Title) -> Tag
    tags: HashMap<(TagKind, String), Tag>,
    text2wikititle: HashMap<String, String>,
    collection: Collection,
    morph: MorphAnalyzer,
    wiki: Wiki,
    comparator: ThreeSetCompare,
}

lazy_static! {
    static ref FIRST_SENTENCE_RE: Regex = Regex::new(r"— (?P<sentence>.*?)\.").unwrap();
    static ref BRACKETS_RE: Regex = Regex::new(r"\(.*?\)").unwrap();
    static ref SQUARE_BRACKETS_RE: Regex = Regex::new(r"\[.*?\]").unwrap();
}

async fn preload_tags(tags_col: Collection) -> HashMap<(TagKind, String), Tag> {
    let mut tags = tags_col.find(None, None).await.unwrap();
    let mut res = HashMap::new();

    while let Some(tag) = tags.next().await {
        let tag: Tag = bson::from_document(tag.unwrap()).unwrap();
        res.insert((tag.kind.clone(), tag.title.to_owned()), tag);
    }

    res
}

impl TagsManager {
    pub async fn new(tags_col: Collection) -> Self {
        let tags = preload_tags(tags_col.clone()).await;

        Self { tags }
    }
}

impl TagsManagerWriter {
    pub async fn new(tags_col: Collection) -> Self {
        let tags = preload_tags(tags_col.clone()).await;

        let mut wiki = Wiki::default();
        wiki.language = "ru".to_owned();
        wiki.search_results = 1;

        Self {
            tags,
            text2wikititle: HashMap::new(),
            collection: tags_col,
            morph: MorphAnalyzer::from_file(rsmorphy_dict_ru::DICT_PATH),
            wiki,
            comparator: ThreeSetCompare::new(),
        }
    }

    pub fn get_tag(&self, kind: &TagKind, title: &str) -> Option<&Tag> {
        self.tags.get(&(kind.clone(), title.to_owned()))
    }

    pub fn search_for_tag(&mut self, what: &str, kind: TagKind) -> Option<Tag> {
        // let word = if what.contains(" ") {
        //     println!("Search word contains space, split it");
        //     what.split(" ")
        //         .map(|subword| {
        //             println!("\t{}", subword);
        //             self.normal_form(subword).unwrap_or(subword.to_owned())
        //         })
        //         .collect::<Vec<String>>()
        //         .join(" ")
        // } else {
        //     println!("Search word is single word");
        //     what.to_owned()
        // };
        let word = what.to_string();

        let wikititle = if let Some(wikititle) = self.text2wikititle.get(&word) {
            println!("Got wikititle from cache");
            Some(wikititle.to_owned())
        } else {
            println!("Search wiki for: {}; {}", word, kind);
            let search_result = self.wiki.search(&word).unwrap();
            let found = search_result.first().cloned();
            if let Some(ref found_wiki_title) = found {
                self.text2wikititle
                    .insert(word.to_owned(), found_wiki_title.to_owned());
            }

            found
        };

        if wikititle.is_none() {
            return None;
        }

        let mut found = wikititle.unwrap();
        let original_found = found.to_owned();

        println!("Wikititle: {}", original_found);

        found = found.to_lowercase().replace(",", "");
        found = BRACKETS_RE.replace_all(&found, "").to_string();

        let found_tag = self.get_tag(&kind, &found);
        if found_tag.is_some() {
            println!("\treturn tag from cache");
            return found_tag.cloned();
        }

        let similarity = self.comparator.similarity(&found, &word);

        if similarity > 0.5 {
            let page = self.wiki.page_from_title(original_found.to_owned());

            let wiki_html = page.get_html_content().unwrap();
            let document = Html::parse_document(&wiki_html);
            let selector = Selector::parse(".infobox-image img").unwrap();

            let mut image_src = String::new();

            for element in document.select(&selector) {
                image_src = element.value().attr("src").unwrap_or("").to_string();
                break;
            }

            let summary = {
                let mut result = (None, None);
                if let Ok(mut summary) = page.get_summary() {
                    summary = SQUARE_BRACKETS_RE.replace_all(&summary, "").to_string();
                    summary = BRACKETS_RE.replace_all(&summary, "").to_string();
                    if let Some(caps) = FIRST_SENTENCE_RE.captures(&summary) {
                        result = (Some(caps["sentence"].to_owned()), Some(summary));
                    }
                }

                result
            };

            let tag = Tag {
                _id: ObjectId::default(),
                kind: kind.to_owned(),
                sentence: summary.0.unwrap_or(String::new()),
                summary: summary.1.unwrap_or(String::new()),
                wiki_title: original_found.to_owned(),
                title: found.to_owned(),
                image: image_src,
            };

            // println!("Write tag to database");
            let tag_bson = bson::to_document(&tag).unwrap();
            self.tags.insert((kind, found), tag.clone());
            self.collection.insert_one(tag_bson, None);

            return Some(tag);
        }

        None
    }

    fn normal_form(&self, word: &str) -> Option<String> {
        let parsed = self.morph.parse(word);
        if !parsed.is_empty() {
            let lex = parsed[0].lex.clone();
            if let Some(part) = lex.get_tag(&self.morph).pos {
                return if part == Noun {
                    Some(lex.get_normal_form(&self.morph).to_string())
                } else {
                    None
                };
            }
        }

        None
    }
}
