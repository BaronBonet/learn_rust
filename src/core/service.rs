use crate::core::domain::{ArticleQuery, NewsArticle};
use crate::core::ports;
use crate::core::ports::{NewsRepository, NewsSearchClient};

pub struct NewsService {
    news_repository: Box<dyn NewsRepository>,
    news_search_client: Box<dyn NewsSearchClient>,
}

impl NewsService {
    pub fn new(
        news_repository: Box<dyn NewsRepository>,
        news_search_client: Box<dyn NewsSearchClient>,
    ) -> Self {
        Self {
            news_repository,
            news_search_client,
        }
    }
}

impl NewsService {
    pub async fn get_articles_with_categories(
        &self,
        categories: Vec<String>,
    ) -> Result<Vec<NewsArticle>, Box<dyn std::error::Error>> {
        self.news_repository
            .get_articles_with_categories(categories)
            .await
    }

    pub async fn is_valid_category(
        &self,
        category: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.news_repository.is_valid_category(category).await
    }

    pub async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>> {
        self.news_repository.add_category(category).await
    }

    pub async fn fetch_and_store_articles(
        &self,
        query: ArticleQuery,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        println!("starting fetch and store articles");
        let articles = self.news_search_client.query_for_articles(query);
        println!("articles: {:?}", articles);
        self.news_repository.store_articles(articles).await
    }
}
