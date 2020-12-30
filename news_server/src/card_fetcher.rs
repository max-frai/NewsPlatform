use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::card_queries::CardQuery;
use anyhow::Context;
use anyhow::Result;
use bson::{doc, oid::ObjectId};
use chrono::prelude::*;
use futures::stream::StreamExt;
use lru_cache::LruCache;
use mongodb::options::FindOptions;
use mongodb::Collection;
use news_general::{card::*, tag::TagsManager};
use std::time::{Duration, Instant};

pub struct CardFetcher {
    collection: Collection,
    tags_manager: Arc<RwLock<TagsManager>>,

    // Cache name -> (Cards, future timestamp when cache timeouts)
    cache: Mutex<LruCache<String, (Vec<Card>, i64)>>,
    // Card id -> Card
    exact_cache: Mutex<LruCache<String, Card>>,
}

impl CardFetcher {
    pub fn new(
        collection: Collection,
        tags_manager: Arc<RwLock<TagsManager>>,
        queries_cache_size: usize,
        exact_card_cache_size: usize,
    ) -> Self {
        CardFetcher {
            collection,
            tags_manager,
            cache: Mutex::new(LruCache::new(queries_cache_size)),
            exact_cache: Mutex::new(LruCache::new(exact_card_cache_size)),
        }
    }

    async fn prepare_card(&self, card: &mut Card) {
        card.markdown2html();
        card.fill_marks();
        // Second call if there is mark in mark
        card.fill_marks();
        card.fill_description();

        let tags_manager = self.tags_manager.read().await;
        tags_manager.fill_card_tags(card).await;
    }

    pub async fn fetch(&self, mut query: CardQuery, cache: bool) -> Result<Vec<Card>> {
        let query_hash = query.to_string();

        if cache {
            if let Ok(mut cache) = self.cache.lock() {
                if let Some((cards, timeouts)) = cache.get_mut(&query_hash) {
                    if Utc::now().timestamp() >= *timeouts {
                        // Invalidate cache, just skip this step
                    } else {
                        // println!("Return cards from cache");
                        return Ok(cards.clone());
                    }
                }
            }
        }

        // This added after query hash
        // TODO: Make indecies from this
        query.query.insert("rewritten", true);
        query.query.insert("categorised", true);
        query.query.insert("tagged", true);

        if !query.query.contains_key("category") {
            query.query.insert(
                "category",
                doc! {
                    "$ne" : format!("{:?}", news_general::category::Category::Unknown)
                },
            );
        }

        // dbg!(&query);

        let options = FindOptions::builder()
            .sort(query.sort)
            .limit(query.limit)
            .allow_disk_use(true)
            .build();

        let mut cards = self.collection.find(query.query, options).await?;

        let mut result = vec![];
        while let Some(card) = cards.next().await {
            let mut card_typed: Card = bson::from_document(card?)?;
            self.prepare_card(&mut card_typed).await;
            result.push(card_typed);
        }

        if cache {
            if let Ok(mut cache) = self.cache.lock() {
                // println!("Fetch cards from DB");
                let when_timeouts = Utc::now() + query.lifetime;
                cache.insert(query_hash, (result.clone(), when_timeouts.timestamp()));
            }
        }

        Ok(result)
    }

    pub async fn fetch_exact(&self, slug: String) -> Result<Card> {
        if let Ok(mut cache) = self.exact_cache.lock() {
            if let Some(card) = cache.get_mut(&slug) {
                // println!("Return exact card from cache");
                return Ok(card.clone());
            }
        }

        let card = self
            .collection
            .find_one(
                doc! {
                    "slug" : &slug
                },
                None,
            )
            .await;

        let mut card: Card = bson::from_document(card?.context("No such card")?)?;
        self.prepare_card(&mut card).await;

        if let Ok(mut cache) = self.exact_cache.lock() {
            // println!("Get exact card from DB");
            cache.insert(slug, card.clone());
        }

        Ok(card)
    }
}
