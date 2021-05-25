use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub ws_server_url: String,
    pub mongodb_url: String,
    pub ner_url: String,
    pub database_name: String,
    pub queries_cache_size: usize,
    pub exact_card_cache_size: usize,
    pub platform_hash: String,
    pub full_domain: String,
    pub full_domain_raw: String,

    pub authors: Vec<String>,
    pub stop_tags: Vec<String>,

    pub country_1: String,
    pub country_2: String,
    pub country_3: String,

    pub corona_confirm_index: usize,
    pub corona_deaths_index: usize,
    pub corona_recovered_index: usize,

    pub parser_parse: bool,
    pub parser_rewrite: bool,
    pub parser_tag: bool,
    pub parser_categorise: bool,
    pub parser_translate: bool,

    pub cards_collection_name: String,
    pub tags_collection_name: String,
    pub sources_collection_name: String,
    pub twitter_collection_name: String,
}
