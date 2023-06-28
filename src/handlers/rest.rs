use crate::core::{ports, service};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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
