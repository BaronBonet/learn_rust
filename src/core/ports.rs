use crate::core::domain::{ArticleQuery, DateRange, NewsArticle};
use crate::core::service;
use async_trait::async_trait;

#[async_trait]
pub trait NewsService: Send {
    // Retrieves articles from the repository with the provided categories
    async fn get_articles_with_categories(
        &self,
        category: Vec<String>,
        date_range: DateRange,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>>;

    async fn is_valid_category(&self, category: String)
        -> Result<bool, Box<dyn std::error::Error>>;

    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>>;

    async fn fetch_and_store_articles(
        &self,
        query: ArticleQuery,
    ) -> Result<i32, service::NewsServiceError>;
}

pub trait NewsSearchClient: Send + Sync {
    fn query_for_articles(&self, query: ArticleQuery) -> Vec<NewsArticle>;
}

#[async_trait]
pub trait NewsRepository: Send + Sync {
    async fn get_articles_with_categories(
        &self,
        categories: Vec<String>,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>>;
    async fn store_articles(
        &self,
        articles: Vec<NewsArticle>,
    ) -> Result<i32, Box<dyn std::error::Error>>;
    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>>;
    async fn is_valid_category(&self, category: String)
        -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Logger: Send + Sync {
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn fatal(&self, msg: &str);
}
