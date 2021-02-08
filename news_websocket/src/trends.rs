use crate::{state::State, ws_server::TodayTrendsMessage};
use actix_web::web;
use bson::doc;
use chrono::prelude::*;
use lazy_static::*;
use maplit::hashmap;
use news_general::card_queries::last_between_dates;
use rayon::prelude::*;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*, rsmorphy_dict_ru};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};
// use whatlang::{detect, Lang, Script};

lazy_static! {
    static ref IGNORE_START_WORDS: Vec<&'static str> = vec![
        "человек",
        "час",
        "июнь",
        "июль",
        "август",
        "сентябр",
        "ноябр",
        "октябр",
        "декабр",
        "вересн",
        "январ",
        "феврал",
        "апрел",
        "май",
        "сума",
        "украин",
        "росси",
        "америк",
        "европ",
        "прездиент",
        "понад",
        "затримал",
        "област",
        "треба",
        "район",
        "фото",
        "сутки",
        "тысяч",
        "миллион",
        "беларус",
        "белорус",
        "киян",
        "росий",
        "будут",
        "назвал",
        "киевл",
        "рассказ",
        "главн",
        "новы",
        "гривен",
        "киив",
        "росии",
        "киев",
        "промес",
        "после",
        "оценил",
        "заявил",
        "возможн",
        "сообщ",
        "будет"
    ];
    static ref IGNORE_WORDS: Vec<&'static str> = vec![
        "сша",
        "клуб",
        "рынок",
        "умерший",
        "заражение",
        "время",
        "день",
        "год",
        "як",
        "школа",
        "выбор",
        "дело",
        "видео",
        "время",
        "фото",
        "март",
        "також",
        "який",
        "пише",
        "така",
        "стали",
        "голова",
        "украина",
        "тысяча",
        "украинец",
        "случай",
        "человек",
        "киев",
        "область",
        "количество",
        "борьба",
        "дом",
        "мир",
        "страна",
        "работа",
        "подозрение",
        "ситуация",
        "женщина",
        "режим",
        "кабмина",
        "мужчина",
        "власть",
        "житель",
        "условие",
        "состояние",
        "глава",
        "город",
        "неделя",
        "заседание",
        "сеть",
        "правительство",
        "гражданин",
        "совет",
        "деньга",
        "цена",
        "центр",
        "число",
        "продукт",
        "ограничение",
        "жизнь",
        "эксперт",
        "народ",
        "решение",
        "место",
        "бизнес",
        "подробность",
        "средство",
        "водитель",
        "прогноз",
        "нарушение",
        "период",
        "погода",
        "буковин",
        "полицейский",
        "евро",
        "правило",
        "заявление",
        "полиция",
        "ребёнок",
        "львов",
        "помощь",
        "доллар",
        "депутат",
        "врач",
        "матч",
        "путин",
        "трамп",
        "днепр",
        "одесса",
        "крым",
        "динамо",
        "нардеп",
        "харьков",
        "дорога",
        "миллиард",
        "журналист",
        "зона",
        "группа",
        "месяц",
        "закон",
        "президент"
    ];
}

fn normal_form(word: &str, morph: &MorphAnalyzer) -> Option<String> {
    let parsed = morph.parse(word);
    if !parsed.is_empty() {
        let lex = parsed[0].lex.clone();
        if let Some(part) = lex.get_tag(morph).pos {
            return if part == Noun {
                Some(lex.get_normal_form(morph).to_string())
            } else {
                None
            };
        }
    }

    None
}

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

    let word_re = regex::Regex::new(r"(\w+)").unwrap();
    let statistics: Mutex<BTreeMap<String, (i32, String)>> = Mutex::new(BTreeMap::new());

    all_docs.par_iter().for_each(|article| {
        let title = article.title.replace("ё", "е");

        for capture in word_re.captures_iter(&title) {
            let original_word = capture.get(1).unwrap().as_str().to_lowercase();
            if original_word.chars().count() <= 3 {
                continue;
            }

            if let Some(normal_word) = normal_form(&original_word, &morph) {
                let mut break_now = false;
                if IGNORE_WORDS.contains(&normal_word.as_str())
                    || IGNORE_WORDS.contains(&original_word.as_str())
                {
                    continue;
                }

                for entry in IGNORE_START_WORDS.iter() {
                    if normal_word.starts_with(entry) || original_word.starts_with(entry) {
                        break_now = true;
                        break;
                    }
                }

                if break_now {
                    continue;
                }

                let mut writeable = statistics.lock().unwrap();
                let counter = writeable
                    .entry(normal_word.clone())
                    .or_insert((0, normal_word.clone()));

                (*counter).0 += 1;
            }
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
