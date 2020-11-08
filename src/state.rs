use std::sync::Arc;

use crate::card_fetcher::CardFetcher;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
}
