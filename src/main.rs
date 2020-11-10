use std::sync::Arc;

use actix_files::Files;
use actix_web::{
    dev::{self, Body, ResponseBody},
    http,
    middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers},
    Error,
};
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use card_fetcher::CardFetcher;
use listenfd::ListenFd;

use crate::routes::exact::exact;
use crate::routes::test::test;
use config;
use constants::AppConfig;
use mongodb::{options::ClientOptions, Client};
use state::State;
use tailwind::process_tailwind;
use tera::Tera;

// use crate::routes::error_500::render_500;
// use crate::routes::exact::exact;
use crate::routes::index::index;

pub mod card;
pub mod card_fetcher;
pub mod constants;
pub mod modules;
pub mod routes;
pub mod state;
pub mod tailwind;
pub mod templates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "actix_web=info,actix_files=info");
    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    println!("Start css processing...");
    if let Err(e) = process_tailwind().await {
        println!("Failed to process tailwind modules");
        dbg!(e);
    }
    println!("Css is processed now");

    println!("Load tera templates...");
    let tera = templates::init_tera();
    println!("Templates are loaded");

    println!("Connect mongodb");
    let client = Client::with_uri_str(&constants.mongodb_url)
        .await
        .expect("Failed to connect mongodb");

    let news = client
        .database(&constants.database_name)
        .collection(&constants.cards_collection_name);

    let fetcher = Arc::new(CardFetcher::new(news));
    let state = web::Data::new(State {
        fetcher: fetcher.clone(),
        constants: constants.clone(),
        tera: tera.clone(),
    });

    println!("Create server");
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            // .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(exact)
            .service(test)
            .service(Files::new("/static", "./templates/"))
    });

    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(&constants.server_url)?
    };

    server.run().await
}
