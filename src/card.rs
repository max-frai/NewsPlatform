use bson::{oid::ObjectId, DateTime};
use chrono::prelude::*;
use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

pub trait CardBounds: Send + Sync + Clone + Serialize + Deserialize<'static> + 'static {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card<C>
where
    C: Serialize + Clone,
{
    pub _id: ObjectId,
    pub og_image: String,
    pub title: String,
    pub html: String,
    pub slug: String,
    // #[serde(serialize_with = "ts_seconds::serialize")]
    pub date: DateTime,
    // genre: Vec<String>,
    // views: u32,
    #[serde(flatten)]
    additional_fields: C,
}

impl<C> Card<C>
where
    C: Serialize + Clone,
{
    pub fn time_str(&self) -> String {
        self.date.format("%R").to_string()
    }

    pub fn full_date_str(&self) -> String {
        self.date
            .format_localized("%e %B, %T", Locale::ru_RU)
            .to_string()
    }
}
