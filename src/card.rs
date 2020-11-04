use bson::{oid::ObjectId, DateTime};
use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

pub trait CardBounds: Send + Sync + Clone + Serialize + Deserialize<'static> + 'static {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card<C>
where
    C: Serialize + Clone,
{
    _id: ObjectId,
    image: String,
    title: String,
    description: String,
    slug: String,
    #[serde(serialize_with = "ts_seconds::serialize")]
    when: DateTime,
    genre: Vec<String>,
    views: u32,

    #[serde(flatten)]
    additional_fields: C,
}
