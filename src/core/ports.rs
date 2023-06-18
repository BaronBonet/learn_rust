use crate::core::domain;

use iso_country::alpha2;
use chrono::NaiveDate;

pub trait NewsSearchAdapter {
    fn query_for_articles(&self, query: domain::ArticleQuery) -> Vec<domain::NewsArticle>;
}