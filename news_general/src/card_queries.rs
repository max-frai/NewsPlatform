use bson::Document;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Duration, Utc};

#[derive(Debug)]
pub struct CardQuery {
    pub lifetime: Duration,
    pub limit: Option<i64>,
    pub sort: Option<Document>,
    pub query: Document,
    pub projection: Option<Document>,
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
        projection: None,
    }
}

pub fn last_25() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(25),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
        projection: None,
    }
}

pub fn all_sitemap() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(1000000),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
        projection: None,
    }
}

pub fn last_n(num: i64) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: Some(num),
        sort: Some(doc! { "date" : -1 }),
        query: doc! {},
        projection: None,
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
        projection: None,
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
        projection: None,
    }
}

pub fn last_hours(hours: i64) -> CardQuery {
    let filter_utc = Utc::now() - chrono::Duration::hours(hours);

    CardQuery {
        lifetime: Duration::seconds(120),
        limit: None,
        sort: Some(doc! { "date" : -1 }),
        query: doc! {
            "date" : { "$gte" : filter_utc }
        },
        projection: None,
    }
}

pub fn last_between_dates(lower: DateTime<Utc>, upper: DateTime<Utc>) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(120),
        limit: None,
        sort: Some(doc! { "date" : -1 }),
        query: doc! {
            "$and" : vec![
                    doc! {
                        "date" : { "$gte" : lower },
                    },
                    doc! {
                        "date" : { "$lte" : upper }
                    }
                ]
        },
        projection: None
        // projection: Some(doc! {
        //     "title": 1
        // }),
    }
}
