use chrono::{DateTime, Utc};
use isocountry::CountryCode;

pub struct NewsArticle {
  pub title: String,
  pub category: ArticleCategory,
  pub date: DateTime<Utc>,
  pub url: String,
  pub domain: String,
  pub language: String,
  pub country: CountryCode,
}

pub struct ArticleQuery {
    pub source_country: CountryCode,
    pub category: ArticleCategory,
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
}

#[derive(Copy, Clone)]
pub enum ArticleCategory {
    ClimateChange,
    GlobalWarming,
}

impl NewsArticle {
    pub fn category_as_string(&self) -> &str {
        match &self.category {
            ArticleCategory::ClimateChange => "Climate Change",
            ArticleCategory::GlobalWarming => "Global Warming",
        }
    }
    pub fn new(title: String, category: ArticleCategory, date: DateTime<Utc>, url: String, domain: String, language: String, country: CountryCode) -> Self {
        Self {
            title,
            category,
            date,
            url,
            domain,
            language,
            country,
        }
    }
}

impl ArticleQuery {
    pub fn new(source_country: CountryCode, category: ArticleCategory, start_datetime: DateTime<Utc>, end_datetime: DateTime<Utc>) -> Self {
        Self {
            source_country,
            category,
            start_datetime,
            end_datetime,
        }
    }
}