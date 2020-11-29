use news_general::card::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CategoriesListTpl {
    pub categories: Vec<(String, String)>,
}
