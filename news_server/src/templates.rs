use std::{collections::HashMap, sync::Arc};
use tera::{from_value, to_value, Tera, Value};
use tera::{Error, Result};

use crate::card::Card;

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

pub fn init_tera() -> Arc<Tera> {
    let mut tera = Tera::new("templates/**/*.tera").expect("Failed to load templates");
    tera.register_function("make_card_url", make_card_url);

    Arc::new(tera)
}
