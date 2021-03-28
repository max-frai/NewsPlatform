use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

    pub authors: Vec<String>,

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
