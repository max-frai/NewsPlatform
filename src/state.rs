use std::sync::Arc;

use crate::{card_fetcher::CardFetcher, constants::AppConfig};

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
}
