mod adapters;
mod core;
mod infrastructure;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
extern crate ureq;
use std::env;

use crate::adapters::news_search_client_gdeltproject::GDeltaProjectNewsSearchAdapter;
use crate::chrono::Utc;
use crate::core::domain::{ArticleQuery, NewsArticle};
use crate::core::ports::{NewsRepository, NewsSearchClient};
use csv::Writer;
use std::error::Error;

#[tokio::main]

async fn main() {
    // in main.rs
    let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| String::from("postgres"));
    let db_password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| String::from("postgres"));
    let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| String::from("postgres"));
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| String::from("localhost"));
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| String::from("5432"));

    let pool =
        infrastructure::postgres::get_db_pool(db_user, db_password, db_name, db_host, db_port)
            .await
            .expect("Failed to connect to Postgres");

    let repo = adapters::news_repository_postgres::PostgresNewsRepository::new(pool);
    let added = repo
        .add_category("ClimateChange".to_string())
        .await
        .expect("Failed to add category");
    println!("Added category: {}", added);

    //     let g_delta_project_adapter = GDeltaProjectNewsSearchAdapter {};
    //     let end_datetime = Utc::now();
    //     let start_datetime = end_datetime - chrono::Duration::days(1);
    //     let q = ArticleQuery::new(
    //         CountryCode::FRA,
    //         ArticleCategory::ClimateChange,
    //         start_datetime,
    //         end_datetime,
    //     );
    //     let articles = g_delta_project_adapter.query_for_articles(q);
    //     let _ = save_to_csv(articles);
    // }
    // fn save_to_csv(articles: Vec<NewsArticle>) -> Result<(), Box<dyn Error>> {
    //     let mut wtr = Writer::from_path("articles_new.csv")?;
    //
    //     // Write headers
    //     wtr.write_record(&["url", "title", "date", "country", "language", "category"])?;
    //
    //     // Write records
    //     for article in articles {
    //         wtr.write_record(&[
    //             &article.url,
    //             &article.title,
    //             &article.date.format("%Y%m%d%H%M%S").to_string(),
    //             &article.country.to_string(),
    //             &article.language,
    //             &article.category.to_string(),
    //         ])?;
    //     }
    //
    //     // Flush the writer to ensure all data is written to the file
    //     wtr.flush()?;
    //
    //     println!("Articles written to articles.csv");
    // Ok(())
}
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR&mode=artlist&maxrecords=250&format=json&startdatetime=20230617125228&enddatetime=20230617133000
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR(%22climate%20change%22%20OR%20%22global%20warming%22)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR( climate change OR global warming)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
