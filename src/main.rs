mod adapters;
mod core;
mod handlers;
mod infrastructure;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
extern crate ureq;
use std::env;

use crate::adapters::logger_slog::SlogLoggerAdapter;
use crate::adapters::news_search_client_gdeltproject;
use crate::chrono::Utc;
use crate::core::domain::{ArticleQuery, NewsArticle};
use crate::core::ports::{Logger, NewsRepository, NewsSearchClient};
use crate::core::service::NewsServiceError;
use csv::Writer;
use isocountry::CountryCode;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let logger = Box::new(SlogLoggerAdapter::new());
    logger.info("Application has started");
    let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| String::from("postgres"));
    let db_password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| String::from("postgres"));
    let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| String::from("postgres"));
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| String::from("localhost"));
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| String::from("15432"));

    let pool =
        infrastructure::postgres::get_db_pool(db_user, db_password, db_name, db_host, db_port)
            .await
            .expect("Failed to connect to Postgres");
    logger.info("Successfully connected to Postgres");

    let repo =
        adapters::news_repository_postgres::PostgresNewsRepository::new(pool, logger.clone());

    let g_delta_project_adapter =
        news_search_client_gdeltproject::GDeltaProjectNewsSearchAdapter::new(logger.clone());
    let news_service = Arc::new(core::service::NewsService::new(
        logger.clone(),
        Box::new(repo),
        Box::new(g_delta_project_adapter),
    ));

    // psudo code
    let rest_handler =
        handlers::rest::RestHandler::new(news_service, logger.clone(), "3000".to_string());
    rest_handler.start().await.unwrap();
}
// let end_datetime = Utc::now();
// let start_datetime = end_datetime - chrono::Duration::days(1);
// let q = ArticleQuery::new(
// CountryCode::FRA,
// "fake".to_string(),
// start_datetime,
// end_datetime,
// );
// match news_service.fetch_and_store_articles(q).await {
// Ok(num) => logger.info(&format!("Successfully stored {} articles.", num)),
// Err(NewsServiceError::InvalidCategory(category)) => logger.warn(&format!(
//     "Invalid category passed to fetch_and_store_articles: {}",
//     category
// )),
// Err(NewsServiceError::RepositoryError(err)) => {
// logger.error(&format!("Failed to fetch and store articles: {}", err))
// }
// }
