use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub mongodb_url: String,
    pub database_name: String,
    pub cards_collection_name: String,
}
