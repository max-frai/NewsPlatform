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
use duct::cmd;
use mongodb::{options::ClientOptions, Client};
use state::State;

use crate::routes::error_500::render_500;
use crate::routes::exact::exact;
use crate::routes::index::index;

pub mod card;
pub mod card_fetcher;
pub mod modules;
pub mod routes;
pub mod state;

async fn process_tailwind() -> std::io::Result<String> {
    let mut css_container = String::new();
    let modules_dir = "templates/modules/";

    for entry in std::fs::read_dir(modules_dir)? {
        let entry = entry?;
        let path = format!("{}/tpl.scss", entry.path().as_os_str().to_str().unwrap());
        let css = std::fs::read_to_string(path)?;

        css_container = format!("{}\n{}", css_container, css);
    }

    let main_css = std::fs::read_to_string("templates/css/main.scss")?;
    let all_css = format!("{}\n{}", main_css, css_container);

    std::fs::write("templates/css/main.css", all_css)?;

    cmd!("postcss", "templates/css/main.css", "--replace").read()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "actix_web=info,actix_files=info");
    env_logger::init();

    println!("Start css processing...");
    if let Err(e) = process_tailwind().await {
        println!("Failed to process tailwind modules");
        dbg!(e);
    }
    println!("Css is processed now");

    println!("Connect mongodb");
    let client = Client::with_uri_str("mongodb://127.0.0.1:27019")
        .await
        .expect("Failed to connect mongodb");

    // let list_databases = client.list_database_names(None, None).await;
    // dbg!(&list_databases);

    let news = client.database("twn").collection("news");
    let fetcher = Arc::new(CardFetcher::new(news));
    let state = web::Data::new(State {
        fetcher: fetcher.clone(),
    });

    println!("Create server");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(exact)
            .service(Files::new("/static", "./templates/"))
    })
    .bind("127.0.0.1:4244")?
    .run()
    .await
}
