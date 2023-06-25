mod adapters;
mod core;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
extern crate ureq;

use crate::adapters::news_search_adapter_gdeltproject::GDeltaProjectNewsSearchAdapter;
use crate::chrono::Utc;
use crate::core::domain::{ArticleCategory, ArticleQuery, NewsArticle};
use crate::core::ports::NewsSearchAdapter;
use csv::Writer;
use isocountry::CountryCode;
use std::error::Error;

fn main() {
    let g_delta_project_adapter = GDeltaProjectNewsSearchAdapter {};
    let end_datetime = Utc::now();
    let start_datetime = end_datetime - chrono::Duration::days(1);
    let q = ArticleQuery::new(
        CountryCode::FRA,
        ArticleCategory::ClimateChange,
        start_datetime,
        end_datetime,
    );
    let articles = g_delta_project_adapter.query_for_articles(q);
    let _ = save_to_csv(articles);
}
fn save_to_csv(articles: Vec<NewsArticle>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path("articles_new.csv")?;

    // Write headers
    wtr.write_record(&["url", "title", "date", "country", "language", "category"])?;

    // Write records
    for article in articles {
        wtr.write_record(&[
            &article.url,
            &article.title,
            &article.date.format("%Y%m%d%H%M%S").to_string(),
            &article.country.to_string(),
            &article.language,
            &article.category.to_string(),
        ])?;
    }

    // Flush the writer to ensure all data is written to the file
    wtr.flush()?;

    println!("Articles written to articles.csv");
    Ok(())
}
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR&mode=artlist&maxrecords=250&format=json&startdatetime=20230617125228&enddatetime=20230617133000
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR(%22climate%20change%22%20OR%20%22global%20warming%22)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR( climate change OR global warming)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
