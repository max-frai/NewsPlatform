use crate::{
    card_fetcher::CardFetcher, graphs::graphs_manager::ChartsManager, tag_cache::TagCache,
    ws_server::WsServer,
};
use actix::prelude::*;
use mongodb::Collection;
use news_general::tag::TagsManager;
use news_general::{constants::*, tag::Tag};
use std::{collections::HashMap, sync::Arc};
use tera::Tera;
use tokio::sync::RwLock;

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tera: Arc<Tera>,
    pub tags_manager: Arc<RwLock<TagsManager>>,

    pub tags_cache: Arc<RwLock<HashMap<TagCache, Vec<Tag>>>>,

    pub js_bundle: Arc<RwLock<String>>,
    pub sitemap: Arc<RwLock<String>>,

    pub charts_manager: ChartsManager,
    pub ws_server_addr: Addr<WsServer>,

    pub sources_col: Collection,
}
