use crate::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewsListTpl {
    pub title: Option<String>,
    pub cards: Vec<Card>,
}
