use std::sync::Arc;

use mongodb::bson::*;
use mongodb::Database;
use news_general::constants::AppConfig;

pub async fn ensure_indecies(db: Database, constants: Arc<AppConfig>) {
    println!("Ensure database indecies...");
    db.run_command(
        doc! {
            "createIndexes" : &constants.cards_collection_name,
            "indexes" : vec![
                doc! {
                    "key" : doc! {
                        "link" : 1
                    },
                    "name" : "UniqueLinks",
                    "unique" : true
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to ensure indecies");
}
