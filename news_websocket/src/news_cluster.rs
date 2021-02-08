use crate::{
    state::State,
    ws_server::{MostRecentClusterMessage, PopularClusterMessage, SummaryClusterMessage},
};
use actix_web::web;
use bson::{oid::ObjectId, *};
use futures::StreamExt;
use news_general::{card::Card, card_queries::last_hours};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::{collections::HashMap, io::Write, process::Command};

#[derive(Serialize, Deserialize)]
struct ClusteringItem {
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

async fn clustering_logic(
    for_hours: i64,
    limit_threads: usize,
    limit_articles: usize,
    // clustering_distance_threshold: f64,
    state: web::Data<State>,
    allow_all_categories: bool,
) -> Vec<ClusteringResult> {
    let url_re = regex::Regex::new(r"((\w+://\S+)|(\w+[\.:]\w+\S+))[^\s,\.]").unwrap();
    let iframe_re = regex::Regex::new(r"<iframe.*?</iframe>").unwrap();
    let space_re = regex::Regex::new(r" +").unwrap();

    let mut source2name: HashMap<bson::oid::ObjectId, String> = HashMap::new();
    let mut sources_cursor = state
        .sources_col
        .find(None, None)
        // .find(Some(doc! {"country": "ua"}), None)
        .await
        .unwrap();

    while let Some(source) = sources_cursor.next().await {
        if let Ok(source_raw) = source {
            let oid = source_raw.get("_id").unwrap().as_object_id().unwrap();
            let name = source_raw.get("name").unwrap().as_str().unwrap_or_default();

            source2name.insert(oid.to_owned(), name.to_string());
        }
    }

    let all_docs = state
        .fetcher
        .fetch(last_hours(for_hours), true)
        .await
        .unwrap();

    let items = all_docs
        .par_iter()
        .map(|item| {
            let mut markdown = item
                .markdown
                .replace("\\-", "-")
                .replace("\\+", "+")
                .replace("\u{a0}", " ")
                .replace("#", "")
                .replace("*", "")
                .replace("----------", "")
                .replace("![", "")
                .replace("[", "")
                .replace("(", " ")
                .replace(")", " ")
                .replace("]", "")
                .replace("|", " ")
                .replace("\n\n", "\n")
                .replace("\n \n", "\n");

            markdown = url_re.replace_all(&markdown, "").to_string();
            markdown = iframe_re.replace_all(&markdown, "").to_string();
            markdown = markdown
                .replace("\n>", "\n")
                .replace("\n\n\n", "\n")
                .replace("\n\n", "\n");

            markdown = space_re.replace_all(&markdown, " ").to_string();

            // if let Some(info) = detect(&markdown) {
            //     if info.lang() == Lang::Ukr {
            //         // println!("Ukranian detected!");
            //         markdown = translate_uk2ru(markdown);
            //     }
            // }

            if !source2name.contains_key(&item.source_id) {
                return None;
            }

            Some(ClusteringItem {
                category: String::default(),
                description: String::default(),
                timestamp: item.date.timestamp(),
                site_name: source2name[&item.source_id].to_owned(),
                text: markdown.to_owned(),
                title: item.title.to_owned(),
                url: item.link.to_owned(),
                file_name: item._id.to_string(),
            })
        })
        .flat_map(|item| item)
        .collect::<Vec<ClusteringItem>>();

    // println!("\tGenerate json");
    let mut file = std::fs::File::create("news.json").unwrap();
    let json_str = serde_json::to_string(&items).unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
    file.sync_all().unwrap();

    let news_json_path = env::current_dir().unwrap().join("news.json");
    // dbg!(&news_json_path);

    let prev_cd = env::current_dir().unwrap();
    let cd = Path::new("news_nlp");
    env::set_current_dir(&cd).expect("Failed to change current dir to nlp folder");

    // println!(
    //     "Execute clustering binary with clustering threshold: {}",
    //     clustering_distance_threshold
    // );
    // ./build/tgnews top test/data/news.json --from_json
    let output = Command::new(format!("./nlp_{}", env::consts::OS))
        .arg("top")
        .arg(&news_json_path)
        // .arg("--ru_clustering_distance_threshold")
        // .arg(clustering_distance_threshold.to_string())
        // .arg("--from_json")
        .output()
        .unwrap();

    env::set_current_dir(&prev_cd).expect("Failed to change current dir to prev folder");

    // let stderr = std::str::from_utf8(&output.stderr).unwrap_or_default();
    // dbg!(&stderr);

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or_default();
    // dbg!(&stdout);
    let result = serde_json::from_str::<Vec<ClusteringResult>>(stdout).unwrap();

    // dbg!(&result);

    let mut clusters = vec![];
    for mut cluster in result {
        if cluster.category == "any"
            || cluster.category == "other"
            || cluster.category == "entertainment"
            || cluster.category == "technology"
            || cluster.category == "sports"
            || cluster.category == "science"
            || cluster.threads.is_empty()
        {
            continue;
        }

        if !allow_all_categories && cluster.category != "society" {
            continue;
        }

        cluster.threads = cluster.threads.into_iter().take(limit_threads).collect();

        for thread in cluster.threads.iter_mut() {
            thread.articles = thread
                .articles
                .iter()
                .take(limit_articles)
                .map(|i| i.clone())
                .collect();

            for article in &thread.articles {
                let oid = ObjectId::with_string(&article).unwrap();
                state.fetcher.fetch_exact_by_id(oid).await.unwrap();
            }
        }

        if cluster.threads.is_empty() {
            continue;
        }

        clusters.push(cluster);
    }

    clusters
}

#[derive(PartialEq, Eq)]
enum NewsTitleSorting {
    DoNotSort,
    Ascending,
    Descending,
}

async fn generate_news(
    for_hours: i64,
    limit_threads: usize,
    limit_articles: usize,
    // clusering_distance: f64,
    title_sorting: NewsTitleSorting,
    state: web::Data<State>,
    allow_all_categories: bool,
) -> Vec<ClusteringResult> {
    let mut clusters = clustering_logic(
        for_hours,
        limit_threads,
        limit_articles,
        // clusering_distance,
        state.clone(),
        allow_all_categories,
    )
    .await;

    // dbg!(&clusters);

    for summary in clusters.iter_mut() {
        summary
            .threads
            .sort_by(|a, b| b.articles.len().partial_cmp(&a.articles.len()).unwrap());

        summary.threads = summary
            .threads
            .iter()
            // .filter(|thread| thread.articles.len() > 2)
            .filter(|thread| thread.articles.len() >= 1)
            .take(15)
            .cloned()
            .collect();

        // dbg!(&summary);

        let mut items_cache = HashMap::new();
        for mut thread in summary.threads.iter_mut() {
            for article_id in &thread.articles {
                let article_id = ObjectId::with_string(article_id).unwrap();
                // dbg!(&article_id);
                let item = state
                    .fetcher
                    .fetch_exact_by_id(article_id.clone())
                    .await
                    .unwrap();
                // dbg!(&item);
                items_cache.insert(article_id, item);
            }

            if title_sorting != NewsTitleSorting::DoNotSort {
                thread.articles.sort_by(|a, b| {
                    let a_oid = ObjectId::with_string(a).unwrap();
                    let b_oid = ObjectId::with_string(b).unwrap();

                    let a_item = items_cache[&a_oid].to_owned();
                    let b_item = items_cache[&b_oid].to_owned();

                    let a_chars = a_item.title.chars().count();
                    let b_chars = b_item.title.chars().count();

                    (match title_sorting {
                        NewsTitleSorting::Ascending => a_chars.partial_cmp(&b_chars),
                        NewsTitleSorting::Descending => b_chars.partial_cmp(&a_chars),
                        _ => panic!("its impossible"),
                    })
                    .unwrap()
                });
            }

            let oid = ObjectId::with_string(thread.articles.first().unwrap()).unwrap();

            thread.main_item = items_cache.get_mut(&oid).unwrap().to_owned();
            thread.main_item.html = String::new();
        }

        summary.threads.sort_by(|a, b| {
            b.main_item
                .date
                .timestamp()
                .partial_cmp(&a.main_item.date.timestamp())
                .unwrap()
        });
    }

    return clusters;
}

pub async fn generate_json_for_clustering(state: web::Data<State>) -> anyhow::Result<()> {
    println!("--- GENERATE JSON FOR CLUSTERING ---");

    let popular_clusters = generate_news(
        6,
        // 100000,
        12,
        300,
        // 0.014,
        NewsTitleSorting::DoNotSort,
        state.clone(),
        true,
    )
    .await;
    state.ws_server_addr.do_send(PopularClusterMessage {
        clusters: popular_clusters,
    });

    let summary_24h_clusters = generate_news(
        24,
        // 100000,
        50,
        300,
        // 0.013,
        NewsTitleSorting::Descending,
        state.clone(),
        false,
    )
    .await;
    for cluster in summary_24h_clusters {
        if cluster.category == "society" {
            state
                .ws_server_addr
                .do_send(SummaryClusterMessage { cluster });
        }
    }

    let most_recent_clusters = generate_news(
        4,
        // 1000000,
        50,
        300,
        // 0.018,
        NewsTitleSorting::Ascending,
        state.clone(),
        false,
    )
    .await;
    for cluster in most_recent_clusters {
        if cluster.category == "society" {
            state
                .ws_server_addr
                .do_send(MostRecentClusterMessage { cluster });
        }
    }

    Ok(())
}
