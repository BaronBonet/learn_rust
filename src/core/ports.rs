use crate::core::domain;

use iso_country::alpha2;
use chrono::NaiveDate;

pub trait NewsSearchClient {
    fn query_for_articles(&self, source_country: isocountry::alpha2, category: domain::Category, start_datetime: NaiveDate, end_datetime: NaiveDate) -> Vec<domain::NewsArticle>;
}