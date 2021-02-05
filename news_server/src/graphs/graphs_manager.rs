use crate::ws_server::{ChartsMessage, WsServer};
use actix::prelude::*;
use maplit::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type ChartsManager = Arc<RwLock<Charts>>;

pub struct Charts {
    stocks: HashMap<i32, Vec<f64>>,
    // weather: Weather,
    air: i64,
    co: i64,
    traffic: i64,
    radiation: (f64, f64),
    ws_server: Addr<WsServer>,
}

impl Charts {
    pub fn new(ws_server: Addr<WsServer>) -> Self {
        Self {
            stocks: HashMap::new(),
            // weather: Weather::default(),
            air: 0,
            co: 0,
            traffic: 0,
            radiation: (0.0, 0.0),
            ws_server,
        }
    }

    pub fn send_updates(&self) {
        self.ws_server.do_send(ChartsMessage {
            stocks: self.stocks.clone(),
            // weather: self.weather.clone(),
            air: self.air.clone(),
            co: self.co.clone(),
            traffic: self.traffic.clone(),
            radiation: self.radiation.clone(),
        });
    }

    pub fn update_air(&mut self, pm: i64) {
        self.air = pm;
        self.send_updates();
    }

    pub fn update_co(&mut self, co: i64) {
        self.co = co;
        self.send_updates();
    }

    pub fn update_traffic(&mut self, traffic: i64) {
        self.traffic = traffic;
        self.send_updates();
    }

    pub fn update_radiation(&mut self, radiation: (f64, f64)) {
        self.radiation = radiation;
        self.send_updates();
    }

    // pub fn update_weather(&mut self, weather: Weather) {
    //     self.weather = weather;
    //     self.send_updates();
    // }

    pub fn update_charts(&mut self, data: HashMap<i32, Vec<f64>>) {
        for (id, points) in data {
            *self.stocks.entry(id).or_insert(vec![]) = points;
        }
        self.send_updates();
    }
}
