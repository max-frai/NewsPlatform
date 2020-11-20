use bson::{oid::ObjectId, DateTime};
// use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub _id: ObjectId,
    pub og_image: String,
    pub title: String,
    pub html: String,
    pub slug: String,
    pub category: String,
    // #[serde(serialize_with = "ts_seconds::serialize")]
    pub date: DateTime,
    pub tags: Vec<String>,
}
