use actix::prelude::*;
use mongodb::Collection;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use news_general::{
    card_fetcher::CardFetcher, constants::AppConfig, tag::TagsManager, tweet::Tweet,
};

use crate::{graphs_manager::ChartsManager, ws_server::WsServer};

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tags_manager: Arc<RwLock<TagsManager>>,
    pub charts_manager: ChartsManager,
    pub ws_server_addr: Addr<WsServer>,
    pub sources_col: Collection,
    pub twitter_col: Collection,
    pub tweets_cache: Arc<RwLock<HashMap<&'static str, Vec<Tweet>>>>,
    pub is_dev: bool,
}
