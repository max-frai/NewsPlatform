use crate::{card::Card, tag::bson::oid::ObjectId};
use bson::doc;
use chrono::prelude::*;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use maplit::hashmap;
use mongodb::bson;
use mongodb::Collection;
use regex::*;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*, rsmorphy_dict_ru, Source};
use scraper::Html;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::ToString;
use std::{collections::HashMap, fs::File};
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
    Org,
    Gpe,
    Event,
    Product,
    Facility,
}

impl TagKind {
    pub fn to_description(&self) -> &'static str {
        match self {
            TagKind::Person => "Люди",
            TagKind::Norp => "Группы",
            TagKind::Org => "Организации",
            TagKind::Gpe => "Локации",
            TagKind::Event => "События",
            TagKind::Product => "Продукты",
            TagKind::Facility => "Объекты",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub _id: ObjectId,
    pub kind: TagKind,
    pub summary: String,
    pub sentence: String,
    pub wiki_title: String,
    pub slug: String,
    pub title: String,
    pub image: String,
}

impl Tag {
    pub fn kind(&self) -> String {
        self.kind.to_string().to_lowercase()
    }
}

pub struct TagsManager {
    news_col: Collection,
    pub tags: HashMap<ObjectId, Tag>,
    // (Kind, slug) -> Tag
    tags_lookup: HashMap<(TagKind, String), Tag>,
}

pub struct TagsManagerWriter {
    // (Kind, Title) -> Tag
    tags: HashMap<(TagKind, String), Tag>,
    // (Kind, WikiTitle) -> Tag or None if wrong
    text2wikititle: HashMap<(TagKind, String), Option<String>>,

    // morph: MorphAnalyzer,
    comparator: ThreeSetCompare,
}

lazy_static! {
    static ref FIRST_SENTENCE_RE: Regex =
        RegexBuilder::new(r"— (?P<sentence1>.*?)\.((?P<sentence2>.*?)\.)?")
            .multi_line(true)
            .build()
            .unwrap();
    static ref BRACKETS_RE: Regex = Regex::new(r"\(.*?\)").unwrap();
    static ref SQUARE_BRACKETS_RE: Regex = Regex::new(r"\[.*?\]").unwrap();
}

impl TagsManager {
    pub async fn new(tags_col: Collection, news_col: Collection) -> Self {
        let mut raw_tags = tags_col.find(None, None).await.unwrap();
        let mut tags = HashMap::new();
        let mut tags_lookup = HashMap::new();

        while let Some(tag) = raw_tags.next().await {
            let tag: Tag = bson::from_document(tag.unwrap()).unwrap();
            tags.insert(tag._id.to_owned(), tag.to_owned());
            tags_lookup.insert((tag.kind.clone(), tag.slug.to_owned()), tag);
        }

        Self {
            news_col,
            tags,
            tags_lookup,
        }
    }

    pub async fn find(&self, kind: TagKind, slug: &str) -> Option<&Tag> {
        self.tags_lookup.get(&(kind, slug.to_owned()))
    }
    pub async fn fill_card_tags(&self, card: &mut Card) {
        card.filled_tags = vec![];

        for _id in &card.tags {
            if let Some(tag) = self.tags.get(&_id) {
                card.filled_tags.push(tag.clone());
            }
        }
    }

