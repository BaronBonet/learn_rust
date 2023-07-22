use crate::core::ports::NewsRepository;
use crate::core::{domain, ports};
use async_trait::async_trait;
use isocountry::CountryCode;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::error::Error;

pub struct PostgresNewsRepository {
    pool: PgPool,
    logger: Box<dyn ports::Logger>,
}

impl PostgresNewsRepository {
    pub fn new(pool: PgPool, logger: Box<dyn ports::Logger>) -> Self {
        Self { pool, logger }
    }
}

#[async_trait]
impl NewsRepository for PostgresNewsRepository {
    async fn get_articles_by_categories(
        &self,
        categories: Vec<String>,
        date_range: domain::DateRange,
    ) -> Result<Vec<domain::NewsArticle>, Box<dyn std::error::Error>> {
        let rows = sqlx::query(
            r#"
                SELECT news_articles.*, news_article_categories.category_name
                FROM news_articles
                JOIN news_article_categories ON news_articles.id = news_article_categories.news_article_id
                WHERE news_article_categories.category_name = ANY($1)
                AND news_articles.seen_at >= $2
                AND news_articles.seen_at <= $3
                "#,
        )
        .bind(&categories)
        .bind(date_range.inclusive_start_date)
        .bind(date_range.inclusive_end_date)
        .fetch_all(&self.pool)
        .await?;

        // TODO make this into a function and add a unit test
        let articles = rows
            .into_iter()
            .map(|row| {
                let country_str: String = row.get("country_iso_alpha_3");
                // TODO error handling for country code
                let country_code = CountryCode::for_alpha3(&country_str).unwrap();

                domain::NewsArticle {
                    title: row.get("title"),
                    category: row.get("category_name"),
                    domain: row.get("domain"),
                    country: country_code,
                    url: row.get("url"),
                    language: row.get("language"),
                    datetime: row.get("seen_at"),
                }
            })
            .collect();

        Ok(articles)
    }

    async fn store_articles(
        &self,
        articles: Vec<domain::NewsArticle>,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let mut tx = self.pool.begin().await?;
        let mut num_inserted = 0;

        for article in &articles {
            match insert_article(&mut tx, article).await {
                Ok(id) => {
                    // If there was no error then attempt to add the category to the article
                    sqlx::query(
                        "INSERT INTO news_article_categories (news_article_id, category_name) 
                    VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(&article.category)
                    .execute(&mut tx)
                    .await?;
                    num_inserted += 1;
                }
                Err(e) => {
                    self.logger.error(
                        format!(
                            "Error inserting article: \n error: {} \n article: {}",
                            e, article.title
                        )
                        .as_str(),
                    );
                }
            }
        }
        self.logger
            .debug(format!("Attempting to insert {} articles", num_inserted).as_str());

        tx.commit().await?;

        Ok(num_inserted)
    }

    async fn add_category(&self, category: String) -> Result<bool, Box<dyn std::error::Error>> {
        let result =
            sqlx::query("INSERT INTO categories (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
                .bind(&category)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn is_valid_category(
        &self,
        category: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories WHERE name = $1")
            .bind(&category)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0 > 0)
    }

    async fn get_categories(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let rows = sqlx::query("SELECT name FROM categories")
            .fetch_all(&self.pool)
            .await?;
        let categories = rows.into_iter().map(|row| row.get("name")).collect();
        Ok(categories)
    }

    async fn get_countries(&self) -> Result<Vec<CountryCode>, Box<dyn Error>> {
        let rows = sqlx::query("SELECT iso_alpha_3 from countries")
            .fetch_all(&self.pool)
            .await?;
        let countries = rows
            .into_iter()
            .filter_map(|row| get_country_code(&row, "iso_alpha_3").ok())
            .collect();
        Ok(countries)
    }
}

async fn insert_article(
    tx: &mut Transaction<'_, Postgres>,
    article: &domain::NewsArticle,
) -> Result<i32, sqlx::Error> {
    match sqlx::query_as(
        "INSERT INTO news_articles (title, domain, country, seen_at, url, language) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        ON CONFLICT (title, domain, country, seen_at) DO NOTHING RETURNING id",
    )
    .bind(&article.title)
    .bind(&article.domain)
    .bind(article.country.alpha3())
    .bind(&article.datetime)
    .bind(&article.url)
    .bind(&article.language)
    .fetch_optional(&mut *tx)
    .await?
    {
        Some((id,)) => Ok(id),
        None => {
            // Return the id of the existing article
            let id: (i32,) = sqlx::query_as(
                "SELECT id FROM news_articles
                WHERE title = $1 AND domain = $2 AND country = $3 AND seen_at = $4",
            )
            .bind(&article.title)
            .bind(&article.domain)
            .bind(article.country.alpha3())
            .bind(&article.datetime)
            .fetch_one(&mut *tx)
            .await?;
            Ok(id.0)
        }
    }
}

fn get_country_code(row: &PgRow, field_name: &str) -> Result<CountryCode, Box<dyn Error>> {
    let country_str: String = row.get(field_name);
    CountryCode::for_alpha3(&country_str).map_err(|_| Box::new(CountryCodeError) as Box<dyn Error>)
}

#[derive(Debug)]
struct CountryCodeError;

impl std::fmt::Display for CountryCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid country code")
    }
}

impl std::error::Error for CountryCodeError {}
