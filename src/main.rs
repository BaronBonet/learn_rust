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
    sourcecountry: String
}

fn main() {
    let mut now: DateTime<Utc> = Utc::now();
    let yesterday: DateTime<Utc> = Utc::now() - chrono::Duration::days(1);
    let mut all_articles = vec![];

    loop {
        let formatted_now = now.format("%Y%m%d%H%M%S").to_string();
        let formatted_yesterday = yesterday.format("%Y%m%d%H%M%S").to_string();

        let mut params = HashMap::new();
        params.insert("query", "sourcelang:french sourcecountry:FR");
        params.insert("mode", "artlist");
        params.insert("maxrecords", "250");
        params.insert("format", "json");
        params.insert("startdatetime", &formatted_yesterday);
        params.insert("enddatetime", &formatted_now);

        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&maxrecords={}&format={}&startdatetime={}&enddatetime={}",
            params["query"], params["mode"], params["maxrecords"], params["format"], params["startdatetime"], params["enddatetime"]
        );

        let resp = ureq::get(&url).call();

        match resp {
            Ok(response) => {
                if response.status() == 200 {
                    let response_string = response.into_string().unwrap();
                    match serde_json::from_str::<HashMap<String, Vec<Article>>>(&response_string) {
                        Ok(body) => {
                            let articles = &body["articles"];
                            all_articles.extend_from_slice(articles);

                            let last_article = articles.last().unwrap();
                            let mut last_date: Option<DateTime<Utc>> = None;


                            match DateTime::parse_from_rfc3339(&last_article.seendate) {
                                Ok(date) => {
                                    let parsed_date = date.with_timezone(&Utc);

                                    last_date = Some(parsed_date);
                                    println!("{}", last_date.unwrap());
                                    if let Some(last_date_value) = last_date {
                                        if last_date_value < yesterday {
                                            break;
                                        }
                                        println!("Last date: {}", last_article.seendate);

                                        now = last_date_value;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse date: {}. Error: {}", &last_article.seendate, e);
                                    continue
                                }
                            }


                        },
                        Err(_) => {
                            println!("Could not parse response as JSON. Response: {}", response_string);
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

        // Sleep for 5 seconds to avoid getting blocked
        thread::sleep(Duration::from_secs(5));
    }

    // Print all articles
    println!("number of articles: {}", all_articles.len());
    for article in all_articles {
        println!("{:?}", article);
    }
}
