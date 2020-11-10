use askama::Template;

use crate::card::Card;

#[derive(Template)]
#[template(path = "modules/compact_news_list/tpl.html")]
pub struct NewsListTpl {
    pub title: Option<String>,
    pub cards: Vec<Card<()>>,
}
