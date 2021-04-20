use lazy_static::*;
use rsmorphy::MorphAnalyzer;
use rsmorphy::{opencorpora::kind::PartOfSpeach::Noun, prelude::*};

lazy_static! {
    static ref WORD_RE: regex::Regex = regex::Regex::new(r"(\w+)").unwrap();
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

fn opencorpora_tag_to_universal(mut pos: String) -> String {
    if pos == "ADJF" || pos == "ADJS" {
        pos = String::from("ADJ");
    }

    if pos == "ADVB" {
        pos = String::from("ADV");
    }

    if pos == "NUMR" {
        pos = String::from("NUM");
    }

    if pos == "PRTF" || pos == "RPTS" {
        pos = String::from("PART");
    }

    pos
}

pub fn normalize_words(
    title: &str,
    morph: &MorphAnalyzer,
    ignore_common: bool,
) -> Vec<(String, String)> {
    let mut words = vec![];

    for capture in WORD_RE.captures_iter(&title) {
        let original_word = capture.get(1).unwrap().as_str().to_lowercase();
        if original_word.chars().count() <= 3 {
            continue;
        }

        let lexems = morph.parse(&original_word);
        if lexems.is_empty() {
            continue;
        }

        let lex = lexems[0].lex.get_lemma(&morph);
        let mut pos = lex
            .get_tag(&morph)
            .string
            .split(",")
            .next()
            .unwrap_or("")
            .to_string();

        pos = opencorpora_tag_to_universal(pos);
        let normal_form = lex.get_normal_form(&morph).to_string();

        if ignore_common {
            let mut break_now = false;

            if IGNORE_WORDS.contains(&normal_form.as_str())
                || IGNORE_WORDS.contains(&original_word.as_str())
            {
                continue;
            }

            for entry in IGNORE_START_WORDS.iter() {
                if normal_form.starts_with(entry) || original_word.starts_with(entry) {
                    break_now = true;
                    break;
                }
            }

            if break_now {
                continue;
            }
        }

        words.push((normal_form, pos));
    }

    words
}
