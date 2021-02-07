use serde::{Deserialize, Serialize};
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

#[derive(
    Display, EnumString, EnumIter, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone,
)]
#[strum(serialize_all = "snake_case")]
pub enum Category {
    Society,
    Entertainment,
    Economy,
    Technology,
    Sports,
    Science,
    Other,
    Unknown,
}

impl Category {
    pub fn to_description(&self) -> &'static str {
        match self {
            Category::Society => "Общество",
            Category::Entertainment => "Развлечения",
            Category::Economy => "Экономика",
            Category::Technology => "Технологии",
            Category::Sports => "Спорт",
            Category::Science => "Наука",
            Category::Other => "Общее",
            Category::Unknown => "",
        }
    }
}
