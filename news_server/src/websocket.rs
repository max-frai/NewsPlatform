use actix_web::web::Data;
use futures::StreamExt;
use news_general::cluster::Cluster;
use news_general::constants::AppConfig;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

use crate::state::State;

pub async fn connect_websocket(
    is_dev: bool,
    websocket_constants: Arc<AppConfig>,
    websocket_state: Data<State>,
) {
    println!("Start websocket client...");
    let domain = if is_dev {
        "ws://0.0.0.0".to_string()
    } else {
        format!("wss://{}", &websocket_constants.full_domain_raw)
    };

    let ws_addr = format!("{}:2087/ws", domain);
    println!("\tWS Address: {}", ws_addr);

    loop {
        if let Ok((mut socket, _)) = connect_async(&ws_addr).await {
            println!("Successfull connection to statistics websocket");
            while let Some(msg) = socket.next().await {
                if let Ok(msg) = msg {
                    if !msg.is_text() {
                        continue;
                    }

                    let data = msg.into_text().unwrap();

                    if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(&data) {
                        if msg_json
                            .get("kind")
                            .map(|kind| kind.as_str())
                            .flatten()
                            .unwrap_or("")
                            == "PopularClusterMessage"
                        {
                            println!("Got PopularClusterMessage from websocket, overwrite data");
                            let data = msg_json.get("data").unwrap().as_str().unwrap();
                            let cluster: Cluster = serde_json::from_str(&data).unwrap();
                            let mut write = websocket_state.popular_clusters.write().await;
                            *write = cluster;
                        }
                    }
                }
            }
        } else {
            println!("Failed to connect statistics websocket, sleep and try again...");
            sleep(Duration::from_secs(10)).await;
        }
    }
}
