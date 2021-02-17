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

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
// use std::io;

pub mod air;
pub mod covid;
pub mod fuel_uah;
pub mod graphs_manager;
pub mod news_cluster;
pub mod state;
pub mod stocks;
pub mod trends;
pub mod ws_client;
pub mod ws_server;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "running", about = "Analytics Platform")]
struct Args {
    /// Activate debug mode
    #[structopt(short = "d", long = "dev")]
    dev: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::from_args();
    let is_dev = args.dev;

    if is_dev {
        println!("------- DEVELOPMENT --------");
    }

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
        ws_server_addr: ws_server_addr.clone(),
        charts_manager: charts_manager.clone(),
        sources_col: sources_col.clone(),
    });

    let charts_manager2 = charts_manager.clone();
    let charts_manager3 = charts_manager.clone();
    let clustering1 = state.clone();
    let clustering2 = state.clone();
    let covid_state = state.clone();

    tokio::task::spawn(async move {
        loop {
            println!("------- PARSE STOCKS -------");
            let charts = charts_manager.clone();
            tokio::task::spawn(async move {
                stocks::parse_stocks(charts).await;
            })
            .await;
            sleep(Duration::from_secs(60 * 3)).await;
        }
    });

    tokio::task::spawn(async move {
        loop {
            println!("------- PARSE AIR -------");
            let charts = charts_manager2.clone();
            tokio::task::spawn(async move {
                air::parse_air_quality(charts).await;
            })
            .await;
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    tokio::task::spawn(async move {
        loop {
            println!("----- PARSE BLACK UAH, FUEL -----");
            let charts = charts_manager3.clone();
            tokio::task::spawn(async move {
                fuel_uah::parse_black_uah(charts).await;
            })
            .await;
            sleep(Duration::from_secs(60 * 4)).await;
        }
    });

    tokio::task::spawn(async move {
        loop {
            println!("--- GENERATE JSON FOR CLUSTERING ---");
            let clustering = clustering1.clone();
            tokio::task::spawn(async move {
                news_cluster::generate_json_for_clustering(clustering).await;
            })
            .await;
            sleep(Duration::from_secs(60 * 4)).await;
        }
    });

    tokio::task::spawn(async move {
        loop {
            println!("--- GENERATE TRENDS ---");
            let clustering = clustering2.clone();
            tokio::task::spawn(async move {
                trends::parse_trends(clustering).await;
            })
            .await;
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    tokio::task::spawn(async move {
        loop {
            println!("--- PARSE COVID ---");
            let covid = covid_state.clone();
            tokio::task::spawn(async move {
                covid::parse_covid(covid).await;
            })
            .await;
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

    println!("Configure cert for websocket server");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        if is_dev {
            server.bind(&constants.ws_server_url)?
        } else {
            server.bind_openssl(&constants.ws_server_url, builder)?
        }
    };

    server.run().await
}
