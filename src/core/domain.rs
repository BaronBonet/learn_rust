use chrono::{DateTime, Utc};
use isocountry::CountryCode;

#[derive(Debug)]
pub struct NewsArticle {
    pub title: String,
    pub category: String,
    pub date: DateTime<Utc>,
    pub url: String,
    pub domain: String,
    pub language: String,
    pub country: CountryCode,
}

#[derive(Debug)]
pub struct ArticleQuery {
    pub source_country: CountryCode,
    pub category: String,
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
}

impl NewsArticle {
    pub fn new(
        title: String,
        category: String,
        date: DateTime<Utc>,
        url: String,
        domain: String,
        language: String,
        country: CountryCode,
    ) -> Self {
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
    pub fn new(
        source_country: CountryCode,
        category: String,
        start_datetime: DateTime<Utc>,
        end_datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            source_country,
            category,
            start_datetime,
            end_datetime,
        }
    }
}

pub struct DateRange {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl DateRange {
    pub fn new(start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Self, &'static str> {
        if start_date <= end_date {
            Ok(Self {
                start_date,
                end_date,
            })
        } else {
            Err("End date must not be before the start date")
        }
    }
}
