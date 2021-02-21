use crate::{state::State, ws_server};
use actix_web::web;
use bson::{bson, doc, oid::ObjectId, Bson};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{cmp, collections::HashMap};
use ws_server::CovidTimeMessage;

pub type CovidPoints = Vec<(i64, i64)>;
pub type CovidMap = Vec<((f64, f64), i64)>;

#[derive(Serialize, Deserialize, Debug)]
pub struct CovidData {
    // Ukraine timeline points
    pub confirmed_points: CovidPoints,
    pub deaths_points: CovidPoints,
    pub recovered_points: CovidPoints,

    // All countries timeline points
    pub confirmed_points_all: CovidPoints,
    pub deaths_points_all: CovidPoints,
    pub recovered_points_all: CovidPoints,

    // Ukraine overall latest values
    pub confirmed_ua: i64,
    pub deaths_ua: i64,
    pub recovered_ua: i64,

    // All countries overall latest values
    pub confirmed_all: i64,
    pub deaths_all: i64,
    pub recovered_all: i64,
}

pub async fn get_csv(url: &str) -> anyhow::Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub async fn generate_timing(
    url: &str,
    skip_row_index: usize,
) -> anyhow::Result<(CovidPoints, CovidPoints)> {
    let csv_buffer = get_csv(url).await?;
    let mut rdr = csv::Reader::from_reader(csv_buffer.as_bytes());

    let base_date = Utc.ymd(2020, 1, 22).and_hms(0, 0, 0);

    let mut points: CovidPoints = vec![];
    let mut all_points: CovidPoints = vec![];

    let skip_first_columns = 4;

    let mut timestamp2value: HashMap<i64, i64> = HashMap::new();

    for (row_index, row) in rdr.records().enumerate() {
        let row = row.unwrap();
        let columns = row.iter();

        for (day_index, column) in columns.skip(skip_first_columns).enumerate() {
            let number = column.parse::<f64>().unwrap_or(0.).round() as i64;
            let timestamp = (base_date + chrono::Duration::days(1 * day_index as i64)).timestamp();

            *timestamp2value.entry(timestamp).or_insert(number) += number;
            if row_index == skip_row_index - 3 {
                points.push((number, timestamp));
            }
        }
    }

    for (timestamp, value) in timestamp2value {
        all_points.push((value, timestamp));
    }

    all_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    Ok((points, all_points))
}

pub async fn parse_overall(url: &str) -> anyhow::Result<((i64, i64, i64), (i64, i64, i64))> {
    let csv_buffer = get_csv(url).await?;
    let mut rdr = csv::Reader::from_reader(csv_buffer.as_bytes());

    let mut confirmed = 0;
    let mut deaths = 0;
    let mut recovered = 0;

    let mut confirmed_ua = 0;
    let mut deaths_ua = 0;
    let mut recovered_ua = 0;

    for row in rdr.records() {
        let row = row.unwrap();
        let mut columns = row.iter();

        let country = columns.by_ref().skip(1).next().unwrap();

        let mut col_iter = columns.skip(3);
        let iteration_confirmed = col_iter.next().unwrap().parse::<i64>().unwrap_or(0);
        let iteration_deaths = col_iter.next().unwrap().parse::<i64>().unwrap_or(0);
        let iteration_recovered = col_iter.next().unwrap().parse::<i64>().unwrap_or(0);

        if country.to_lowercase() == "ukraine" {
            confirmed_ua = iteration_confirmed;
            recovered_ua = iteration_recovered;
            deaths_ua = iteration_deaths;
        }

        confirmed += iteration_confirmed;
        deaths += iteration_deaths;
        recovered += iteration_recovered;
    }

    Ok((
        (confirmed_ua, deaths_ua, recovered_ua),
        (confirmed, deaths, recovered),
    ))
}

fn clear_match(data: &str) -> i64 {
    data.replace(",", "")
        .replace(" ", "")
        .parse::<i64>()
        .unwrap_or(-1)
}

async fn parse_ukraine_tkmedia(url: &str) -> anyhow::Result<(i64, i64, i64)> {
    let html_buffer = reqwest::get(url).await?.text().await?;

    let overall_re = regex::Regex::new("(?si)virus_counter_total\">(\\d+)<").unwrap();

    let mut overall_iter = overall_re.captures_iter(&html_buffer);
    let confirmed = clear_match(overall_iter.next().unwrap().get(1).unwrap().as_str());
    let deaths = clear_match(overall_iter.next().unwrap().get(1).unwrap().as_str());
    let recovered = clear_match(overall_iter.next().unwrap().get(1).unwrap().as_str());

    Ok((confirmed, recovered, deaths))
}

