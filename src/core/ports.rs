use crate::core::domain::{ArticleQuery, DateRange, NewsArticle};

pub trait NewsService {
    // Retrieves articles from the repository with the provided categories
    fn get_articles_with_categories(
        &self,
        category: Vec<String>,
        date_range: DateRange,
    ) -> Vec<NewsArticle>;
    // Validates if the category provided is valid
    fn is_valid_category(&self, category: String) -> bool;
    // Adds a new category to the repository
    fn add_category(&self, category: String) -> bool;
    // Runs the full pipeline which queries for articles from the NewsSearchClient and stores the new articles
    fn fetch_and_store_articles(&self, query: ArticleQuery) -> i32;
}

pub trait NewsSearchClient {
    fn query_for_articles(&self, query: ArticleQuery) -> Vec<NewsArticle>;
}

pub trait NewsRepository {
    fn get_articles_with_categories(&self, category: Vec<String>) -> Vec<NewsArticle>;
    fn store_articles(&self, articles: Vec<NewsArticle>) -> i32;
    fn add_category(&self, category: String) -> bool;
    fn is_valid_category(&self, category: String) -> bool;
}
