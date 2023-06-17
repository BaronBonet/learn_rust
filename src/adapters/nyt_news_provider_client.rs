use crate::core::domain::NewsArticle;
use crate::core::ports::NewsProviderClient;
use chrono::NaiveDate;
use serde::Deserialize;
use std::error::Error;

const API_URL: &str = "https://api.nytimes.com/svc/topstories/v2/home.json?api-key=";

pub struct NYTimesAdapter {
    api_key: String,
}

impl NYTimesAdapter {
    pub fn new(api_key: String) -> NYTimesAdapter {
        NYTimesAdapter { api_key }
    }
}

impl NewsProviderClient for NYTimesAdapter {
    async fn get_top_stories(&self) -> Result<Vec<NewsArticle>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}{}", API_URL, self.api_key);
        let response: NYTApiResponse = reqwest::get(&url).await?.json().await?;

        if response.results.is_empty() {
            return Err("No articles found".into());
        }

        let main_article = response.results[0].clone();
        let date = NaiveDate::parse_from_str(&main_article.published_date, "%Y-%m-%d")?;

        Ok(vec![NewsArticle ::new(
             main_article.title,
             main_article.abstract_text,
            date,
            main_article.url,
        )])
    }
}

#[derive(Debug, Deserialize, Clone)]
struct NYTApiResponse {
    results: Vec<NYTArticle>,
}

#[derive(Debug, Deserialize, Clone)]
struct NYTArticle {
    title: String,
    #[serde(rename = "abstract")]
    abstract_text: String,
    published_date: String,
    url: String,
}
