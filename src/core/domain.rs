use chrono::NaiveDate;
use serde_json::error::Category;
use iso_country::alpha2;

pub struct NewsArticle {
  pub title: String,
  pub category: ArticleCategory,
  pub date: NaiveDate,
  pub url: String,
  pub domain: String,
  pub language: String,
  pub country: isocountry::alpha2
}

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
    pub fn new(title: String, category: ArticleCategory, date: NaiveDate, url: String, domain: String, language: String, country: iso_country::alpha2) -> Self {
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
