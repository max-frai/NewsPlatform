use crate::{card_fetcher::CardFetcher, constants::AppConfig};
use std::sync::Arc;
use tera::Tera;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
}
