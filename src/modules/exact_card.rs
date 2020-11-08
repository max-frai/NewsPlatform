use askama::Template;

use crate::card::Card;

#[derive(Template)]
#[template(path = "modules/exact_card/tpl.html")]
pub struct ExactCardTpl {
    pub card: Card<()>,
}