async fn parse_overall_better(url: &str) -> anyhow::Result<((i64, i64, i64), (i64, i64, i64))> {
    let html_buffer = reqwest::get(url).await?.text().await?;

    let overall_re = regex::Regex::new(r"(?si)maincounter-number.*?>(\d.*?)</span>").unwrap();
    let ukraine_re = regex::Regex::new(r"(?si)>Ukraine<.*?>([0-9,]+)</td>").unwrap();

    let mut total_cases_ua = -1;
    if let Some(item) = ukraine_re.captures(&html_buffer) {
        let str_data = item.iter().skip(1).next().unwrap().unwrap().as_str();
        total_cases_ua = clear_match(str_data);
    }

    let mut overall_iter = overall_re.captures_iter(&html_buffer);

    let overall_cases = clear_match(
        overall_iter
            .by_ref()
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
    );
    let overall_deaths = clear_match(
        overall_iter
            .by_ref()
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
    );
    let overall_recovered = clear_match(
        overall_iter
            .by_ref()
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
    );

    Ok((
        (total_cases_ua, -1, -1),
        (overall_cases, overall_deaths, overall_recovered),
    ))
}

pub async fn parse_covid(state: web::Data<State>) -> anyhow::Result<()> {
    let confirmed = "https://raw.githubusercontent.com/CSSEGISandData/COVID-19/master/csse_covid_19_data/csse_covid_19_time_series/time_series_covid19_confirmed_global.csv";

    let deaths = "https://raw.githubusercontent.com/CSSEGISandData/COVID-19/master/csse_covid_19_data/csse_covid_19_time_series/time_series_covid19_deaths_global.csv";

    let recovered = "https://raw.githubusercontent.com/bumbeishvili/covid19-daily-data/master/time_series_19-covid-Recovered.csv";

    let overall =
        "https://raw.githubusercontent.com/CSSEGISandData/COVID-19/web-data/data/cases.csv";

    let overall_better = "https://www.worldometers.info/coronavirus/";
    let tkmedia = "https://tk.media/coronavirus";

    let (confirmed_points, confirmed_points_all) = generate_timing(confirmed, 254).await?;
    let (deaths_points, deaths_points_all) = generate_timing(deaths, 254).await?;
    let (recovered_points, recovered_points_all) = generate_timing(recovered, 188).await?;
    // dbg!(&deaths_points);

    let ukraine_results_tkmedia = parse_ukraine_tkmedia(tkmedia).await?;
    let overall_results = parse_overall(overall).await?;
    let overall_results_odometer = parse_overall_better(overall_better).await?;

    // dbg!(&overall_results);

    let overall_results_final = (
        (
            // Confirmed
            cmp::max(
                ukraine_results_tkmedia.0,
                cmp::max((overall_results.0).0, (overall_results_odometer.0).0),
            ),
            // Deaths
            cmp::max(
                ukraine_results_tkmedia.1,
                cmp::max((overall_results.0).1, (overall_results_odometer.0).1),
            ),
            // Recovered
            cmp::max(
                ukraine_results_tkmedia.2,
                cmp::max((overall_results.0).2, (overall_results_odometer.0).2),
            ),
        ),
        (
            if (overall_results.1).0 > (overall_results_odometer.1).0 {
                (overall_results.1).0
            } else {
                (overall_results_odometer.1).0
            },
            if (overall_results.1).1 > (overall_results_odometer.1).1 {
                (overall_results.1).1
            } else {
                (overall_results_odometer.1).1
            },
            if (overall_results.1).2 > (overall_results_odometer.1).2 {
                (overall_results.1).2
            } else {
                (overall_results_odometer.1).2
            },
        ),
    );

    // dbg!(&overall_results);

    let skip_step = 3;
    state.ws_server_addr.do_send(CovidTimeMessage(CovidData {
        confirmed_points: confirmed_points
            .iter()
            .step_by(skip_step)
            .cloned()
            .collect(),
        deaths_points: deaths_points.iter().step_by(skip_step).cloned().collect(),
        recovered_points: recovered_points
            .iter()
            .step_by(skip_step)
            .cloned()
            .collect(),

        confirmed_points_all: confirmed_points_all
            .iter()
            .step_by(skip_step)
            .cloned()
            .collect(),
        deaths_points_all: deaths_points_all
            .iter()
            .step_by(skip_step)
            .cloned()
            .collect(),
        recovered_points_all: recovered_points_all
            .iter()
            .step_by(skip_step)
            .cloned()
            .collect(),

        confirmed_ua: (overall_results_final.0).0,
        deaths_ua: (overall_results_final.0).1,
        recovered_ua: (overall_results_final.0).2,

        confirmed_all: (overall_results_final.1).0,
        deaths_all: (overall_results_final.1).1,
        recovered_all: (overall_results_final.1).2,
    }));

    Ok(())
}
