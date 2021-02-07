use serde_json::Value;

use super::graphs_manager::ChartsManager;

pub async fn parse_air_quality(charts_manager: ChartsManager) -> anyhow::Result<()> {
    let url = "https://api.waqi.info/mapq/bounds/?bounds=50.28495199762422,30.120391845703125,50.65207267042477,30.76858520507813";

    let json: Value = reqwest::get(url).await?.json().await?;
    let items = &json.as_array().unwrap();

    let mut max_aqi = 0;
    for item in items.iter() {
        let aqi = item
            .get("aqi")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<i64>()
            .unwrap_or(0);

        if aqi > max_aqi {
            max_aqi = aqi;
        }
    }

    {
        let mut write = charts_manager.write().await;
        write.update_air(max_aqi);
    }

    Ok(())
}
