use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::tag::Tag;

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
    pub tags: Vec<ObjectId>,
    pub filled_tags: Vec<Tag>,

    pub rewritten: bool,
    pub categorised: bool,
    pub tagged: bool,
}
