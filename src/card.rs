use bson::{oid::ObjectId, DateTime};
use chrono::prelude::*;
// use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub _id: ObjectId,
    pub og_image: String,
    pub title: String,
    pub html: String,
    pub slug: String,
    // #[serde(serialize_with = "ts_seconds::serialize")]
    pub date: DateTime,
    // genre: Vec<String>,
    // views: u32,
}

// let extended = Extended {
//     card: self,
//     time_str: self.date.format("%R").to_string(),
//     full_date_str: self
//         .date
//         .format_localized("%e %B %H:%M", Locale::ru_RU)
//         .to_string(),
//     link: format!("/general/{}_{}", self._id, self.slug),
// };

// serializer.
