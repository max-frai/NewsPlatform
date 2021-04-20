use crate::{state::State, ws_server::TodayTrendsMessage};
use actix_web::web;
use bson::doc;
use chrono::prelude::*;
use lazy_static::*;
use maplit::hashmap;
use news_general::{card_queries::last_between_dates, normalize_words::normalize_words};
use rayon::prelude::*;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};
// use whatlang::{detect, Lang, Script};

async fn generate_trends(
    state: web::Data<State>,
    morph: &MorphAnalyzer,
    lower_utc: DateTime<Utc>,
    upper_utc: DateTime<Utc>,
) -> anyhow::Result<HashMap<String, i32>> {
    let all_docs = state
        .fetcher
        .fetch(last_between_dates(lower_utc, upper_utc), true)
        .await
        .unwrap();

    let statistics: Mutex<BTreeMap<String, (i32, String)>> = Mutex::new(BTreeMap::new());

    all_docs.par_iter().for_each(|article| {
        let title = article.title.replace("ё", "е");

        let words: Vec<String> = normalize_words(&title, &morph, true)
            .iter()
            .map(|item| item.0.to_owned())
            .collect();
        let mut writeable = statistics.lock().unwrap();
        for normal_word in words {
            let counter = writeable
                .entry(normal_word.clone())
                .or_insert((0, normal_word.clone()));

            (*counter).0 += 1;
        }
    });

    let writeable = statistics.lock().unwrap();
    let mut sorted = writeable
        .clone()
        .into_iter()
        .collect::<Vec<(String, (i32, String))>>();

    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let trends = sorted
        .iter()
        .take(40)
        .cloned()
        .map(|i| ((i.1).1, (i.1).0))
        .collect::<HashMap<String, i32>>();

    Ok(trends)
}

pub async fn parse_trends(state: web::Data<State>) -> anyhow::Result<()> {
    let morph = MorphAnalyzer::from_file("news_rsmorphy/");

    let upper_utc_today: DateTime<Utc> = Utc::now();
    let lower_utc_today: DateTime<Utc> = Utc::now() - chrono::Duration::hours(24);

    let upper_utc_yesterday: DateTime<Utc> = Utc::now() - chrono::Duration::hours(24);
    let lower_utc_yesterday: DateTime<Utc> = Utc::now() - chrono::Duration::hours(48);

    let trends_today =
        generate_trends(state.clone(), &morph, lower_utc_today, upper_utc_today).await?;
    let trends_yesterday = generate_trends(
        state.clone(),
        &morph,
        lower_utc_yesterday,
        upper_utc_yesterday,
    )
    .await?;

    let mut final_trends = hashmap! {};
    for trend in &trends_today {
        let mut koef = 1;
        if let Some(prev_trend) = trends_yesterday.get(trend.0) {
            let diff = trend.1 - prev_trend;
            if diff >= 0 {
                koef = diff;
            }
        }

        final_trends.insert(trend.0.to_string(), trend.1 + koef * 3);
    }

    let mut final_trends_sorted = final_trends
        .iter()
        .map(|i| (i.0.to_string(), i.1.to_owned()))
        .collect::<Vec<(String, i32)>>();

    final_trends_sorted.sort_by(|a, b| b.1.cmp(&a.1));

    state.ws_server_addr.do_send(TodayTrendsMessage {
        trends: final_trends_sorted.iter().take(20).cloned().collect(),
    });

    Ok(())
}
