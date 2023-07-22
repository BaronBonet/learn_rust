use chrono::{DateTime, Utc};
use isocountry::CountryCode;
use serde::Serializer;
use thiserror::Error;

#[derive(Debug, serde::Serialize)]
pub struct NewsArticle {
    pub title: String,
    pub category: String,
    #[serde(serialize_with = "serialize")]
    pub datetime: DateTime<Utc>,
    pub url: String,
    pub domain: String,
    pub language: String,
    pub country: CountryCode,
}

#[derive(Debug)]
pub struct ArticleQuery {
    pub source_country: CountryCode,
    pub category: String,
    pub date_range: DateRange,
}

impl NewsArticle {
    pub fn new(
        title: String,
        category: String,
        datetime: DateTime<Utc>,
        url: String,
        domain: String,
        language: String,
        country: CountryCode,
    ) -> Self {
        Self {
            title,
            category,
            datetime,
            url,
            domain,
            language,
            country,
        }
    }
}

impl ArticleQuery {
    pub fn new(source_country: CountryCode, category: String, date_range: DateRange) -> Self {
        Self {
            source_country,
            category,
            date_range,
        }
    }
    pub fn build_queries(
        categories: Vec<String>,
        countries: Vec<CountryCode>,
        date_range: DateRange,
    ) -> Vec<ArticleQuery> {
        let mut queries = Vec::new();
        for country in countries {
            for category in &categories {
                queries.push(ArticleQuery::new(
                    country.clone(),
                    category.clone(),
                    date_range.clone(),
                ));
            }
        }
        queries
    }
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub inclusive_start_date: DateTime<Utc>,
    pub inclusive_end_date: DateTime<Utc>,
}

impl DateRange {
    pub fn new(start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Self, DateRangeError> {
        if start_date < end_date {
            Ok(Self {
                inclusive_start_date: start_date,
                inclusive_end_date: end_date,
            })
        } else {
            Err(DateRangeError::InvalidDateRange)
        }
    }
}

#[derive(Debug, Error)]
pub enum DateRangeError {
    #[error("Start date must be before end date")]
    InvalidDateRange,
}

// Helper function to serialize datetime
fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.to_rfc3339();
    serializer.serialize_str(&s)
}
