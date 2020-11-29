use crate::card_fetcher::CardFetcher;
use news_general::tag::TagsManager;
use news_general::{constants::*, tag::Tag};
use std::sync::Arc;
use tera::Tera;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
    pub tags_manager: Arc<TagsManager>,
    pub top_persons: Vec<Tag>,
    pub top_organizations: Vec<Tag>,
}
