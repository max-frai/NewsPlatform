use news_general::card::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewsListTpl {
    pub title: Option<String>,
    pub cards: Vec<Card>,
    pub is_amp: bool,
}
