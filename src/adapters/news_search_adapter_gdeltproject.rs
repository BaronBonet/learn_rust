use crate::core::domain::{ArticleCategory, ArticleQuery, NewsArticle};
use crate::core::ports::NewsSearchAdapter;
use chrono::format::ParseError;
use chrono::prelude::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use isocountry::full::{ISO_FULL_FRA, ISO_FULL_GBR};
use isocountry::{full, CountryCode};
use std::collections::HashMap;
use std::error::Error;

pub struct GDeltaProjectNewsSearchAdapter {}

impl NewsSearchAdapter for GDeltaProjectNewsSearchAdapter {
    fn query_for_articles(&self, query: ArticleQuery) -> Vec<NewsArticle> {
        let mut start_time = query.start_datetime;
        println!(
            "Fetching articles between {} and {}...",
            start_time.format("%Y-%m-%d %H:%M:%S"),
            query.end_datetime.format("%Y-%m-%d %H:%M:%S")
        );
        let mut all_articles: Vec<NewsArticle> = vec![];

        while start_time < query.end_datetime {
            println!("{}", start_time);
            println!("{}", query.end_datetime);
            let resp: ureq::Response;
            match call_url(start_time, query.end_datetime, query.source_country) {
                Ok(response) => {
                    resp = response;
                }
                Err(err) => {
                    println!("Error calling URL: {}", err);
                    break;
                }
            }
            let articles: Vec<GDeltaArticle>;
            match extract_articles_from_response(resp) {
                Ok(ars) => {
                    articles = ars;
                }
                Err(err) => {
                    println!("Error extracting articles: {err}");
                    break;
                }
            }
            let mut news_articles =
                to_news_article(articles, &query.category, query.source_country);
            match news_articles.len() {
                0 => {
                    println!("No articles found");
                    break;
                }
                1..=249 => {
                    all_articles.append(&mut news_articles);
                    break;
                }
                250.. => {
                    let t = news_articles.last().unwrap().date;
                    println!("Latest article date: {}", t);
                    if t == start_time {
                        println!("Latest article date is the same as start_time adding one second");
                        start_time = t + chrono::Duration::seconds(1);
                    } else {
                        start_time = t;
                    }
                    println!("Latest start_time date: {}", start_time);
                    // TODO here it should go on a channel for persisting, but get the basics working 1st
                    all_articles.append(&mut news_articles);
                }
                _ => {
                    println!("An unexpected length was returned");
                    break;
                }
            }
        }
        all_articles
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GDeltaArticle {
    url: String,
    title: String,
    seendate: String,
    domain: String,
    language: String,
    sourcecountry: String,
}

fn build_url(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    source_country: CountryCode,
    category: ArticleCategory,
) -> String {
    // https://blog.gdeltproject.org/gdelt-doc-2-0-api-debuts/
    let formatted_start_time = start_time.format("%Y%m%d%H%M%S").to_string();
    let formatted_end_time = end_time.format("%Y%m%d%H%M%S").to_string();
    let mut params = HashMap::new();
    // TODO implement other categories
    params.insert(
        "query",
        format!(
            "sourcecountry:{}%20AND%20%22climate%20change%22",
            source_country.alpha2()
        )
        .to_string(),
    );
    params.insert("mode", "artlist".to_string());
    params.insert("maxrecords", "250".to_string());
    params.insert("format", "json".to_string());
    params.insert("startdatetime", formatted_start_time.to_string());
    params.insert("enddatetime", formatted_end_time.to_string());

    format!(
        "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&maxrecords={}&format={}&startdatetime={}&enddatetime={}&sort=datedesc",
        params["query"], params["mode"], params["maxrecords"], params["format"], params["startdatetime"], params["enddatetime"]
    )
}

fn extract_articles_from_response(
    response: ureq::Response,
) -> Result<Vec<GDeltaArticle>, Box<dyn Error>> {
    let response_string = match response.into_string() {
        Err(e) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error reading response body: {}", e),
            )))
        }
        Ok(s) => s,
    };

    let body: Result<HashMap<String, Vec<GDeltaArticle>>, serde_json::Error> =
        serde_json::from_str(&response_string);

    match body {
        Ok(body) => Ok(body["articles"].clone()),
        Err(_) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error parsing response body: {}", response_string),
        ))),
    }
}

