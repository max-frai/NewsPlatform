use std::sync::Mutex;

use crate::card_queries::CardQuery;
use anyhow::Result;
use bson::{doc, oid::ObjectId};
use chrono::prelude::*;
use futures::stream::StreamExt;
use lru_cache::LruCache;
use mongodb::options::FindOptions;
use mongodb::Collection;
use news_general::card::*;

pub struct CardFetcher {
    collection: Collection,

    // Cache name -> (Cards, future timestamp when cache timeouts)
    cache: Mutex<LruCache<String, (Vec<Card>, i64)>>,
    // Card id -> Card
    exact_cache: Mutex<LruCache<String, Card>>,
}

impl CardFetcher {
    pub fn new(
        collection: Collection,
        queries_cache_size: usize,
        exact_card_cache_size: usize,
    ) -> Self {
        CardFetcher {
            collection,
            cache: Mutex::new(LruCache::new(queries_cache_size)),
            exact_cache: Mutex::new(LruCache::new(exact_card_cache_size)),
        }
    }

    // fn query_hash(&self, query: &CardQuery) -> String {
    //     format!("{:?}{:?}{:?}", query.limit, sort, query.query)
    // }

    pub async fn fetch(&self, mut query: CardQuery) -> Result<Vec<Card>> {
        query.query.insert("rewritten", true);
        query.query.insert("categorised", true);
        query.query.insert("tagged", true);

        let query_hash = query.to_string();
        dbg!(&query_hash);

        if let Ok(mut cache) = self.cache.lock() {
            if let Some((cards, timeouts)) = cache.get_mut(&query_hash) {
                if Utc::now().timestamp() >= *timeouts {
                    // Invalidate cache, just skip this step
                } else {
                    return Ok(cards.clone());
                }
            }
        }

        let options = FindOptions::builder()
            .sort(query.sort)
            .limit(query.limit)
            .build();

        let mut cards = self.collection.find(query.query, options).await?;

        let mut result = vec![];
        while let Some(card) = cards.next().await {
            let card_typed: Card = bson::from_document(card?)?;
            result.push(card_typed);
        }

        if let Ok(mut cache) = self.cache.lock() {
            let when_timeouts = Utc::now() + query.lifetime;
            cache.insert(query_hash, (result.clone(), when_timeouts.timestamp()));
        }

        Ok(result)
    }

    pub async fn fetch_exact(&self, id: String) -> Result<Card> {
        if let Ok(mut cache) = self.exact_cache.lock() {
            if let Some(card) = cache.get_mut(&id) {
                return Ok(card.clone());
            }
        }

        let card = self
            .collection
            .find_one(
                doc! {
                    "_id" : ObjectId::with_string(&id).unwrap()
                },
                None,
            )
            .await;

        let card: Card = bson::from_document(card?.unwrap())?;
        if let Ok(mut cache) = self.exact_cache.lock() {
            cache.insert(id, card.clone());
        }

        Ok(card)
    }
}
