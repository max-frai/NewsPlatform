use std::{sync::Arc, time::Duration};

// use crate::graphs_manager::Charts;
use actix::prelude::*;
use actix_web::{middleware, web, App, HttpServer};
use graphs_manager::Charts;
use listenfd::ListenFd;
use mongodb::Client;
use news_general::{card_fetcher::CardFetcher, constants::AppConfig, tag::TagsManager};
use state::State;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub mod air;
pub mod fuel_uah;
pub mod graphs_manager;
pub mod news_cluster;
pub mod state;
pub mod stocks;
pub mod trends;
pub mod ws_client;
pub mod ws_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    let client = Client::with_uri_str(&constants.mongodb_url)
        .await
        .expect("Failed to connect mongodb");
    let db = client.database(&constants.database_name);

    let sources_col = db.collection(&constants.sources_collection_name);
    let news_col = db.collection(&constants.cards_collection_name);
    let tags_col = db.collection(&constants.tags_collection_name);

    let tags_manager = Arc::new(RwLock::new(TagsManager::new(tags_col, news_col.clone())));

    let fetcher = Arc::new(CardFetcher::new(
        news_col,
        tags_manager.clone(),
        constants.queries_cache_size,
        constants.exact_card_cache_size,
    ));

    println!("Start websocket server: {}", constants.ws_server_url);
    let ws_server_addr = ws_server::WsServer::default().start();

    let charts_manager = Arc::new(RwLock::new(Charts::new(ws_server_addr.clone())));

    let state = web::Data::new(State {
        fetcher: fetcher.clone(),
        constants: constants.clone(),
        tags_manager: tags_manager.clone(),
        // tags_cache: Arc::new(RwLock::new(HashMap::new())),
        ws_server_addr: ws_server_addr.clone(),
        charts_manager: charts_manager.clone(),
        sources_col: sources_col.clone(),
    });

    let charts_manager_clone = charts_manager.clone();
    tokio::task::spawn(async move {
        loop {
            stocks::parse_stocks(charts_manager_clone.clone())
                .await
                .unwrap();
            sleep(Duration::from_secs(60 * 3)).await;
        }
    });

    let charts_manager_clone2 = charts_manager.clone();
    tokio::task::spawn(async move {
        loop {
            air::parse_air_quality(charts_manager_clone2.clone())
                .await
                .unwrap();
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    let charts_manager_clone3 = charts_manager.clone();
    tokio::task::spawn(async move {
        loop {
            fuel_uah::parse_black_uah(charts_manager_clone3.clone())
                .await
                .unwrap();
            sleep(Duration::from_secs(60 * 4)).await;
        }
    });

    let clustering_state = state.clone();
    tokio::task::spawn(async move {
        loop {
            news_cluster::generate_json_for_clustering(clustering_state.clone())
                .await
                .unwrap();
            sleep(Duration::from_secs(60 * 4)).await;
        }
    });

    let clustering_state2 = state.clone();
    tokio::task::spawn(async move {
        loop {
            trends::parse_trends(clustering_state2.clone())
                .await
                .unwrap();
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws").route(web::get().to(ws_server::ws_index)))
    });

    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(&constants.ws_server_url)?
    };

    server.run().await
}