    pub async fn get_popular_by_kind(&self, kind: TagKind) -> Vec<Tag> {
        let mut tags = vec![];
        for (_, tag) in &self.tags {
            if tag.kind == kind {
                let count = self
                    .news_col
                    .count_documents(
                        doc! {
                            "date" : {
                                "$gte" : Utc::now() - chrono::Duration::days(2)
                            },
                            "tags" : tag._id.clone()
                        },
                        None,
                    )
                    .await
                    .unwrap();

                tags.push((tag, count));
            }
        }

        tags.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        tags.iter()
            .take(4)
            .map(|item| item.0.to_owned())
            .collect::<Vec<Tag>>()
    }
}

impl TagsManagerWriter {
    pub async fn new(tags_col: Collection) -> Self {
        let mut raw_tags = tags_col.find(None, None).await.unwrap();
        let mut tags = HashMap::new();

        while let Some(tag) = raw_tags.next().await {
            let tag: Tag = bson::from_document(tag.unwrap()).unwrap();
            tags.insert((tag.kind.clone(), tag.title.to_owned()), tag);
        }

        Self {
            tags,
            text2wikititle: Self::load_text2wikititle(),
            // morph: MorphAnalyzer::from_file(rsmorphy_dict_ru::DICT_PATH),
            comparator: ThreeSetCompare::new(),
        }
    }

    pub fn get_tag(&self, kind: &TagKind, title: &str) -> Option<&Tag> {
        // println!("Get tag: {}; {}", kind, title);
        self.tags.get(&(kind.clone(), title.to_owned()))
    }

    async fn verify_found_wikititle_ok_by_tag(
        &self,
        summary: &str,
        should_be_tag: TagKind,
        service_url: &str,
    ) -> Option<()> {
        if let Some(tags) = crate::ner::ner_tags(summary.to_owned(), service_url).await {
            if !tags.is_empty() {
                // println!("Check: {} == {}", should_be_tag, tags.first().unwrap().1);
                if should_be_tag == tags.first().unwrap().1 {
                    return Some(());
                }
            }
        }

        None
    }

    fn load_text2wikititle() -> HashMap<(TagKind, String), Option<String>> {
        use std::io::Read;

        let mut file = File::open("text2wikititle").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let mut result = hashmap! {};
        for line in contents.split("\n") {
            if line.trim() == "" {
                continue;
            }

            let mut split = line.split("=");
            let kind = TagKind::from_str(split.next().unwrap()).unwrap();
            let word = split.next().unwrap();

            let wiki = split.next().map(|title| {
                if title == "NONE" {
                    None
                } else {
                    Some(title.to_owned())
                }
            });

            result.insert((kind.to_owned(), word.to_owned()), wiki.flatten());
        }

        // dbg!(&result);

        result
    }

    fn save_text2wikititle(&mut self, text: &str, wikititle: Option<&str>, kind: TagKind) {
        use std::io::prelude::*;

        if self
            .text2wikititle
            .contains_key(&(kind.to_owned(), text.to_owned()))
        {
            return;
        }

        self.text2wikititle.insert(
            (kind.to_owned(), text.to_owned()),
            wikititle.map(|i| i.to_string()),
        );

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("text2wikititle")
            .unwrap();

        // let mut f = File::with_options()
        //     .append(true)
        //     .create(true)
        //     .open("text2wikititle")
        //     .unwrap();

        let result = format!("{}={}={}\n", kind, text, wikititle.unwrap_or("NONE"));
        f.write_all(result.as_bytes()).unwrap();
    }

