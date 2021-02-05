use bson::{bson, doc, oid::ObjectId, Bson};
use maplit::*;
use mongodb::Collection;
use reqwest::header;
use serde_json::{Result, Value};

use super::graphs_manager::ChartsManager;

pub async fn parse_stocks(charts_manager: ChartsManager) -> anyhow::Result<()> {
    println!("------- PARSE STOCKS -------");

    let day_stock_chart = "https://www.investing.com/common/modules/js_instrument_chart/api/data.php?pair_id=%PAIR_ID%&pair_id_for_news=%PAIR_ID%&chart_type=area&pair_interval=86400&candle_count=120&events=yes&volume_series=yes";

    let pairid_to_name = hashmap! {
        8833 => "brent-oil",
        8830 => "gold",
        169 => "us-30",
        945629 => "btc-usd",
        2208 => "usd-uah",
        2186 => "usd-rub",
        166 => "us-spx-500",
        1 => "eur-usd"
    };

    let stock_pairs = vec![8833, 8830, 169, 945629, 2208, 2186, 166, 1];
    // dbg!(&stock_pairs);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::HeaderName::from_static("accept"),
        header::HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"),
    );
    headers.insert(
        header::HeaderName::from_static("accept-language"),
        header::HeaderValue::from_static("en-GB,en-US;q=0.9,en;q=0.8"),
    );
    headers.insert(
        header::HeaderName::from_static("connection"),
        header::HeaderValue::from_static("keep-alive"),
    );
    headers.insert(
        header::HeaderName::from_static("cookie"),
        header::HeaderValue::from_static("PHPSESSID=vj6acebtrj6dd5hutdj1rm5r3i; geoC=PL; prebid_session=0; adBlockerNewUserDomains=1584636949; StickySession=id.75592891101.165_www.investing.com; _ga=GA1.2.219646900.1584636951; _gid=GA1.2.2091249157.1584636952; __gads=ID=32c3ca031ed4076d:T=1584636951:S=ALNI_MZICwVsVUDSIefg_bgTTz44QNePvQ; G_ENABLED_IDPS=google; _fbp=fb.1.1584636952476.604599589; r_p_s_n=1; editionPostpone=1584638300057; gtmFired=OK; _hjid=c96e600b-f8f0-4abf-a05a-205977b0a462; _hjIncludedInSample=1; SideBlockUser=a%3A2%3A%7Bs%3A10%3A%22stack_size%22%3Ba%3A1%3A%7Bs%3A11%3A%22last_quotes%22%3Bi%3A8%3B%7Ds%3A6%3A%22stacks%22%3Ba%3A1%3A%7Bs%3A11%3A%22last_quotes%22%3Ba%3A2%3A%7Bi%3A0%3Ba%3A3%3A%7Bs%3A7%3A%22pair_ID%22%3Bs%3A4%3A%228830%22%3Bs%3A10%3A%22pair_title%22%3Bs%3A0%3A%22%22%3Bs%3A9%3A%22pair_link%22%3Bs%3A17%3A%22%2Fcommodities%2Fgold%22%3B%7Di%3A1%3Ba%3A3%3A%7Bs%3A7%3A%22pair_ID%22%3Bs%3A4%3A%228833%22%3Bs%3A10%3A%22pair_title%22%3Bs%3A0%3A%22%22%3Bs%3A9%3A%22pair_link%22%3Bs%3A22%3A%22%2Fcommodities%2Fbrent-oil%22%3B%7D%7D%7D%7D; prebid_page=0; _gat_allSitesTracker=1; _gat=1; nyxDorf=ODwyaGYzMHIybTooZzUwM2I3MXQyNGZsNzc%3D; GED_PLAYLIST_ACTIVITY=W3sidSI6IjVxajkiLCJ0c2wiOjE1ODQ2NTAxMTUsIm52IjoxLCJ1cHQiOjE1ODQ2NTAxMTAsImx0IjoxNTg0NjUwMTE0fSx7InUiOiJ2S2pCIiwidHNsIjoxNTg0NjUwMDk3LCJudiI6MSwidXB0IjoxNTg0NjUwMDk2LCJsdCI6MTU4NDY1MDA5Nn1d"),
    );
    headers.insert(
        header::HeaderName::from_static("host"),
        header::HeaderValue::from_static("www.investing.com"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-dest"),
        header::HeaderValue::from_static("empty"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-mode"),
        header::HeaderValue::from_static("cors"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-site"),
        header::HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        header::HeaderName::from_static("user-agent"),
        header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36"),
    );
    headers.insert(
        header::HeaderName::from_static("x-requested-with"),
        header::HeaderValue::from_static("XMLHttpRequest"),
    );

    for pair in &stock_pairs {
        // dbg!(pair);

        let url = day_stock_chart.replace("%PAIR_ID%", &pair.to_string());
        // dbg!(&url);

        let mut pair_headers = headers.clone();
        pair_headers.insert(
            header::HeaderName::from_static("referer"),
            header::HeaderValue::from_str(&format!(
                "https://www.investing.com/commodities/{}",
                pairid_to_name[pair]
            ))
            .unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(pair_headers)
            .build()?;

        let json: Value = client.get(&url).send().await?.json().await?;

        let candles = json["candles"].as_array().unwrap();
        let points: Vec<f64> = candles
            .iter()
            .rev()
            .take(30)
            .map(|item| item.as_array().unwrap()[1].as_f64().unwrap_or(0.0))
            .rev()
            .collect();

        {
            let mut write = charts_manager.write().await;
            write.update_charts(hashmap! { pair.clone() => points});
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    Ok(())
}
