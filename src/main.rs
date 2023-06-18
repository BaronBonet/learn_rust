#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate ureq;
extern crate serde_json;

use std::error::Error;
use chrono::prelude::*;
use std::collections::HashMap;
use std::string::ParseError;
use std::thread;
use std::time::Duration;
use csv::Writer;

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

fn build_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> String {
    let formatted_start_time = start_time.format("%Y%m%d%H%M%S").to_string();
    let formatted_end_time = end_time.format("%Y%m%d%H%M%S").to_string();

    let mut params = HashMap::new();
    params.insert("query", "sourcelang:french sourcecountry:FR");
    params.insert("mode", "artlist");
    params.insert("maxrecords", "250");
    params.insert("format", "json");
    params.insert("startdatetime", &formatted_end_time);
    params.insert("enddatetime", &formatted_start_time);

    format!(
        "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&maxrecords={}&format={}&startdatetime={}&enddatetime={}",
        params["query"], params["mode"], params["maxrecords"], params["format"], params["startdatetime"], params["enddatetime"]
    )
}


fn extract_articles_from_response(response: ureq::Response) -> Result<Vec<Article>, Box<dyn Error>> {
    let response_string = match response.into_string(){
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error reading response body: {}", e)))),
        Ok(s) => s,
    };

    let body: Result<HashMap<String, Vec<Article>>, serde_json::Error> = serde_json::from_str(&response_string);

    match body {
        Ok(body) => Ok(body["articles"].clone()),
        Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error parsing response body: {}", response_string)))),
    }
}


fn call_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<ureq::Response, Box<dyn Error>> {
    // Start time must be before end time
    let url = build_url(start_time, end_time);
    println!("Fetching articles from {}... ", url);
    let resp = ureq::get(&url).call();
    match resp {
        Ok(response) => {
            match response.status() {
                200 => Ok(response),
                _ => {
                    let status = response.status();
                    let body = response.into_string().unwrap();
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("HTTP error, status: {}, body: {}",status,  body))))
                }
            }
        },
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error calling URL: {}", e)))),
    }
}

fn extract_date(article: &Article) -> Result<DateTime<Utc>, chrono::ParseError> {
    let date = NaiveDateTime::parse_from_str(&article.seendate, "%Y%m%dT%H%M%SZ")?;
    Ok(DateTime::from_utc(date, Utc))
}

fn fetch_articles_between(mut start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<Article> {
    println!("Fetching articles between {} and {}...", start_time.format("%Y-%m-%d %H:%M:%S"), end_time.format("%Y-%m-%d %H:%M:%S"));
    let mut all_articles = vec![];

    while start_time > end_time {
        let resp: ureq::Response;
        match call_url(start_time, end_time) {
            Ok(response) => {
                resp = response;
            },
            Err(err) => {
                println!("Error calling URL: {}", err);
                break;
            },
        }
        let mut articles: Vec<Article>;
        match extract_articles_from_response(resp) {
            Ok(ars) => {
                articles = ars;
            },
            Err(err) => {
                println!("Error extracting articles: {}", err);
                break;
            }
        }
        println!("Number of articles before match: {}", articles.len());
        match articles.len() {
            0 => {
                println!("No articles found");
                break;
            },
            1..=249 => {
                println!("{} articles found, less than 250 so stopping", articles.len());
                all_articles.append(&mut articles);
                break;
            },
            250.. => {
                match  extract_date(articles.last().unwrap()) {
                    Ok(date) => {
                        start_time = date;
                        println!("Latest article date: {}", date)
                    },
                    Err(err) => {
                        println!("Error extracting date: {}", err);
                        break;
                    }
                };
                all_articles.append( &mut articles);
            }
            _ => {
                println!("An unexpected length was returned");
                break;
            }
        }
        // Sleep for 5 seconds to avoid getting blocked
        // std::thread::sleep(std::time::Duration::from_secs(5));
    }
    all_articles
}


fn main() {
    let now: DateTime<Utc> = Utc::now();
    let all_articles = fetch_articles_between(now - chrono::Duration::days(0), now - chrono::Duration::days(1));
    let _ = save_to_csv(all_articles.clone());

    // Print all articles
    println!("number of articles: {}", all_articles.len());
}


fn save_to_csv(articles: Vec<Article>) ->  Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path("articles_new.csv")?;


    // Write headers
    wtr.write_record(&["url", "url_mobile", "title", "seendate", "socialimage", "domain", "language", "sourcecountry"])?;

    // Write records
    for article in articles {
        wtr.write_record(&[&article.url, &article.url_mobile, &article.title, &article.seendate, &article.socialimage, &article.domain, &article.language, &article.sourcecountry])?;
    }

    // Flush the writer to ensure all data is written to the file
    wtr.flush()?;

    println!("Articles written to articles.csv");
    Ok(())
}

