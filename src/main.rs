#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate ureq;
extern crate serde_json;

use chrono::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
struct Article {
    url: String,
    url_mobile: String,
    title: String,
    seendate: String,
    socialimage: String,
    domain: String,
    language: String,
    sourcecountry: String,
}

fn build_url(now: DateTime<Utc>, yesterday: DateTime<Utc>) -> String {
    let formatted_now = now.format("%Y%m%d%H%M%S").to_string();
    let formatted_yesterday = yesterday.format("%Y%m%d%H%M%S").to_string();

    let mut params = HashMap::new();
    params.insert("query", "sourcelang:french sourcecountry:FR");
    params.insert("mode", "artlist");
    params.insert("maxrecords", "250");
    params.insert("format", "json");
    params.insert("startdatetime", &formatted_yesterday);
    params.insert("enddatetime", &formatted_now);

    format!(
        "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&maxrecords={}&format={}&startdatetime={}&enddatetime={}",
        params["query"], params["mode"], params["maxrecords"], params["format"], params["startdatetime"], params["enddatetime"]
    )
}

fn get_articles_from_response(response: ureq::Response) -> Result<Vec<Article>, serde_json::Error> {
    let response_string = response.into_string().unwrap();
    let body: HashMap<String, Vec<Article>> = serde_json::from_str(&response_string)?;
    Ok(body["articles"].clone())
}

fn update_date(last_article: &Article, yesterday: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, chrono::ParseError> {
    let date = NaiveDateTime::parse_from_str(&last_article.seendate, "%Y%m%dT%H%M%SZ")?;

    let parsed_date = DateTime::from_utc(date, Utc);


    if parsed_date < yesterday {
        Ok(None)
    } else {
        Ok(Some(parsed_date))
    }
}

fn main() {
    let mut now: DateTime<Utc> = Utc::now();
    let yesterday: DateTime<Utc> = Utc::now() - chrono::Duration::days(1);
    let mut all_articles = vec![];

    loop {
        let url = build_url(now, yesterday);
        let resp = ureq::get(&url).call();

        match resp {
            Ok(response) => {
                if response.status() == 200 {
                    match get_articles_from_response(response) {
                        Ok(mut articles) => {
                            println!("Fetched {} articles.", articles.len());
                            let last_article = articles.last().unwrap();

                            match update_date(last_article, yesterday) {
                                Ok(last_date_option) => {
                                    match last_date_option {
                                        Some(last_date) => {
                                            if last_date == now {
                                                println!("No more new articles to fetch.");
                                                break;
                                            }
                                            println!("{}", last_date);
                                            now = last_date;
                                            println!("Now: {}", now);
                                            all_articles.append(&mut articles);
                                        }
                                        None => {
                                            println!("No more articles to fetch.");
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse date: {}. Error: {}", &last_article.seendate, e);
                                    continue
                                }
                            }
                        },
                        Err(_) => {
                            println!("Could not parse response as JSON.");
                        }
                    }
                } else {
                    println!("HTTP request failed: {}", response.status());
                }
            }
            Err(e) => {
                println!("HTTP request error: {}", e);
            }
        }
        if let Some(last_article) = all_articles.last() {
            println!("{:?}", last_article);
        } else {
            println!("The vector is empty!");
        }

        // Sleep for 5 seconds to avoid getting blocked
        thread::sleep(Duration::from_secs(5));
    }

    // Print all articles
    println!("number of articles: {}", all_articles.len());
    for article in all_articles {
        println!("{:?}", article);
    }
}
