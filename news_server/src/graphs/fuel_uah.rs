use bson::{bson, doc, oid::ObjectId, Bson};
use maplit::*;
use serde_json::{Result, Value};
use std::{io::Read, sync::Arc};

use super::graphs_manager::ChartsManager;

async fn parse_uah() -> anyhow::Result<Vec<f64>> {
    println!("----- PARSE BLACK UAH -----");

    let url = "https://finance.i.ua/graph/avg_market/?currency=840";

    let json: Value = reqwest::get(url).await?.json().await?;
    let sell = &json.as_array().unwrap()[1].as_array().unwrap();
    // dbg!(&sell);
    Ok(sell
        .iter()
        .rev()
        .take(30)
        .filter_map(|item| item.as_array().unwrap()[1].as_f64())
        .filter(|item| item > &0.1)
        .rev()
        .collect::<Vec<f64>>())
}

async fn parse_fuel(fuel_type: &str) -> anyhow::Result<Vec<f64>> {
    println!("----- PARSE FUEL -----");

    let url = format!(
        "https://finance.i.ua/graph/avg_fuel/?id=&fuel_type={}",
        fuel_type
    );

    let json: Value = reqwest::get(&url).await?.json().await?;
    let sell = &json.as_array().unwrap();
    Ok(sell
        .iter()
        .rev()
        .take(30)
        .filter_map(|item| item.as_array().unwrap()[1].as_f64())
        .rev()
        .collect::<Vec<f64>>())
}

pub async fn parse_black_uah(charts_manager: ChartsManager) -> anyhow::Result<()> {
    let uah_points = parse_uah().await?;
    let fuel_points_a95 = parse_fuel("a_95").await?;
    let fuel_points_dp = parse_fuel("dp").await?;

    {
        let mut write = charts_manager.write().await;
        write.update_charts(hashmap! {
            40 => uah_points,
            41 => fuel_points_a95,
            47 => fuel_points_dp,
        });
    }

    Ok(())
}
