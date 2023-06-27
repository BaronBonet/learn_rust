mod adapters;
mod core;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
extern crate ureq;
use sqlx::types::chrono::NaiveDateTime;

use crate::adapters::news_search_adapter_gdeltproject::GDeltaProjectNewsSearchAdapter;
use crate::chrono::Utc;
use crate::core::domain::{ArticleQuery, NewsArticle};
use crate::core::ports::NewsSearchClient;
use csv::Writer;
use isocountry::CountryCode;
use std::error::Error;

use sqlx::{PgPool, Row};
// use anyhow::Result;
// use chrono::NaiveDateTime; // remember to add "chrono" as a dependency in your Cargo.toml

async fn get_articles_with_category(
    pool: &PgPool,
    category: &str,
) -> Result<Vec<NewsArticle>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT news_articles.*, news_article_categories.category_name
        FROM news_articles
        JOIN news_article_categories ON news_articles.id = news_article_categories.news_article_id
        WHERE news_article_categories.category_name = $1
        "#,
    )
    .bind(category)
    .fetch_all(pool)
    .await?;

    let articles = rows
        .into_iter()
        .map(|row| {
            let country_str: String = row.get("country");
            // TODO error handling for country code
            let country_code = CountryCode::for_alpha3(&country_str).unwrap();

            NewsArticle {
                title: row.get("title"),
                category: row.get("category_name"),
                domain: row.get("domain"),
                country: country_code,
                url: row.get("url"),
                language: row.get("language"),
                date: row.get("seen_at"),
            }
        })
        .collect();

    Ok(articles)
}

#[tokio::main]

async fn main() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:15432/postgres")
        .await
        .expect("Failed to connect to Postgres");
    match get_articles_with_category(&pool, "climate change").await {
        Ok(articles) => {
            for article in articles {
                println!("{:?}", article);
            }
        }
        Err(e) => {
            eprintln!("Failed to get articles: {}", e);
        }
    };

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
