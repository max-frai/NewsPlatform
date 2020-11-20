use browser_rs::Browser;
use chrono::prelude::*;
use html2md;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use rss_parser_rs::{ParseMode, RssItem, RssProcessor};
use serde::{Deserialize, Serialize};
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use whatlang::{detect, Lang, Script};

use bson::oid::ObjectId;
use byteorder::ByteOrder;
use byteorder::{BigEndian, ReadBytesExt};
use config::Config;
use duct::*;
use mongodb::{
    bson::{doc, document::Document, Bson},
    options::{FindOptions, InsertManyOptions},
    sync::Client,
};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RssItemFull {
    link: Option<Url>,
    title: Option<String>,
    slug: Option<String>,
    pub_date: Option<DateTime<Utc>>,
    description: Option<String>,

    // Additional
    source: Option<String>,
    country: Option<String>,
    source_name: Option<String>,
    project: Option<String>,
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
    // #[serde(serialize_with = "ts_seconds::serialize")]
    pub date: bson::DateTime,
    pub description: String,
    pub lang: String,
    pub link: String,
    pub country: String,
    pub category: String,
    pub marks: Vec<String>,
    pub tags: Vec<String>,

    pub rewritten: bool,
    pub categorised: bool,
    pub tagged: bool,
}

impl Default for RssItemFull {
    fn default() -> Self {
        Self {
            link: None,
            title: None,
            slug: None,
            pub_date: None,
            description: None,

            source: None,
            country: None,
            source_name: None,
            project: None,
        }
    }
}

impl RssItem for RssItemFull {
    fn set_link(&mut self, link: Url) {
        self.link = Some(link);
    }

    fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }

    fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_owned());
    }
    fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    fn set_slug(&mut self, slug: &str) {
        self.slug = Some(slug.to_owned());
    }
    fn slug(&self) -> Option<&String> {
        self.slug.as_ref()
    }

    fn set_description(&mut self, desc: &str) {
        self.description = Some(desc.to_owned());
    }
    fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    fn set_pub_date(&mut self, date: DateTime<Utc>) {
        self.pub_date = Some(date);
    }
    fn pub_date(&self) -> Option<&DateTime<Utc>> {
        self.pub_date.as_ref()
    }

    fn set_enclosure(&mut self, data: &str) {}
    fn set_category(&mut self, data: &str) {}
    fn enclosure(&self) -> Option<&String> {
        None
    }
    fn category(&self) -> Option<&String> {
        None
    }
}

pub fn extract_bson_string(data: Option<&Bson>) -> Option<String> {
    if let Some(&Bson::String(ref name)) = data {
        Some(name.clone())
    } else if let Some(&Bson::ObjectId(ref id)) = data {
        Some(id.to_string())
    } else {
        None
    }
}

fn object_id_from_timestamp(timestamp: u32) -> ObjectId {
    let mut buf: [u8; 12] = [0; 12];
    BigEndian::write_u32(&mut buf, timestamp);
    ObjectId::with_bytes(buf)
}

