use crate::{node_helper::DomHelper, tag_cache::TagCache};
use mongodb::Collection;
// use actix::prelude::*;
// use mongodb::Collection;
use news_general::{card_fetcher::CardFetcher, cluster::Cluster, tag::TagsManager};
use news_general::{constants::*, tag::Tag};
use std::{collections::HashMap, sync::Arc};
use tera::Tera;
use tokio::sync::RwLock;

pub struct State {
    pub build_random_number: u64,
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
    pub tags_manager: Arc<RwLock<TagsManager>>,

    pub tags_cache: Arc<RwLock<HashMap<TagCache, Vec<Tag>>>>,

    pub js_bundle: Arc<RwLock<String>>,
    pub sitemap: Arc<RwLock<String>>,
    pub popular_clusters: Arc<RwLock<Cluster>>,

    pub dom_helper: Arc<DomHelper>,

    pub twitter_col: Collection,
}
