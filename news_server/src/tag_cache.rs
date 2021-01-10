use news_general::tag::TagKind;

#[derive(Hash, Eq, PartialEq)]
pub enum TagCache {
    // Kinds from index page with limited size and less time
    DayExactTop(TagKind),

    // Exact kind from /tags/ page
    TwoWeekExactTop(TagKind),

    // Default kind for /tags/ and unknown tags
    TwoWeekOverallTop,
}
