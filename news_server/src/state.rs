use crate::card_fetcher::CardFetcher;
use news_general::tag::TagsManager;
use news_general::{constants::*, tag::Tag};
use std::sync::Arc;
use tera::Tera;
use tokio::sync::RwLock;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
    pub tags_manager: Arc<RwLock<TagsManager>>,
    pub top_persons: Arc<RwLock<Vec<Tag>>>,
    pub top_gpe: Arc<RwLock<Vec<Tag>>>,
    pub js_bundle: Arc<RwLock<String>>,
}
