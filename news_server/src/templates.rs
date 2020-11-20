use comrak::{format_html, markdown_to_html, parse_document, Arena, ComrakOptions};
use std::{collections::HashMap, sync::Arc};
use tera::{from_value, to_value, Tera, Value};
use tera::{Error, Result};

use crate::card::Card;

fn category2name_internal(category: &str) -> String {
    match category {
        "society" => "Общество",
        _ => "",
    }
    .to_string()
}

pub fn make_card_url(args: &HashMap<String, Value>) -> Result<Value> {
    match args.get("card") {
        Some(val) => match from_value::<Card>(val.clone()) {
            Ok(card) => Ok(to_value(format!("/general/{}_{}", card._id, card.slug)).unwrap()),
            Err(_) => Err(Error::msg(
                "Function `make_url` received `card`, but with wrong type",
            )),
        },
        None => Err(Error::msg(
            "Function `make_url` was called without a `card` argument",
        )),
    }
}

pub fn category_name(args: &HashMap<String, Value>) -> Result<Value> {
    match args.get("category") {
        Some(val) => match from_value::<String>(val.clone()) {
            Ok(category) => Ok(to_value(category2name_internal(&category)).unwrap()),
            Err(_) => Err(Error::msg(
                "Function `category_name` received `category`, but with wrong type",
            )),
        },
        None => Err(Error::msg(
            "Function `category_name` was called without a `category` argument",
        )),
    }
}

pub fn markdown2html(args: &HashMap<String, Value>) -> Result<Value> {
    match args.get("markdown") {
        Some(val) => match from_value::<String>(val.clone()) {
            Ok(markdown) => {
                let arena = Arena::new();
                let root = parse_document(&arena, &markdown, &ComrakOptions::default());
                let mut html = vec![];
                format_html(root, &ComrakOptions::default(), &mut html).unwrap();
                let html_string = String::from_utf8(html).unwrap();

                Ok(to_value(html_string).unwrap())
            }
            Err(_) => Err(Error::msg(
                "Function `category_name` received `markdown`, but with wrong type",
            )),
        },
        None => Err(Error::msg(
            "Function `markdown2html` was called without a `markdown` argument",
        )),
    }
}

pub fn init_tera() -> Arc<Tera> {
    let mut tera = Tera::new("templates/**/*.tera").expect("Failed to load templates");
    tera.register_function("make_card_url", make_card_url);
    tera.register_function("category_name", category_name);
    tera.register_function("markdown2html", markdown2html);

    Arc::new(tera)
}
