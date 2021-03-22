use finalfusion::prelude::*;
use leptess::LepTess;
use mongodb::Client;
use rsmorphy::prelude::*;
use rsmorphy_dict_ru;
use std::io::BufReader;
use std::{cell::RefCell, fs::File};
use std::{rc::Rc, sync::Arc};
use tokio::time::{sleep, Duration};
use tokio::{
    sync::{Mutex, RwLock},
    task,
};

use news_general::constants::*;
use news_general::tag::*;

pub mod categorise;
pub mod parse;
pub mod rewrite;
pub mod tag;
pub mod translate;

#[tokio::main]
async fn main() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    println!("Connect to mongodb");
    let client = Arc::new(
        Client::with_uri_str(&constants.mongodb_url)
            .await
            .expect("Failed to connect mongodb"),
    );

    let db = client.database(&constants.database_name);
    let tags_col = db.collection(&constants.tags_collection_name);

    let tags_manager = Arc::new(Mutex::new(TagsManagerWriter::new(tags_col).await));

    let local = task::LocalSet::new();
    if constants.parser_parse {
        let parse_client = client.clone();
        let parse_constants = constants.clone();
        let failed_to_parse_links = Arc::new(RwLock::new(Vec::<String>::new()));

        println!("Load image and process ORC --------");
        let ocr_handle = Rc::new(RefCell::new(
            LepTess::new(Some("./tessdata"), "rus").unwrap(),
        ));

        println!("Read word embedings ---------------");
        let mut reader =
            BufReader::new(File::open("ruwikiruscorpora_upos_skipgram_300_2_2019.bin").unwrap());
        let embeddings = Rc::new(Embeddings::read_word2vec_binary(&mut reader).unwrap());

        println!("Read morphy dictionaries ----------");
        let morph = Rc::new(MorphAnalyzer::from_file(rsmorphy_dict_ru::DICT_PATH));

        println!("Run local tokio task");
        local.spawn_local(async move {
            println!("Inside run until, spawn local task");
            tokio::task::spawn_local(async move {
                loop {
                    println!("!!!!!!!!!!!!!!!!!!!!!!! Parse news.......");
                    let client = parse_client.clone();
                    let constants = parse_constants.clone();
                    let failed = failed_to_parse_links.clone();
                    let morph = morph.clone();
                    let ocr = ocr_handle.clone();
                    let embeddings = embeddings.clone();
                    tokio::task::spawn_local(async move {
                        crate::parse::parse_news(client, constants, ocr, morph, embeddings, failed)
                            .await;
                    })
                    .await;

                    sleep(Duration::from_secs(60)).await;
                }
            })
            .await;
        });
    }

    if constants.parser_translate {
        let translate_client = client.clone();
        let translate_constants = constants.clone();
        tokio::task::spawn(async move {
            loop {
                println!("!!!!!!!!!!!!!!!!!!! Translate news.......");
                let client = translate_client.clone();
                let constants = translate_constants.clone();
                tokio::task::spawn(async move {
                    crate::translate::translate_news(client, constants.clone()).await;
                })
                .await;

                sleep(Duration::from_secs(60)).await;
            }
        });
    }

    if constants.parser_categorise {
        let categorise_client = client.clone();
        let categorise_constants = constants.clone();
        tokio::task::spawn(async move {
            loop {
                println!("!!!!!!!!!!!!!!!!!!! Categorise news.......");
                let client = categorise_client.clone();
                let constants = categorise_constants.clone();
                tokio::task::spawn(async move {
                    crate::categorise::categorise_news(client, constants.clone()).await;
                })
                .await;

                sleep(Duration::from_secs(20)).await;
            }
        });
    }

    if constants.parser_tag {
        let tag_client = client.clone();
        let tag_constants = constants.clone();
        tokio::task::spawn(async move {
            loop {
                println!("!!!!!!!!!!!!!!!!!!! Tag news.......");
                let client = tag_client.clone();
                let constants = tag_constants.clone();
                let tags = tags_manager.clone();
                tokio::task::spawn(async move {
                    crate::tag::tag_news(client, constants, tags).await;
                })
                .await;
                sleep(Duration::from_secs(10)).await;
            }
        });
    }

    if constants.parser_rewrite {
        let rewrite_client = client.clone();
        let rewrite_constants = constants.clone();
        tokio::task::spawn(async move {
            loop {
                println!("Rewrite news.......");
                let client = rewrite_client.clone();
                let constants = rewrite_constants.clone();
                tokio::task::spawn(async move {
                    crate::rewrite::rewrite_news(client, constants.clone()).await;
                })
                .await;
                sleep(Duration::from_secs(60)).await;
            }
        });
    }

    // std::future::pending::<()>().await;
    println!("Await local set");
    local.await;
}
