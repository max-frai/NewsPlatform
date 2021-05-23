use chrono::Utc;
use comrak::{format_html, parse_document, Arena, ComrakOptions};
use lazy_static::lazy_static;
use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Bson;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{category::Category, tag::Tag};

fn default_author() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=3) as i64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub _id: ObjectId,
    pub source_id: ObjectId,
    pub og_image: String,
    pub title: String,
    pub html: String,
    pub markdown: String,
    pub markdown_original: String,
    pub slug: String,
    pub date: bson::DateTime,
    pub description: String,
    pub lang: String,
    pub link: String,
    pub country: String,
    pub category: Category,
    pub marks: Vec<String>,
    pub tags: Vec<ObjectId>,
    pub filled_tags: Vec<Tag>,

    #[serde(default)]
    pub trends: Vec<String>,

    #[serde(default = "default_author")]
    pub author: i64,

    pub rewritten: bool,
    pub categorised: bool,
    pub tagged: bool,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            _id: ObjectId::new(),
            source_id: ObjectId::new(),
            og_image: String::new(),
            title: String::new(),
            html: String::new(),
            markdown: String::new(),
            markdown_original: String::new(),
            slug: String::new(),
            date: bson::DateTime(Utc::now()),
            description: String::new(),
            lang: String::new(),
            link: String::new(),
            country: String::new(),
            category: Category::Unknown,
            marks: vec![],
            tags: vec![],
            trends: vec![],
            filled_tags: vec![],
            rewritten: false,
            categorised: false,
            tagged: false,
            author: 0,
        }
    }
}

lazy_static! {
    static ref MARK_RE: Regex = Regex::new(r"\{ ?(\d+) ?\}").unwrap();
    static ref MULTI_WHITESPACE_RE: Regex = Regex::new(r" {2,}").unwrap();
}

impl Card {
    pub fn markdown2html(&mut self) {
        let arena = Arena::new();
        let root = parse_document(&arena, &self.markdown, &ComrakOptions::default());
        let mut html = vec![];
        format_html(root, &ComrakOptions::default(), &mut html).unwrap();
        self.html = String::from_utf8(html).unwrap();
    }

    pub fn fill_marks(&mut self) {
        self.html = MARK_RE
            .replace_all(&self.html, |caps: &regex::Captures| {
                if let Ok(index) = caps[1].parse::<usize>() {
                    if index > self.marks.len() {
                        println!("index more than caps");
                    }
                    self.marks.get(index).cloned().unwrap_or_default()
                } else {
                    String::default()
                }
            })
            .to_string();

        self.html = MULTI_WHITESPACE_RE.replace_all(&self.html, " ").to_string();
    }

    pub fn fill_description(&mut self) {
        let mut markdown = self.markdown.trim().replace("\n", " ");
        markdown = MARK_RE.replace_all(&markdown, " ").to_string();
        markdown = MULTI_WHITESPACE_RE.replace_all(&markdown, " ").to_string();

        self.description = markdown.trim().chars().take(100).collect::<String>();
    }
}
