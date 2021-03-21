use crate::{state::State, ws_server::TweetsMessage};
use actix_web::web;
use bson::doc;
use chrono::prelude::*;
use egg_mode::tweet::ExtendedTweetEntities;
use egg_mode::{self, entities::MediaType, tweet::Timeline};
use futures::StreamExt;
use lazy_static::*;
use mongodb::{
    options::{FindOptions, UpdateOptions},
    Collection,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::Read, sync::Arc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MediaTypeOwn {
    Photo,
    Video,
    Gif,
    Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub id: String,
    pub url: String,
    pub media_url: String,
    pub kind: MediaTypeOwn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tweet {
    pub id: String,
    pub when: bson::DateTime,
    pub favorites: i32,
    pub retweets: i32,
    pub rt_and_fav: i32,
    pub text: String,

    pub user_id: String,
    pub user_name: String,
    pub user_screenname: String,
    pub user_image: String,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tweets {
    pub kind: String,
    pub tweets: Vec<Tweet>,
}

lazy_static! {
    static ref CONSUMER_KEY: &'static str = "yh9PNC5eh2wmQBAmxkQ2YgXdV";
    static ref CONSUMER_SECRET: &'static str = "VFhissP1adZ8VP6KGyzQUpLQlvv89IoDGbMWGCqzA6xDT4YjxP";
    static ref ACCESS_KEY: &'static str = "1243855062247636992-8TsPQZVir0bZvFkFBCMKJaDIU87CNE";
    static ref ACCESS_SECRET: &'static str = "SHE1PafEQ1Xey0NcShUFTYawHwPuACJAJ7D7VQhjHTw5j";
}

async fn generate_tweets(state: web::Data<State>) -> anyhow::Result<()> {
    async fn gether_tweets(
        limit: i64,
        date: DateTime<Utc>,
        collection: Collection,
    ) -> anyhow::Result<Vec<Tweet>> {
        let filter = FindOptions::builder()
            .sort(doc! { "retweets" : -1 })
            .limit(limit)
            .build();

        let mut tweets_iter = collection
            .find(
                doc! {
                "when": { "$gte" :  date }
                },
                Some(filter),
            )
            .await?;

        let mut tweets = vec![];
        while let Some(tweet) = tweets_iter.next().await {
            tweets.push(bson::from_bson::<Tweet>(bson::Bson::Document(tweet.unwrap())).unwrap());
        }

        println!("Tweets gethered: {}", tweets.len());

        tweets.sort_by(|a, b| b.retweets.partial_cmp(&a.retweets).unwrap());

        Ok(tweets)
    }

    let tweets_limit = 30;

    let hour_date: DateTime<Utc> = Utc::now() - chrono::Duration::hours(1);
    let hour_tweets = gether_tweets(tweets_limit, hour_date, state.twitter_col.clone()).await?;

    let twelve_hours_date: DateTime<Utc> = Utc::now() - chrono::Duration::hours(12);
    let twelve_hours_tweets =
        gether_tweets(tweets_limit, twelve_hours_date, state.twitter_col.clone()).await?;

    let day_hours_date: DateTime<Utc> = Utc::now() - chrono::Duration::hours(24);
    let day_hours_tweets =
        gether_tweets(tweets_limit, day_hours_date, state.twitter_col.clone()).await?;

    let week_hours_date: DateTime<Utc> = Utc::now() - chrono::Duration::days(7);
    let week_hours_tweets =
        gether_tweets(tweets_limit, week_hours_date, state.twitter_col.clone()).await?;

    let month_hours_date: DateTime<Utc> = Utc::now() - chrono::Duration::days(30);
    let month_hours_tweets =
        gether_tweets(tweets_limit, month_hours_date, state.twitter_col.clone()).await?;

    {
        let mut cache = state.tweets_cache.write().await;
        // if let Ok(mut cache) = data {
        *cache.entry("1hr").or_insert(Vec::new()) = hour_tweets.to_owned();
        *cache.entry("12hr").or_insert(Vec::new()) = twelve_hours_tweets.to_owned();
        *cache.entry("24hr").or_insert(Vec::new()) = day_hours_tweets.to_owned();
        *cache.entry("week").or_insert(Vec::new()) = week_hours_tweets.to_owned();
        *cache.entry("month").or_insert(Vec::new()) = month_hours_tweets.to_owned();
        // }
    }

    state.ws_server_addr.do_send(TweetsMessage {
        tweets: vec![
            Tweets {
                kind: "1hr".to_string(),
                tweets: hour_tweets,
            },
            Tweets {
                kind: "12hr".to_string(),
                tweets: twelve_hours_tweets,
            },
            Tweets {
                kind: "24hr".to_string(),
                tweets: day_hours_tweets,
            },
            Tweets {
                kind: "week".to_string(),
                tweets: week_hours_tweets,
            },
            Tweets {
                kind: "month".to_string(),
                tweets: month_hours_tweets,
            },
        ],
    });

    Ok(())
}

pub async fn parse_twitter(state: web::Data<State>) -> anyhow::Result<()> {
    generate_tweets(state.clone()).await?;

    // if !state.is_dev {
    let consumer_token =
        egg_mode::KeyPair::new(CONSUMER_KEY.to_owned(), CONSUMER_SECRET.to_owned());
    let access_token = egg_mode::KeyPair::new(ACCESS_KEY.to_owned(), ACCESS_SECRET.to_owned());

    let token = egg_mode::Token::Access {
        consumer: consumer_token,
        access: access_token,
    };

    let mut timeline_tweets = vec![];

    let mut timeline = egg_mode::tweet::home_timeline(&token).with_page_size(200);

    println!("Parse tweets first time");
    let (timeline_returned_0, mut this_tweets) = parse_twitter_logic(timeline, true).await?;
    timeline = timeline_returned_0;
    timeline_tweets.append(&mut this_tweets);

    for _ in 0..3 {
        match parse_twitter_logic(timeline, false).await {
            Ok((timeline_returned, mut this_tweets)) => {
                timeline = timeline_returned;
                timeline_tweets.append(&mut this_tweets);
                println!("Current tweets length is: {}", timeline_tweets.len());
            }
            Err(reason) => {
                dbg!(reason);
                break;
            }
        }
    }

    println!("Insert tweets");
    for tweet in timeline_tweets {
        let update_options = UpdateOptions::builder().upsert(true).build();
        let tweet_bson = bson::to_bson(&tweet).unwrap();
        state
            .twitter_col
            .update_one(
                doc! {
                    "id" : tweet.id
                },
                doc! {
                    "$set" : tweet_bson
                },
                Some(update_options),
            )
            .await?;
    }
    // }

    generate_tweets(state.clone()).await?;

    Ok(())
}

async fn parse_twitter_logic(
    timeline: Timeline,
    start: bool,
) -> anyhow::Result<(Timeline, Vec<Tweet>)> {
    let result = if start {
        timeline.start().await?
    } else {
        timeline.older(None).await?
    };

    let url_re = regex::Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();

    let mut result_tweets = vec![];
    for mut tweet in result.1.iter() {
        if let Some(retweet) = &tweet.retweeted_status {
            tweet = &retweet;
        }

        let lang = tweet.lang.clone().unwrap_or("und".to_string());
        if lang != "ru" && lang != "uk" {
            continue;
        }

        let favorites = tweet.favorite_count;
        let retweets = tweet.retweet_count;
        let rt_and_fav = favorites + retweets;

        if rt_and_fav < 50 {
            continue;
        }

        let id = tweet.id as i64;
        let when = bson::DateTime(tweet.created_at);
        let text = url_re.replace_all(&tweet.text, "").to_string();

        let mut user_id = 0 as i64;
        let mut user_name = String::new();
        let mut user_screenname = String::new();
        let mut user_image = String::new();

        if let Some(user) = &tweet.user {
            user_id = user.id as i64;
            user_name = user.name.to_owned();
            user_screenname = user.screen_name.to_owned();
            user_image = user.profile_image_url_https.to_owned();
        }

        let mut all_entities = vec![];

        for url in &tweet.entities.urls {
            let full_url = url.expanded_url.to_owned().unwrap_or_default();
            all_entities.push(Entity {
                id: String::from(""),
                url: full_url,
                media_url: String::new(),
                kind: MediaTypeOwn::Url,
            });
        }

        if let Some(entities) = &tweet.extended_entities {
            for entity in &entities.media {
                let id = entity.id as i64;

                let mut url = entity.expanded_url.to_owned();
                let mut media_url = entity.media_url_https.to_owned();

                let kind = match entity.media_type {
                    MediaType::Photo => MediaTypeOwn::Photo,
                    MediaType::Gif => MediaTypeOwn::Gif,
                    MediaType::Video => MediaTypeOwn::Video,
                };

                if matches!(kind, MediaTypeOwn::Video) {
                    url = media_url.to_owned();
                    if let Some(info) = &entity.video_info {
                        media_url = info
                            .variants
                            .first()
                            .map(|i| i.url.to_owned())
                            .unwrap_or(String::new());
                    }
                }

                all_entities.push(Entity {
                    id: id.to_string(),
                    url,
                    media_url,
                    kind,
                });
            }
        }

        result_tweets.push(Tweet {
            id: id.to_string(),
            when: when.into(),
            favorites,
            retweets,
            rt_and_fav,
            text,

            user_id: user_id.to_string(),
            user_name,
            user_screenname,
            user_image,

            entities: all_entities,
        });
    }

    Ok((result.0, result_tweets))
}
