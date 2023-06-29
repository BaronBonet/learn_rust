use crate::core::domain::{ArticleQuery, NewsArticle};
use crate::core::ports;
use crate::core::ports::NewsSearchClient;
use chrono::format::ParseError;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use isocountry::CountryCode;
use std::collections::HashMap;
use std::error::Error;
use urlencoding::encode;

pub struct GDeltaProjectNewsSearchAdapter {
    logger: Box<dyn ports::Logger>,
}
impl GDeltaProjectNewsSearchAdapter {
    pub fn new(logger: Box<dyn ports::Logger>) -> Self {
        Self { logger }
    }
}

impl NewsSearchClient for GDeltaProjectNewsSearchAdapter {
    fn query_for_articles(&self, query: ArticleQuery) -> Vec<NewsArticle> {
        let mut start_time = query.date_range.inclusive_start_date;
        self.logger.debug(
            format!(
                "Fetching articles between {} and {}...",
                start_time.format("%Y-%m-%d %H:%M:%S"),
                query
                    .date_range
                    .inclusive_end_date
                    .format("%Y-%m-%d %H:%M:%S")
            )
            .as_str(),
        );
        let mut all_articles: Vec<NewsArticle> = vec![];

        while start_time < query.date_range.inclusive_end_date {
            let resp = match call_url(
                start_time,
                query.date_range.inclusive_end_date,
                query.source_country,
                query.category.to_string(),
            ) {
                Ok(response) => response,
                Err(err) => {
                    self.logger
                        .warn(format!("Error calling URL: {}", err).as_str());
                    break; // Skip this iteration and continue with the next one.
                }
            };

            let articles: Vec<GDeltaArticle>;
            match extract_articles_from_response(resp) {
                Ok(ars) => {
                    articles = ars;
                }
                Err(_err) => {
                    self.logger.warn("Error extracting articles: {err}");

                    break;
                }
            }
            let mut news_articles =
                to_news_article(articles, &query.category, query.source_country);
            match news_articles.len() {
                0 => {
                    self.logger.debug("No articles found");
                    break;
                }
                1..=249 => {
                    self.logger.debug("Less than 250 articles found");
                    all_articles.append(&mut news_articles);
                    break;
                }
                // Since we hard code 250 results from the api
                250.. => {
                    let t = news_articles.last().unwrap().datetime;
                    self.logger
                        .debug(format!("Latest article date: {}", t).as_str());
                    if t == start_time {
                        self.logger.warn(
                            "Latest article date is the same as start_time adding one second",
                        );
                        // TODO this is a bit of a hack becase if there are more than 250 articles with the same datetime then we will never get the ones beyond 250
                        //  There may be ways around this we will have to play with the api
                        //  This should be logged with a warning
                        start_time = t + chrono::Duration::seconds(1);
                    } else {
                        start_time = t;
                    }
                    self.logger
                        .debug(format!("Latest start_time date: {}", start_time).as_str());
                    // TODO here it should go on a channel for persisting, but get the basics working 1st
                    all_articles.append(&mut news_articles);
                }
                _ => {
                    self.logger.error("An unexpected length was returned");
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
    category: String,
) -> String {
    let formatted_start_time = start_time.format("%Y%m%d%H%M%S").to_string();
    let formatted_end_time = end_time.format("%Y%m%d%H%M%S").to_string();

    let query = format!(
        "sourcecountry:{} AND \"{}\"",
        source_country.alpha2(),
        category
    );

    let query = encode(&query);

    format!(
        "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&maxrecords=250&format=json&startdatetime={}&enddatetime={}&sort=datedesc",
        query, formatted_start_time, formatted_end_time
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
    category: String,
) -> Result<ureq::Response, Box<dyn Error>> {
    let url = build_url(start_time, end_time, source_country, category);
    println!("Fetching articles from {}... ", url);
    let resp = ureq::get(&url).call()?;

    match resp.status() {
        200 => Ok(resp),
        status => {
            let body = resp.into_string().unwrap();
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP error, status: {}, body: {}", status, body),
            )))
        }
    }
}

fn to_news_article(
    articles: Vec<GDeltaArticle>,
    category: &String,
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
                category: category.to_string(),
                datetime: date.unwrap().into(),
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
        let url = build_url(
            start_time,
            end_time,
            CountryCode::FRA,
            "climate change".to_string(),
        );
        assert_eq!(
            url,
            "https://api.gdeltproject.org/api/v2/doc/doc?query=sourcecountry%3AFR%20AND%20%22climate%20change%22&mode=artlist&maxrecords=250&format=json&startdatetime=20210101000000&enddatetime=20210102000000&sort=datedesc"
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

        let news_articles =
            to_news_article(articles, &"climate change".to_string(), CountryCode::FRA);

        assert_eq!(news_articles.len(), 1);
        assert_eq!(news_articles[0].title, "Valid Article");
    }
}
