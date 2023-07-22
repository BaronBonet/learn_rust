use crate::core::{domain, ports};
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::trace::TraceLayer;

pub struct RestHandler {
    logger: Box<dyn ports::Logger>,
    news_service: Arc<dyn ports::NewsService>,
    port: String,
}

impl RestHandler {
    pub fn new(
        news_service: Arc<dyn ports::NewsService>,
        logger: Box<dyn ports::Logger>,
        port: String,
    ) -> Self {
        Self {
            logger,
            news_service,
            port,
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route(
                "/is-valid-category/:category_name",
                get(is_valid_category_handler),
            )
            .route(
                "/get-articles-by-category",
                get(get_articles_by_categories_handler),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(self.news_service);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port.parse()?));

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct CategoryQuery {
    pub category_name: String,
}

#[derive(Serialize)]
struct CategoryValidityResponse {
    is_valid: bool,
}

async fn is_valid_category_handler(
    State(news_service): State<Arc<dyn ports::NewsService>>,
    category_query: Path<String>,
) -> impl IntoResponse {
    let is_valid = news_service
        .is_valid_category(category_query.to_string())
        .await
        .unwrap_or(false);
    Json(CategoryValidityResponse { is_valid })
}

#[derive(Serialize)]
struct ArticleResponse {
    articles: Vec<domain::NewsArticle>,
}

// TODO is default required?
#[derive(Debug, Deserialize, Default)]
pub struct ArticleQuery {
    pub categories: String,
    #[serde(deserialize_with = "deserialize")]
    pub inclusive_start_date: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize")]
    pub inclusive_end_date: DateTime<Utc>,
}

async fn get_articles_by_categories_handler(
    State(news_service): State<Arc<dyn ports::NewsService>>,
    Query(query): Query<ArticleQuery>,
) -> impl IntoResponse {
    let categories: Vec<String> = query.categories.split(',').map(|s| s.to_string()).collect();
    // TODO validate categories

    let date_range = domain::DateRange::new(query.inclusive_start_date, query.inclusive_end_date)
        .map_err(|e| {
            // TODO handle error
            eprintln!("error: {}", e);
            return e;
        })
        .unwrap();

    let articles = news_service
        .get_articles_by_categories(categories, date_range)
        .await
        .unwrap_or(Vec::new());
    Json(ArticleResponse { articles })
}

// Helper function to deserialize datetime
fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(Error::custom)?;
    Ok(Utc.from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())))
}
