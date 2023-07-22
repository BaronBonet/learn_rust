use learn_rust::adapters;
use learn_rust::adapters::logger_slog::SlogLoggerAdapter;
use learn_rust::adapters::news_search_client_gdeltproject;
use learn_rust::core;
use learn_rust::core::ports::Logger;
use learn_rust::core::ports::NewsService;
use learn_rust::handlers;
use learn_rust::infrastructure;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
extern crate ureq;
use std::env;

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

    let date_range = match core::domain::DateRange::new(
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now(),
    )
    .map_err(|e| e.to_string())
    {
        Ok(date_range) => date_range,
        Err(e) => panic!("{}", e),
    };

    match news_service
        .sync_articles(date_range)
        .await
        .map_err(|e| e.to_string())
    {
        Ok(_) => logger.info("Successfully synced articles"),
        Err(e) => panic!("{}", e),
    };
}
