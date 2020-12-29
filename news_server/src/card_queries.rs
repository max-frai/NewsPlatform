use bson::Document;
use bson::{doc, oid::ObjectId};
use chrono::Duration;
use lazy_static::lazy_static;

#[derive(Debug)]
pub struct CardQuery {
    pub lifetime: Duration,
    pub limit: Option<i64>,
    pub sort: Option<Document>,
    pub query: Document,
}

impl std::fmt::Display for CardQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:?}{:?}{:?}",
            self.lifetime, self.limit, self.sort, self.query
        )
    }
}

pub fn last_15() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(15),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
    }
}

pub fn last_25() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(25),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
    }
}

pub fn all_sitemap() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(10),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
    }
}

pub fn last_n(num: i64) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(num),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
    }
}

pub fn last_25_by_category(category: &str) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(25),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {
            "category" : category,
        },
    }
}

pub fn last_15_by_tag(tag_id: ObjectId) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(15),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {
            "tags" : tag_id
        },
    }
}
