use crate::{graphs::news_cluster::ClusteringResult, state::State, ws_client::WebSocketClient};
use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Serialize;
use std::collections::HashMap;

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    state: web::Data<State>,
) -> Result<HttpResponse, Error> {
    let res = ws::start(
        WebSocketClient::new(state.ws_server_addr.clone()),
        &r,
        stream,
    );
    res
}

#[derive(Message, Serialize, Clone, Debug)]
#[rtype(result = "()")]
pub struct JsonMessage {
    pub kind: String,
    pub data: String,
}

#[derive(Message, Serialize, Clone, Debug)]
#[rtype(result = "()")]
pub struct ChartsMessage {
    pub stocks: HashMap<i32, Vec<f64>>,
    // pub weather: Weather,
    pub air: i64,
    pub co: i64,
    pub traffic: i64,
    pub radiation: (f64, f64),
}

#[derive(Message, Serialize, Debug)]
#[rtype(result = "()")]
pub struct TodayTrendsMessage {
    pub trends: Vec<(String, i32)>,
}

#[derive(Message, Serialize, Debug)]
#[rtype(result = "()")]
pub struct PopularClusterMessage {
    pub clusters: Vec<ClusteringResult>,
}

#[derive(Message, Serialize, Debug)]
#[rtype(result = "()")]
pub struct SummaryClusterMessage {
    pub cluster: ClusteringResult,
}

#[derive(Message, Serialize, Debug)]
#[rtype(result = "()")]
pub struct MostRecentClusterMessage {
    pub cluster: ClusteringResult,
}

// #[derive(Message, Serialize, Debug)]
// #[rtype(result = "()")]
// pub struct TweetsMessage {
//     pub tweets: Vec<Tweets>,
// }

// #[derive(Message, Serialize, Debug)]
// #[rtype(result = "()")]
// pub struct CovidTimeMessage(pub CovidData);

// #[derive(Message, Serialize, Debug)]
// #[rtype(result = "()")]
// pub struct CovidMapMessage(pub CovidMapData);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<JsonMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

impl Default for JsonMessage {
    fn default() -> Self {
        Self {
            kind: String::new(),
            data: String::new(),
        }
    }
}
impl Default for ChartsMessage {
    fn default() -> Self {
        Self {
            stocks: HashMap::default(),
            // weather: Weather::default(),
            air: 0,
            co: 0,
            traffic: 0,
            radiation: (0.0, 0.0),
        }
    }
}

pub struct WsServer {
    sessions: HashMap<usize, Recipient<JsonMessage>>,
    last_messages: HashMap<String, JsonMessage>,
}

impl WsServer {
    fn send_all(&self, message: JsonMessage) {
        for (_, addr) in &self.sessions {
            self.send_one(message.clone(), addr);
        }
    }

    fn send_one(&self, message: JsonMessage, addr: &Recipient<JsonMessage>) {
        // dbg!(message);
        addr.do_send(message).unwrap();
    }

    fn add_client(&mut self, addr: Recipient<JsonMessage>) -> usize {
        let id = self.sessions.len() + 1;
        self.sessions.insert(id, addr);

        id
    }

    fn cache_message(&mut self, message: JsonMessage) {
        *self
            .last_messages
            .entry(message.kind.to_string())
            .or_insert(message) = message.clone();
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self {
            last_messages: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

fn handle_message<T: Serialize>(server: &mut WsServer, message: T) {
    let kind = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap()
        .to_string();
    let json_message = JsonMessage {
        kind,
        data: serde_json::to_string(&message).unwrap(),
    };
    server.cache_message(json_message.clone());
    server.send_all(json_message);
}

impl Handler<ChartsMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: ChartsMessage, _: &mut Context<Self>) {
        handle_message(self, msg);
    }
}

// impl Handler<TodayTrendsMessage> for WsServer {
//     type Result = ();

//     fn handle(&mut self, msg: TodayTrendsMessage, _: &mut Context<Self>) {
//         handle_message(self, msg);
//     }
// }

impl Handler<PopularClusterMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: PopularClusterMessage, _: &mut Context<Self>) {
        handle_message(self, msg);
    }
}

// impl Handler<TweetsMessage> for WsServer {
//     type Result = ();

//     fn handle(&mut self, msg: TweetsMessage, _: &mut Context<Self>) {
//         handle_message(self, msg);
//     }
// }

impl Handler<SummaryClusterMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SummaryClusterMessage, _: &mut Context<Self>) {
        handle_message(self, msg);
    }
}

// impl Handler<CovidTimeMessage> for WsServer {
//     type Result = ();

//     fn handle(&mut self, msg: CovidTimeMessage, _: &mut Context<Self>) {
//         handle_message(self, msg);
//     }
// }

// impl Handler<CovidMapMessage> for WsServer {
//     type Result = ();

//     fn handle(&mut self, msg: CovidMapMessage, _: &mut Context<Self>) {
//         handle_message(self, msg);
//     }
// }

impl Handler<MostRecentClusterMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: MostRecentClusterMessage, _: &mut Context<Self>) {
        handle_message(self, msg);
    }
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // println!("Someone connected");
        let id = self.add_client(msg.addr.clone());
        for message in &self.last_messages {
            self.send_one(message.1.clone(), &msg.addr);
        }

        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // println!("Someone disconnected");
        self.sessions.remove(&msg.id);
    }
}
