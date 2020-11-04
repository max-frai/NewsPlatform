use actix_files::Files;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use duct::cmd;
use mongodb::{options::ClientOptions, Client};

pub mod card;
pub mod modules;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    center_content: String,
}

#[get("/")]
async fn index() -> impl Responder {
    let news_list_tpl = modules::news_list::NewsListTpl {
        title: Some(String::from("Проверка заголовка")),
    }
    .render()
    .unwrap();

    HttpResponse::Ok().content_type("text/html").body(
        IndexTemplate {
            center_content: news_list_tpl,
        }
        .render()
        .unwrap(),
    )
}

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

    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect mongodb");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(index)
            .service(Files::new("/static", "./templates/"))
    })
    .bind("127.0.0.1:4244")?
    .run()
    .await
}
