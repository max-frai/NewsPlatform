use bson::doc;
use chrono::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::Read, sync::Arc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MediaTypeOwn {
    Photo,
    Video,
    Gif,
    Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub id: String,
    pub url: String,
    pub media_url: String,
    pub kind: MediaTypeOwn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tweet {
    pub id: String,
    pub when: bson::DateTime,
    pub favorites: i32,
    pub retweets: i32,
    pub rt_and_fav: i32,
    pub text: String,

    pub user_id: String,
    pub user_name: String,
    pub user_screenname: String,
    pub user_image: String,
    pub entities: Vec<Entity>,
}
