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

    let body: HashMap<String, Vec<Article>> = match serde_json::from_str(&response_string){
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error parsing response body: {}", e)))),
        Ok(s) => s,
    };

    Ok(body["articles"].clone())
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

fn fetch_articles_between(mut start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<Article> {
    let mut all_articles = vec![];

    while start_time > end_time {
        let mut articles: Vec<Article>;
        // let resp: ureq::Response;
        match call_url(start_time, end_time) {
            Ok(response) => {
                match extract_articles_from_response(response) {
                    Ok(ars) => {
                        // Append articles to all_articles
                        articles = ars;
                    },
                    Err(err) => {
                        println!("Error extracting articles: {}", err);
                        break;
                    }
                }
            },
            Err(err) => {
                println!("Error calling URL: {}", err);
                break;
            },
        }

        // get the last article in the vector which is the oldest
        // replace that timestamp with the start time.

        // Note what format are the times in?
        // 20230617T204500Z
        // 20230617T130000Z
        // So we can parse them with something like
        //    let date = NaiveDateTime::parse_from_str(&last_article.seendate, "%Y%m%dT%H%M%SZ")?;
        //     let parsed_date = DateTime::from_utc(date, Utc);

        break // Remove once this works
        // // Sleep for 5 seconds to avoid getting blocked
        // std::thread::sleep(std::time::Duration::from_secs(5));
    }
    all_articles
}


fn main() {
    let now: DateTime<Utc> = Utc::now();
    let all_articles = fetch_articles_between(now - chrono::Duration::days(0), now - chrono::Duration::days(1));
    // let _ = save_to_csv(all_articles.clone());

    // Print all articles
    println!("number of articles: {}", all_articles.len());
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

fn save_to_csv(articles: Vec<Article>) ->  Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path("articles.csv")?;


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

