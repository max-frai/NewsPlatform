use crate::sitemap::generate_categories_sitemap;
use crate::sitemap::generate_head_sitemap;
use crate::sitemap::generate_posts_sitemap;
use crate::sitemap::generate_tags_sitemap;
use routes::{
    authors::{authors, authors_fix},
    exact_category::{
        economy_category, economy_category_fix, entertainment_category, entertainment_category_fix,
        other_category, other_category_fix, science_category, science_category_fix,
        society_category, society_category_fix, sports_category, sports_category_fix,
        technology_category, technology_category_fix,
    },
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tag_cache::TagCache;
use tokio::sync::RwLock;

use actix_files::Files;
// use actix_web::{
//     middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers},
//     Error,
// };
use actix_web::{middleware, web, App, HttpServer};
use listenfd::ListenFd;

use crate::routes::categories::categories;
use crate::routes::categories::categories_fix;
use crate::routes::exact::exact;
use crate::routes::exact::exact_amp;
use crate::routes::exact_tag::exact_tag;
use crate::routes::exact_tag::exact_tag_fix;
use crate::routes::exact_tag::{fix_gpe_exact_tag, fix_person_exact_tag};
use crate::routes::index::index;
use crate::routes::js_bundle::js_bundle;
use crate::routes::robots::robots;
use crate::routes::rss::feed;
use crate::routes::search_console::search_console;
use crate::routes::tags::{tags_all, tags_all_fix, tags_scope, tags_scope_fix};
use crate::routes::test::test;
use crate::routes::tweets::tweets as tweets_route;
use crate::routes::tweets::tweets_fix as tweets_fix_route;
use strum::IntoEnumIterator;

use crate::node_helper::DomHelper;
use config;
use mongodb::Client;
use news_general::tag::*;
use news_general::{card_fetcher::CardFetcher, constants::*};
use state::State;
use tailwind::process_tailwind;
use tokio::time::sleep;

pub mod canonical_middleware;
pub mod helper;
pub mod indecies;
pub mod layout_context;
pub mod lowercase_middleware;
pub mod modules;
pub mod node_helper;
pub mod routes;
pub mod sitemap;
pub mod state;
pub mod tag_cache;
pub mod tailwind;
pub mod templates;

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
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let args = Args::from_args();
    let is_dev = args.dev;

    if is_dev {
        println!("------- DEVELOPMENT --------");
    } else {
        println!("------- PROD UCTION --------");
    }

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config.toml"))
        .expect("Failed to load Config.toml");

    let constants: Arc<AppConfig> =
        Arc::new(settings.try_into().expect("Wrong configuration format"));

    if is_dev {
        println!("Start css processing...");
        if let Err(e) = process_tailwind().await {
            println!("Failed to process tailwind modules");
            dbg!(e);
        }
        println!("Css is processed now");
    }

    println!("Load tera templates...");
    let tera = templates::init_tera();
    println!("Templates are loaded");

    println!("Connect mongodb: {}", &constants.mongodb_url);
    let client = Client::with_uri_str(&constants.mongodb_url)
        .await
        .expect("Failed to connect mongodb");

    let db = client.database(&constants.database_name);

    indecies::ensure_indecies(db.clone(), constants.clone()).await;

    println!("Select news and tags collections");
    let news_col = db.collection(&constants.cards_collection_name);
    let tags_col = db.collection(&constants.tags_collection_name);
    let twitter_col = db.collection(&constants.twitter_collection_name);

    println!("Create tags manager");
    let tags_manager = Arc::new(RwLock::new(TagsManager::new(tags_col, news_col.clone())));

    println!("Create cards fetcher");
    let fetcher = Arc::new(CardFetcher::new(
        news_col,
        tags_manager.clone(),
        constants.queries_cache_size,
        constants.exact_card_cache_size,
    ));

    println!("Create state");
    let state = web::Data::new(State {
        build_random_number: random_number::random!(),
        fetcher: fetcher.clone(),
        constants: constants.clone(),
        tera: tera.clone(),
        tags_manager: tags_manager.clone(),
        tags_cache: Arc::new(RwLock::new(HashMap::new())),
        js_bundle: Arc::new(RwLock::new(String::new())),
        sitemap: Arc::new(RwLock::new(String::new())),
        dom_helper: Arc::new(DomHelper::new()),

        twitter_col: twitter_col.clone(),
    });

    println!("Unique build number: {}", state.build_random_number);

    // Tags reloader
    let worker_tags_manager = tags_manager.clone();
    tokio::task::spawn(async move {
        loop {
            println!("Reload tags for tags manager in spawned task...");
            let (tags, tags_lookup) = {
                let tags_manager = worker_tags_manager.read().await;
                tags_manager.load().await
            };
            {
                let mut tags_manager = worker_tags_manager.write().await;
                tags_manager.set_data(tags, tags_lookup);
            }

            sleep(Duration::from_secs(60)).await;
        }
    });

    // Top persons & places reloader
    let worker_state = state.clone();
    tokio::task::spawn(async move {
        // First time wait for tags manager to load
        sleep(Duration::from_secs(20)).await;
        loop {
            println!("Load day exact top of tags in spawned task...");
            {
                let tags_manager = worker_state.tags_manager.read().await;
                for tag in TagKind::iter() {
                    println!("\tday top for kind: {}", tag);
                    let top = tags_manager
                        .get_popular_by_kind(Some(tag.clone()), chrono::Duration::days(1), 10)
                        .await
                        .expect("Failed to get day top");
                    {
                        let mut gpe_mut = worker_state.tags_cache.write().await;
                        gpe_mut.insert(TagCache::DayExactTop(tag), top);
                    }
                }
            }
            sleep(Duration::from_secs(60 * 30)).await;
        }
    });

    let worker_state = state.clone();
    tokio::task::spawn(async move {
        // First time wait for tags manager to load
        sleep(Duration::from_secs(20)).await;
        loop {
            println!("Load two week exact top of tags in spawned task...");
            {
                let tags_manager = worker_state.tags_manager.read().await;
                for tag in TagKind::iter() {
                    println!("\ttwo week top for kind: {}", tag);
                    let top = tags_manager
                        .get_popular_by_kind(Some(tag.clone()), chrono::Duration::days(14), 50)
                        .await
                        .expect("Failed to get week top");

                    {
                        let mut gpe_mut = worker_state.tags_cache.write().await;
                        gpe_mut.insert(TagCache::TwoWeekExactTop(tag), top);
                    }
                }

                let overall_top = tags_manager
                    .get_popular_by_kind(None, chrono::Duration::days(14), 50)
                    .await
                    .expect("Failed to get overall week top");
                {
                    let mut overall_mut = worker_state.tags_cache.write().await;
                    overall_mut.insert(TagCache::TwoWeekOverallTop, overall_top);
                }
            }
            sleep(Duration::from_secs(60 * 60 * 24)).await;
        }
    });

    let sitemap_state = state.clone();
    tokio::task::spawn(async move {
        loop {
            println!("Run sitemap generation in spawned task...");
            generate_posts_sitemap(sitemap_state.clone()).await;
            generate_categories_sitemap(sitemap_state.clone()).await;
            generate_tags_sitemap(sitemap_state.clone()).await;
            generate_head_sitemap(sitemap_state.clone()).await;
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    println!(
        "~~~~~~~~~~~ CREATE SERVER: {} ~~~~~~~~~~~~",
        constants.server_url
    );
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(lowercase_middleware::LowercaseRequest)
            .wrap(canonical_middleware::CanonicalRequest)
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(robots)
            .service(feed)
            .service(js_bundle)
            .service(search_console)
            .service(tags_all)
            .service(tags_all_fix)
            .service(tags_scope)
            .service(tags_scope_fix)
            .service(index)
            .service(test)
            .service(categories)
            .service(categories_fix)
            .service(fix_person_exact_tag)
            .service(fix_gpe_exact_tag)
            .service(exact_tag_fix)
            .service(exact_tag)
            .service(tweets_route)
            .service(tweets_fix_route)
            .service(exact)
            .service(exact_amp)
            .service(authors)
            .service(authors_fix)
            // Exact cateogories --------------------------------
            .service(society_category_fix)
            .service(entertainment_category_fix)
            .service(economy_category_fix)
            .service(technology_category_fix)
            .service(sports_category_fix)
            .service(science_category_fix)
            .service(other_category_fix)
            .service(society_category)
            .service(entertainment_category)
            .service(economy_category)
            .service(technology_category)
            .service(sports_category)
            .service(science_category)
            .service(other_category)
            // Exact cateogories --------------------------------
            .service(Files::new("/static", "./news_templates/"))
            .service(Files::new("/", "./news_templates/static/"))
    });

    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(&constants.server_url)?
    };

    server.run().await
}
