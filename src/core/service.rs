use crate::core::domain::{ArticleQuery, DateRange, NewsArticle};
use crate::core::ports;
use async_trait::async_trait;
use std::fmt;
use std::io::{Error, ErrorKind};

pub struct NewsService {
    logger: Box<dyn ports::Logger>,
    news_repository: Box<dyn ports::NewsRepository>,
    news_search_client: Box<dyn ports::NewsSearchClient>,
}

impl NewsService {
    pub fn new(
        logger: Box<dyn ports::Logger>,
        news_repository: Box<dyn ports::NewsRepository>,
        news_search_client: Box<dyn ports::NewsSearchClient>,
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
    async fn get_articles_with_categories(
        &self,
        categories: Vec<String>,
        date_range: DateRange,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>> {
        self.news_repository
            .get_articles_with_categories(categories)
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
        let articles = self.news_search_client.query_for_articles(query);
        self.news_repository
            .store_articles(articles)
            .await
            .map_err(NewsServiceError::RepositoryError)
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
