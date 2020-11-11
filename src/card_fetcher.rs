use std::sync::Mutex;

use crate::{card::Card, card_queries::CardQuery};
use anyhow::Result;
use bson::{doc, oid::ObjectId};
use chrono::prelude::*;
use futures::stream::StreamExt;
use lru_cache::LruCache;
use mongodb::Collection;

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

    async fn fetcher(&self, query: &CardQuery) -> Result<Vec<Card>> {
        if let Ok(mut cache) = self.cache.lock() {
            if let Some((cards, timeouts)) = cache.get_mut(&query.name) {
                if Utc::now().timestamp() >= *timeouts {
                    // Invalidate cache, just skip this step
                } else {
                    return Ok(cards.clone());
                }
            }
        }

        let mut cards = self
            .collection
            .find(query.query.clone(), query.options.clone())
            .await?;

        let mut result = vec![];
        while let Some(card) = cards.next().await {
            let card_typed: Card = bson::from_document(card?)?;
            result.push(card_typed);
        }

        if let Ok(mut cache) = self.cache.lock() {
            let when_timeouts = Utc::now() + query.lifetime;
            cache.insert(
                query.name.to_owned(),
                (result.clone(), when_timeouts.timestamp()),
            );
        }

        Ok(result)
    }

    async fn exact_fetcher(&self, id: String) -> Result<Card> {
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

    pub async fn fetch(&self, query: &CardQuery) -> Result<Vec<Card>> {
        self.fetcher(query).await
    }

    pub async fn fetch_exact(&self, id: String) -> Result<Card> {
        self.exact_fetcher(id).await
    }
}
