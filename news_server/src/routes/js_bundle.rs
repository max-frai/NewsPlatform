use crate::state::State;
use actix_web::{get, web, HttpResponse, Responder};

use tokio::fs;

#[get("/bundle.js")]
async fn js_bundle(state: web::Data<State>) -> actix_web::Result<impl Responder> {
    {
        let js_bundle = state.js_bundle.read().await;
        if !js_bundle.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type("text/javascript")
                .body(&*js_bundle));
        }
    }

    println!("Make js bundle cache");
    let mut dir = fs::read_dir("./news_templates/js/").await?;

    let mut paths = vec![];
    while let Some(child) = dir.next_entry().await? {
        let path = child.path();
        paths.push(path.to_str().unwrap().to_string());
    }

    alphanumeric_sort::sort_str_slice(&mut paths);

    let mut js = String::new();
    for path in paths {
        let js_content = fs::read_to_string(path).await?;
        js = format!("{}\n{}", js, js_content);
    }

    // Fix websocker port
    let port = state
        .constants
        .ws_server_url
        .split(":")
        .collect::<Vec<&str>>()[1];
    js = js.replace("WEBSOCKET_PORT", port);

    let mut js_bundle_mut = state.js_bundle.write().await;
    *js_bundle_mut = js.to_owned();

    return Ok(HttpResponse::Ok().content_type("text/javascript").body(&js));
}
