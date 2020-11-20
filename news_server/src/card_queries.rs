use bson::doc;
use bson::Document;
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
