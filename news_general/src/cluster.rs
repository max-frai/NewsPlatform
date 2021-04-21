use crate::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClusteringItem {
    pub category: String,
    pub timestamp: i64,
    pub description: String,
    pub site_name: String,
    pub text: String,
    pub title: String,
    pub url: String,
    pub file_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusteringThread {
    pub articles: Vec<String>,
    pub category: String,
    pub title: String,
    #[serde(default)]
    pub main_item: Card,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusteringResult {
    pub category: String,
    pub threads: Vec<ClusteringThread>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    pub clusters: Vec<ClusteringResult>,
}

impl Default for ClusteringThread {
    fn default() -> Self {
        Self {
            articles: Vec::default(),
            category: String::default(),
            title: String::default(),
            main_item: Card::default(),
        }
    }
}
