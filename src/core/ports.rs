use crate::core::domain::{ArticleQuery, DateRange, NewsArticle};
use crate::core::service;
use async_trait::async_trait;
use isocountry::CountryCode;
use tokio::sync::mpsc;

#[async_trait]
pub trait NewsService: Send + Sync {
    // Retrieves articles from the repository with the provided categories
    async fn get_articles_by_categories(
        &self,
        category: Vec<String>,
        date_range: DateRange,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>>;

    async fn is_valid_category(&self, category: String)
        -> Result<bool, Box<dyn std::error::Error>>;

    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>>;

    // Fetches articles from the news search client and stores them in the repository
    // Based in an ArticleQuery
    async fn fetch_and_store_articles(
        &self,
        query: ArticleQuery,
    ) -> Result<i32, service::NewsServiceError>;

    // Sync articles fetches all articles for the countries and categories we have in our DB
    // for the provided date range <- this is meant for a cron job type of task
    async fn sync_articles(&self, date_range: DateRange) -> Result<i32, service::NewsServiceError>;
}

#[async_trait]
pub trait NewsSearchClient: Send + Sync {
    async fn query_for_articles(
        &self,
        query: ArticleQuery,
        channel: mpsc::Sender<Vec<NewsArticle>>,
    );
}

#[async_trait]
pub trait NewsRepository: Send + Sync {
    async fn get_articles_by_categories(
        &self,
        categories: Vec<String>,
        date_range: DateRange,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>>;

    async fn store_articles(
        &self,
        articles: Vec<NewsArticle>,
    ) -> Result<i32, Box<dyn std::error::Error>>;

    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>>;

    async fn is_valid_category(&self, category: String)
        -> Result<bool, Box<dyn std::error::Error>>;

    async fn get_categories(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn get_countries(&self) -> Result<Vec<CountryCode>, Box<dyn std::error::Error>>;
}

pub trait Logger: Send + Sync {
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn fatal(&self, msg: &str);
    fn clone_box(&self) -> Box<dyn Logger>; // add a clone_box method
}
