use crate::card_fetcher::CardFetcher;
use news_general::constants::*;
use news_general::tag::TagsManager;
use std::sync::Arc;
use tera::Tera;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
    pub tags_manager: Arc<TagsManager>,
}