fn call_url(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    source_country: CountryCode,
) -> Result<ureq::Response, Box<dyn Error>> {
    // Start time must be before end time
    let url = build_url(start_time, end_time, source_country);
    println!("Fetching articles from {}... ", url);
    let resp = ureq::get(&url).call();
    match resp {
        Ok(response) => match response.status() {
            200 => Ok(response),
            _ => {
                let status = response.status();
                let body = response.into_string().unwrap();
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP error, status: {}, body: {}", status, body),
                )))
            }
        },
        Err(e) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error calling URL: {}", e),
        ))),
    }
}

fn to_news_article(
    articles: Vec<GDeltaArticle>,
    category: &ArticleCategory,
    source_country: CountryCode,
) -> Vec<NewsArticle> {
    articles
        .iter()
        .filter_map(|element| {
            let date = to_datetime(&element.seendate);
            if !date.is_ok() {
                println!("Error parsing date: {}", element.seendate);
                return None;
            }

            let country = to_country(&element.sourcecountry, source_country);
            if country.is_none() {
                println!("Country not supported: {}", element.sourcecountry);
                return None;
            }

            Some(NewsArticle {
                title: element.title.clone(),
                category: *category,
                date: date.unwrap().into(),
                url: element.url.clone(),
                domain: element.domain.clone(),
                language: element.language.clone(),
                country: CountryCode::FRA,
            })
        })
        .collect()
}

fn to_datetime(date: &String) -> Result<DateTime<Utc>, ParseError> {
    let date = NaiveDateTime::parse_from_str(date, "%Y%m%dT%H%M%SZ")?;
    Ok(DateTime::<Utc>::from_utc(date, Utc))
}

fn to_country(country_name: &String, source_country: CountryCode) -> Option<CountryCode> {
    if country_name == source_country.name() {
        return Some(source_country);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_to_country() {
        let country = to_country(&"France".to_string(), CountryCode::FRA);
        assert_eq!(country, Some(CountryCode::FRA));
        let country = to_country(&"Fake".to_string(), CountryCode::FRA);
        assert_eq!(country, None);
    }

    #[test]
    fn test_to_datetime() {
        let date = to_datetime(&"20230624T121500Z".to_string()).unwrap();
        let d = Utc.with_ymd_and_hms(2023, 6, 24, 12, 15, 0).unwrap();
        assert_eq!(date, d);
        let invalid_date = to_datetime(&"invalid_date".to_string());
        assert!(invalid_date.is_err());
    }

    #[test]
    fn test_build_url() {
        let start_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap();
        let url = build_url(start_time, end_time, CountryCode::FRA);
        assert_eq!(
            url,
            "https://api.gdeltproject.org/api/v2/doc/doc?query=sourcecountry:FR%20AND%20%22climate%20change%22&mode=artlist&maxrecords=250&format=json&startdatetime=20210101000000&enddatetime=20210102000000&sort=datedesc"
        );
    }

    #[test]
    fn test_to_news_article() {
        let mut articles = Vec::new();

        let valid_article = GDeltaArticle {
            url: "https://example.com".to_string(),
            title: "Valid Article".to_string(),
            seendate: "20230624T121500Z".to_string(),
            domain: "example.com".to_string(),
            language: "French".to_string(),
            sourcecountry: "France".to_string(),
        };
        articles.push(valid_article);

        let invalid_date_article = GDeltaArticle {
            url: "https://invalid.com".to_string(),
            title: "Invalid Date Article".to_string(),
            seendate: "invalid_date".to_string(),
            domain: "invalid.com".to_string(),
            language: "French".to_string(),
            sourcecountry: "France".to_string(),
        };
        articles.push(invalid_date_article);

        let invalid_country_article = GDeltaArticle {
            url: "https://invalid.com".to_string(),
            title: "Invalid Country Article".to_string(),
            seendate: "20230624T121500Z".to_string(),
            domain: "invalid.com".to_string(),
            language: "French".to_string(),
            sourcecountry: "INVALID".to_string(),
        };
        articles.push(invalid_country_article);

        let category = ArticleCategory::ClimateChange;
        let news_articles = to_news_article(articles, &category, CountryCode::FRA);

        assert_eq!(news_articles.len(), 1);
        assert_eq!(news_articles[0].title, "Valid Article");
    }
}
