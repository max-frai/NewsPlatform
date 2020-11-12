use bson::doc;
use bson::Document;
use chrono::Duration;
use lazy_static::lazy_static;
use mongodb::options::FindOptions;

pub struct CardQuery {
    pub name: String,
    pub lifetime: Duration,
    pub options: FindOptions,
    pub query: Document,
}

lazy_static! {
    pub static ref INDEX_CARDQUERY: CardQuery = CardQuery {
        name: "INDEX_CARDQUERY".to_string(),
        lifetime: Duration::seconds(60),
        options: FindOptions::builder()
            .limit(10)
            .sort(Some(doc! {
                "date" : -1
            }))
            .build(),
        query: doc! {
            "country" : "ua"
        }
    };
}
