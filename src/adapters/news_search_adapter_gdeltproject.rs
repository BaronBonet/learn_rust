use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::core::domain::{ArticleCategory, ArticleQuery, NewsArticle};
use crate::core::ports::NewsSearchAdapter;
use isocountry::CountryCode;


pub struct GDeltaProjectNewsSearchAdapter {

}

impl NewsSearchAdapter for GDeltaProjectNewsSearchAdapter {
    fn query_for_articles(&self, query: ArticleQuery) -> Vec<NewsArticle> {
        let mut start_time = query.start_datetime;
        println!("Fetching articles between {} and {}...", query.start_datetime.format("%Y-%m-%d %H:%M:%S"), query.end_datetime.format("%Y-%m-%d %H:%M:%S"));

        let mut all_articles: Vec<NewsArticle> = vec![];

        while query.start_datetime > query.end_datetime {
            let resp: ureq::Response;
            match call_url(start_time, query.end_datetime, query.source_country) {
                Ok(response) => {
                    resp = response;
                },
                Err(err) => {
                    println!("Error calling URL: {}", err);
                    break;
                },
            }
            let articles: Vec<GDeltaArticle>;
            match extract_articles_from_response(resp) {
                Ok(ars) => {
                    articles = ars;
                },
                Err(err) => {
                    println!("Error extracting articles: {err}");
                    break;
                }
            }
            println!("Number of articles before match: {}", articles.len());
            let mut newsArticles = to_news_article(articles, &query.category);
            match newsArticles.len() {
                0 => {
                    println!("No articles found");
                    break;
                },
                1..=249 => {
                    // let a = &articles.len();
                    // println!("{} articles found, less than 250 so stopping", a);
                    // Convert to NewsArticle
                    all_articles.append(&mut newsArticles);
                    break;
                },
                250.. => {
                    // The date now needs to get converted back to string time...
                            start_time = newsArticles.last().unwrap().date;
                            println!("Latest article date: {}", start_time);
                    // TODO here it should go on a channel for persisting, but get the basics working 1st
                    all_articles.append(&mut newsArticles);
                }
                _ => {
                    println!("An unexpected length was returned");
                    break;
                }
            }}
        all_articles
    }
}

fn to_news_article(articles:Vec<GDeltaArticle>, category: &ArticleCategory) -> Vec<NewsArticle> {
   articles.iter().filter_map(|element|{
       let date = DateTime::parse_from_str(&element.seendate, "%Y%m%dT%H%M%SZ");
       let country = CountryCode::for_alpha2(&element.sourcecountry);
       if date.is_err() || country.is_ok () {
          return None
       }
       Some(
           NewsArticle{
               title: element.title.clone(),
               category:*category,
               date: date.unwrap().into(),
               url: element.url.clone(),
               domain: element.domain.clone(),
               language: element.language.clone(),
               country: CountryCode::FRA,
           }
       )
}).collect()
}

#[derive(Debug, Deserialize, Clone)]
struct GDeltaArticle {
    url: String,
    url_mobile: String,
    title: String,
    seendate: String,
    socialimage: String,
    domain: String,
    language: String,
    sourcecountry: String,
}


fn build_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>, source_country: CountryCode ) -> String {
    // https://blog.gdeltproject.org/gdelt-doc-2-0-api-debuts/
    let formatted_start_time = start_time.format("%Y%m%d%H%M%S").to_string();
    let formatted_end_time = end_time.format("%Y%m%d%H%M%S").to_string();

    let mut params = HashMap::new();
    params.insert("query", "sourcelang:french sourcecountry:FR");
    params.insert("mode", "artlist");
    params.insert("maxrecords", "250");
    params.insert("format", "json");
    params.insert("startdatetime", &formatted_end_time);
    params.insert("enddatetime", &formatted_start_time);

    // https://api.gdeltproject.org/api/v2/doc/doc?query=sourcecountry:FR%20AND%20(%22climate%20change%22%20OR%20%22global%20warming%22)&mode=artlist&maxrecords=250&startdatetime=20230617164918&enddatetime=20230618164918&sort=datedesc&format=json
    format!(
        "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&maxrecords={}&format={}&startdatetime={}&enddatetime={}&sort=datedesc",
        params["query"], params["mode"], params["maxrecords"], params["format"], params["startdatetime"], params["enddatetime"]
    )
}


fn extract_articles_from_response(response: ureq::Response) -> Result<Vec<GDeltaArticle>, Box<dyn Error>> {
    let response_string = match response.into_string(){
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error reading response body: {}", e)))),
        Ok(s) => s,
    };

    let body: Result<HashMap<String, Vec<GDeltaArticle>>, serde_json::Error> = serde_json::from_str(&response_string);

    match body {
        Ok(body) => Ok(body["articles"].clone()),
        Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Error parsing response body: {}", response_string)))),
    }
}


fn call_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>, source_country: CountryCode) -> Result<ureq::Response, Box<dyn Error>> {
    // Start time must be before end time
    let url = build_url(start_time, end_time, source_country);
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

fn extract_date(d: &String) -> Result<DateTime<Utc>, chrono::ParseError> {
    let date = NaiveDateTime::parse_from_str(&d, "%Y%m%dT%H%M%SZ")?;
    Ok(DateTime::from_utc(date, Utc))
}