    pub async fn search_for_tag_in_wiki(
        &mut self,
        what: &str,
        kind: TagKind,
        service_url: &str,
    ) -> Option<Tag> {
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

        let mut wiki = Wiki::default();
        wiki.language = "ru".to_owned();
        wiki.search_results = 1;

        let word = what.to_string();

        let wikititle =
            if let Some(wikititle) = self.text2wikititle.get(&(kind.to_owned(), word.to_owned())) {
                // println!("Got wikititle from cache");
                Some(wikititle.to_owned())
            } else {
                println!("Search wiki for: {}; {}", word, kind);
                let search_result = wiki.search(&word).unwrap();
                // dbg!(&search_result);
                // search_result
                //     .sort_by(|a, b| a.chars().count().partial_cmp(&b.chars().count()).unwrap());

                Some(search_result.first().cloned())
            }
            .flatten();

        if wikititle.is_none() {
            self.save_text2wikititle(&word, None, kind.to_owned());
            return None;
        }

        let mut found = wikititle.unwrap();
        let original_found = found.to_owned();

        println!("FOUND FIRST Wikititle: {}", original_found);

        found = found.to_lowercase().replace(",", "");
        found = BRACKETS_RE.replace_all(&found, "").to_string();

        let found_tag = self.get_tag(&kind, &found);
        if found_tag.is_some() {
            println!("\tReturn tag directly from cache");
            return found_tag.cloned();
        }

        let similarity = self.comparator.similarity(&found, &word);
        // dbg!(similarity);

        if similarity >= 0.4 {
            let page = wiki.page_from_title(original_found.to_owned());

            let summary = {
                let mut result = (String::new(), String::new());
                if let Ok(mut summary) = page.get_summary() {
                    summary = SQUARE_BRACKETS_RE.replace_all(&summary, "").to_string();
                    summary = BRACKETS_RE.replace_all(&summary, "").to_string();
                    summary = summary.replace("\n", " ");
                    // dbg!(&summary);
                    if let Some(caps) = FIRST_SENTENCE_RE.captures(&summary) {
                        let mut sentences = vec![];

                        // dbg!(&caps);

                        let first = caps
                            .name("sentence1")
                            .map(|group| group.as_str())
                            .unwrap_or("");
                        let second = caps
                            .name("sentence2")
                            .map(|group| group.as_str())
                            .unwrap_or("");

                        // dbg!(&first);
                        // dbg!(&second);

                        // Sentence too long, so there is some meta information
                        if first.chars().count() < 250 {
                            sentences.push(first);
                        }
                        if second.chars().count() < 250 {
                            sentences.push(second);
                        }

                        let sentence =
                            crate::helper::uppercase_first_letter(sentences.join(". ").trim());
                        result = (sentence, summary);
                    }
                }

                result
            };

            if similarity < 0.9 {
                if self
                    .verify_found_wikititle_ok_by_tag(&summary.1, kind.to_owned(), service_url)
                    .await
                    .is_none()
                {
                    println!("\t\tTHIS WIKI TAG KIND IS WRONG, skip");
                    self.save_text2wikititle(&word, None, kind.to_owned());
                    return None;
                }
            } else {
                println!("Dont check found tag kind on wiki, SIMILARITY >= 0.9");
            }

            let wiki_html = page.get_html_content().unwrap();
            let document = Html::parse_document(&wiki_html);

            fn get_image(document: &Html, selector: &'static str) -> Option<String> {
                let selector = Selector::parse(selector).unwrap();
                for element in document.select(&selector) {
                    return Some(element.value().attr("src").unwrap_or("").to_string());
                }
                None
            }

            let image_src = get_image(&document, ".infobox-image img")
                .or(get_image(&document, ".infobox img"))
                .or(get_image(&document, "img.thumbimage"));

            if image_src.is_none() {
                self.save_text2wikititle(&word, None, kind.to_owned());
                return None;
            }

            self.save_text2wikititle(&word, Some(&original_found), kind.to_owned());

            let tag = Tag {
                _id: ObjectId::default(),
                kind: kind.to_owned(),
                sentence: summary.0,
                summary: summary.1,
                wiki_title: original_found.to_owned(),
                title: found.to_owned(),
                slug: slug::slugify(found.to_owned()),
                image: image_src.unwrap(), // Safe
            };

            // println!("Write tag to database");
            self.tags.insert((kind, found), tag.clone());

            return Some(tag);
        }

        None
    }

    // fn normal_form(&self, word: &str) -> Option<String> {
    //     let parsed = self.morph.parse(word);
    //     if !parsed.is_empty() {
    //         let lex = parsed[0].lex.clone();
    //         if let Some(part) = lex.get_tag(&self.morph).pos {
    //             return if part == Noun {
    //                 Some(lex.get_normal_form(&self.morph).to_string())
    //             } else {
    //                 None
    //             };
    //         }
    //     }

    //     None
    // }
}
