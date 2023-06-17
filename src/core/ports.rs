use crate::core::domain::NewsArticle;

use chrono::NaiveDate;

pub trait NewsProviderClient {
    fn get_top_stories(&self) -> Vec<NewsArticle>;
}