pub fn parse_news(client: Arc<Client>) {
    let browser = Arc::new(Browser::new(
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        Duration::from_secs(10),
    ));

    let db = client.database("news");
    let sources_collection = db.collection("sources");
    let news_collection = db.collection("news");

    // GET SLUG OF ALREADY LOADED NEWS FOR LAST N HOURS -----------------------------
    let filter_utc: chrono::DateTime<Utc> = Utc::now() - chrono::Duration::hours(100);

    let filter =
        doc! { "_id" : { "$gte" :  object_id_from_timestamp(filter_utc.timestamp() as u32) } };

    let last_news = news_collection
        .find(Some(filter), None)
        .expect("Failed to get news for filtering");

    let last_news_slug = last_news
        .map(|item| extract_bson_string(item.as_ref().unwrap().get("link")).unwrap_or_default())
        .collect::<Vec<String>>();

    println!("Last news slug length: {}", last_news_slug.len());

    println!("Get all sources...");
    // let options = FindOptions::builder().limit(1).build();
    let options = FindOptions::builder().build();
    let mut result_rss_items: Vec<RssItemFull> = Vec::with_capacity(1);
    // let data_result = sources_collection.find(None, Some(options));
    let data_result = sources_collection.find(None, None);

    if data_result.is_err() {
        println!("Failed to get sources: {:?}", data_result.as_ref().err());
    }

    if let Ok(cursor) = data_result {
        println!("Collect sources...");
        let mut all_sources: Vec<Document> =
            cursor.filter(|i| i.is_ok()).map(|i| i.unwrap()).collect();

        println!("Sources count: {}", all_sources.len());
        all_sources.shuffle(&mut rand::thread_rng());

        for source_chunk in all_sources.chunks(50) {
            result_rss_items.clear();

            result_rss_items = source_chunk
                .par_iter()
                .map(|source| {
                    let rss = RssProcessor::<RssItemFull>::new(ParseMode::Latest(100));

                    let rss_link = source.get("rss").unwrap().as_str().unwrap_or_default();
                    println!("Parse rss: {:?}", rss_link);

                    let url = url::Url::parse(rss_link).unwrap();
                    let xml = browser
                        .get(url.clone())
                        .map(|response| response.data)
                        .unwrap_or(String::default());

                    let mut result_items = rss.process(&xml).unwrap_or(Vec::default());

                    let parent_id = extract_bson_string(source.get("_id")).unwrap_or_default();
                    let parent_country =
                        extract_bson_string(source.get("country")).unwrap_or_default();
                    let parent_source_name =
                        extract_bson_string(source.get("name")).unwrap_or_default();
                    let project_name =
                        extract_bson_string(source.get("project")).unwrap_or_default();

                    for child in result_items.iter_mut() {
                        child.source = Some(parent_id.clone());
                        child.country = Some(parent_country.clone());
                        child.source_name = Some(parent_source_name.clone());
                        child.project = Some(project_name.clone());
                    }

                    result_items.clone()
                })
                .flat_map(Vec::into_par_iter)
                .filter(|item| return item.pub_date.is_some())
                .collect();

            let before = result_rss_items.len() as i32;
            println!("BEFORE REMOVING: {}", result_rss_items.len());

            if !last_news_slug.is_empty() {
                result_rss_items.retain(|ref item| {
                    if let Some(ref url) = item.link.as_ref() {
                        return !last_news_slug.contains(&url.as_str().to_string());
                    }
                    false
                });
            }

            let after = result_rss_items.len() as i32;
            println!(
                "AFTER REMOVING. NEWS COUNT TO PARSE: {}",
                result_rss_items.len()
            );
            println!("SKIPPED NEWS: {}", before - after);

            result_rss_items.shuffle(&mut rand::thread_rng());

            println!("Parsing & inserting...");
            let models: Vec<Document> = result_rss_items
                .par_iter()
                .map(|item| {
                    if item.link.is_none()
                        || item.title.is_none()
                        || item.slug.is_none()
                        || item.link.clone().unwrap().to_string() == ""
                        || item.title.clone().unwrap() == ""
                        || item.pub_date.is_none()
                        || !item.link.clone().unwrap().has_host()
                    {
                        // println!("Empty link or title or wrong date, skip it");
                        return None;
                    }

                    let link = item.link.clone().unwrap();
                    let title = item.title.clone().unwrap();
                    let slug = item.slug.clone().unwrap();
                    let source_id = ObjectId::with_string(&item.source.clone().unwrap()).unwrap();
                    let date = item.pub_date.unwrap();
                    let country = item.country.clone().unwrap();
                    let source_name = &item.source_name.clone().unwrap();
                    let project = &item.project.clone().unwrap();

                    if date > Utc::now() {
                        return None;
                    }

                    println!("Parse: {:?}", link);
                    let handle = cmd!(
                        format!("./parserbinary_{}", env::consts::OS),
                        link.to_string(),
                        "cfd724963e8336a0965bea0c0279cdab2ebb95de846e7019b62e1cd44292ebbcef5dba1efea6f351b8cbb9bb7bebc17ff3e13c35eba00c930cce494e25133724"
                    )
                    .stdout_capture()
                    .start()
                    .expect("Failed to execute parsebinary");

                    let parse_result = handle.wait();

                    if parse_result.is_err() {
                        // dbg!(parse_result);
                        return None;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(
                        &std::str::from_utf8(&parse_result.unwrap().stdout).unwrap(),
                    ) {
                        let mut html = json.get("content").unwrap().as_str().unwrap().to_string();
                        let description = json.get("description").unwrap().as_str().unwrap();
                        let og_image = json.get("og_image").unwrap().as_str().unwrap();

                        let mark_regex = vec![
                            regex::Regex::new(r"(<table>.*?</table>)").unwrap(), // HTML table
                            regex::Regex::new(r"(<img.*?>)").unwrap(),           // HTML image
                            regex::Regex::new(r"(<iframe>.*?</iframe>)").unwrap(), // HTML table
                            regex::Regex::new(r"(<iframe.*?/>").unwrap(), // HTML table
                        ];

                        let mut marks = vec![];
                        for re in mark_regex {
                            html = re
                                .replace_all(&html, |caps: &regex::Captures| {
                                    let mark_content = caps
                                        .get(1)
                                        .or(caps.get(2))
                                        .map(|i| i.as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    // dbg!(&mark_content);

                                    let mark_index = if let Some(pos) =
                                        marks.iter().position(|item| item == &mark_content)
                                    {
                                        pos
                                    } else {
                                        marks.push(mark_content.to_string());
                                        marks.len() - 1
                                    };
                                    format!(" {{{}}} ", mark_index)
                                })
                                .to_string();
                        }

                        // for (from, to) in FAST_FIX.iter() {
                        //     html = html.replace(from, to);
                        // }

                        let markdown = html2md::parse_html(&html);
                        let lang = detect(&markdown)
                            .map(|info| info.lang())
                            .unwrap_or(Lang::Rus)
                            .code()
                            .to_string();

                        // if lang != "rus" {
                        //     println!("Not russia text, skip this item");
                        //     return None;
                        // }

                        let lower_content =
                            format!("{} {}", title.to_lowercase(), html.to_lowercase());

                        // for stop in &constants.stop_words {
                        //     if lower_content.contains(&stop.to_lowercase()) {
                        //         println!("\tFound stop word: {}", stop);
                        //         return None;
                        //     }
                        // }

                        let item = Card {
                            _id: ObjectId::new(),
                            source_id,
                            link: link.to_string(),
                            og_image: og_image.to_owned(),
                            title,
                            slug,
                            category: String::from(""),
                            date: bson::DateTime(date),
                            country,
                            description: description.to_owned(),
                            lang,
                            html: html.to_string(),
                            markdown: markdown.to_string(),
                            markdown_original: markdown.to_string(),
                            marks,
                            tags: vec![],

                            rewritten: false,
                            categorised: false,
                            tagged: false
                        };

                        return Some(bson::to_bson(&item).unwrap().as_document().unwrap().clone());
                    } else {
                        println!("Wrong returned json from parsebinary");
                    }

                    None
                })
                .filter(|model| model.is_some())
                .map(|model| model.unwrap())
                .collect();

            println!("Models count: {}", models.len());

            news_collection
                .insert_many(models, InsertManyOptions::builder().ordered(false).build());
        }
    }
}