use actix::prelude::*;
use mongodb::Collection;
use std::sync::Arc;
use tokio::sync::RwLock;

use news_general::{card_fetcher::CardFetcher, constants::AppConfig, tag::TagsManager};

use crate::{graphs_manager::ChartsManager, ws_server::WsServer};

pub struct State {
    pub fetcher: Arc<CardFetcher>,
    pub constants: Arc<AppConfig>,
    pub tags_manager: Arc<RwLock<TagsManager>>,
    pub charts_manager: ChartsManager,
    pub ws_server_addr: Addr<WsServer>,
    pub sources_col: Collection,
}
