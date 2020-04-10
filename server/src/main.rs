mod api;
mod auth;
mod config;
mod models;

use std::sync::Arc;

pub type Tx = sqlx::Transaction<PgPoolConn>;
pub type PgPoolConn = sqlx::pool::PoolConnection<sqlx::PgConnection>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Build needed state items for API.
    let cfg = Arc::new(config::Config::new());
    let db = sqlx::PgPool::builder()
        .max_size(10)
        .build(&cfg.database_url)
        .await?;
    let client = reqwest::Client::new();

    // Build & start the API.
    api::new(db, client, cfg.clone())?.await?; // Blocks.
    Ok(())
}
