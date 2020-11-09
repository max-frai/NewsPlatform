use crate::card::Card;
use anyhow::Result;
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Collection};

pub enum CardFetcherKind {
    Index,
    Exact(String),
}

pub struct CardFetcher {
    collection: Collection,
}

impl CardFetcher {
    pub fn new(collection: Collection) -> Self {
        CardFetcher { collection }
    }

    async fn index_fetcher(&self) -> Result<Vec<Card<()>>> {
        let opts = FindOptions::builder()
            .limit(10)
            .sort(Some(doc! {
                "date" : -1
            }))
            .build();

        let mut cards = self
            .collection
            .find(
                doc! {
                    "country" : "ua"
                },
                opts,
            )
            .await?;

        let mut result = vec![];
        while let Some(card) = cards.next().await {
            let card_typed: Card<()> = bson::from_document(card?)?;
            result.push(card_typed);
        }

        Ok(result)
    }

    async fn exact_fetcher(&self, id: String) -> Result<Vec<Card<()>>> {
        let card = self
            .collection
            .find_one(
                doc! {
                    "_id" : ObjectId::with_string(&id).unwrap()
                },
                None,
            )
            .await;

        let card = bson::from_document(card?.unwrap())?;

        Ok(vec![card])
    }

    pub async fn fetch(&self, kind: CardFetcherKind) -> Result<Vec<Card<()>>> {
        match kind {
            CardFetcherKind::Index => self.index_fetcher().await,
            CardFetcherKind::Exact(id) => self.exact_fetcher(id).await,
        }
    }
}
