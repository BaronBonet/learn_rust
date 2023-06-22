use crate::core::domain;


pub trait NewsSearchAdapter {
    fn query_for_articles(&self, query: domain::ArticleQuery) -> Vec<domain::NewsArticle>;
}