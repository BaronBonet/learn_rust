use crate::core::domain::{ArticleQuery, DateRange, NewsArticle};
use crate::core::ports;
use async_trait::async_trait;
use std::fmt;
use tokio::sync::mpsc;

pub struct NewsService {
    logger: Box<dyn ports::Logger>,
    news_repository: Box<dyn ports::NewsRepository>,
    news_search_client: std::sync::Arc<dyn ports::NewsSearchClient>,
}

impl NewsService {
    pub fn new(
        logger: Box<dyn ports::Logger>,
        news_repository: Box<dyn ports::NewsRepository>,
        news_search_client: std::sync::Arc<dyn ports::NewsSearchClient>,
    ) -> Self {
        Self {
            logger,
            news_repository,
            news_search_client,
        }
    }
}

#[async_trait]
impl ports::NewsService for NewsService {
    async fn get_articles_by_categories(
        &self,
        categories: Vec<String>,
        date_range: DateRange,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>> {
        self.news_repository
            .get_articles_by_categories(categories, date_range)
            .await
    }

    async fn is_valid_category(
        &self,
        category: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.news_repository.is_valid_category(category).await
    }

    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>> {
        self.news_repository.add_category(category).await
    }

    async fn fetch_and_store_articles(&self, query: ArticleQuery) -> Result<i32, NewsServiceError> {
        let is_valid = self
            .news_repository
            .is_valid_category(query.category.clone())
            .await
            .map_err(NewsServiceError::RepositoryError)?;
        if !is_valid {
            self.logger.debug("category is not valid");
            return Err(NewsServiceError::InvalidCategory(query.category.clone()));
        }
        self.logger.debug("starting fetch and store articles");
        let (channel, mut rx) = mpsc::channel(10000);
        let client = std::sync::Arc::clone(&self.news_search_client);
        tokio::spawn(async move {
            client.query_for_articles(query, channel).await;
        });
        let mut count = 0;
        while let Some(articles) = rx.recv().await {
            match self.news_repository.store_articles(articles).await {
                Ok(num) => count += num,
                Err(e) => self.logger.error(&format!("Error storing articles: {}", e)),
            }
        }
        Ok(count)
    }

    async fn sync_articles(&self, date_range: DateRange) -> Result<i32, NewsServiceError> {
        let categories = self.news_repository.get_categories().await?;
        let countries = self.news_repository.get_countries().await?;
        let queries = ArticleQuery::build_queries(categories, countries, date_range);
        let mut num_articles = 0;
        for query in queries {
            let num = self.fetch_and_store_articles(query).await?;
            num_articles += num;
        }
        self.logger
            .info(&format!("Synced {} articles", num_articles));
        Ok(num_articles)
    }
}

pub enum NewsServiceError {
    InvalidCategory(String),
    RepositoryError(Box<dyn std::error::Error>),
}

impl fmt::Display for NewsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NewsServiceError::InvalidCategory(category) => {
                write!(f, "Invalid category: {}", category)
            }
            NewsServiceError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl From<Box<dyn std::error::Error>> for NewsServiceError {
    fn from(err: Box<dyn std::error::Error>) -> NewsServiceError {
        NewsServiceError::RepositoryError(err)
    }
}
