use chrono::{DateTime, Duration, Utc};
use mongodb::bson::Document;
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Debug)]
pub struct CardQuery {
    pub lifetime: Duration,
    pub limit: i64,
    pub sort: Document,
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
        limit: 15,
        sort: doc! { "date" : -1 },
        query: doc! {},
    }
}

pub fn last_25() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: 25,
        sort: doc! { "date" : -1 },
        query: doc! {},
    }
}

pub fn all_sitemap() -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(500),
        limit: 20000,
        sort: doc! { "date" : -1 },
        query: doc! {},
    }
}

pub fn last_n(num: i64) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(60),
        limit: num,
        sort: doc! { "date" : -1 },
        query: doc! {},
    }
}

pub fn last_25_by_category(category: &str) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(160),
        limit: 25,
        sort: doc! { "date" : -1 },
        query: doc! {
            "category" : category,
        },
    }
}

pub fn last_40_by_category(category: &str) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(160),
        limit: 40,
        sort: doc! { "date" : -1 },
        query: doc! {
            "category" : category,
        },
    }
}

pub fn last_40_by_tag(tag_id: ObjectId) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(160),
        limit: 40,
        sort: doc! { "date" : -1 },
        query: doc! {
            "tags" : tag_id
        },
    }
}

pub fn last_40_by_trend(trend: &str) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(160),
        limit: 40,
        sort: doc! { "date" : -1 },
        query: doc! {
            "trends" : trend
        },
    }
}

pub fn last_hours(hours: i64) -> CardQuery {
    let filter_utc = Utc::now() - chrono::Duration::hours(hours);

    CardQuery {
        lifetime: Duration::seconds(120),
        limit: 999999999,
        sort: doc! { "date" : -1 },
        query: doc! {
            "date" : { "$gte" : filter_utc }
        },
    }
}

pub fn last_between_dates(lower: DateTime<Utc>, upper: DateTime<Utc>) -> CardQuery {
    CardQuery {
        lifetime: Duration::seconds(120),
        limit: 9999999,
        sort: doc! { "date" : -1 },
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
    }
}
