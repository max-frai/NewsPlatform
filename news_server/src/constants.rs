use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub mongodb_url: String,
    pub database_name: String,
    pub cards_collection_name: String,
    pub queries_cache_size: usize,
    pub exact_card_cache_size: usize,
}