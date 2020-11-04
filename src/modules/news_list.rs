use askama::Template;

#[derive(Template)]
#[template(path = "modules/news_list/tpl.html")]
pub struct NewsListTpl {
    pub title: Option<String>,
}
