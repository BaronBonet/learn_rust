use sqlx::postgres::PgPool;

pub async fn get_db_pool(
    db_user: String,
    db_password: String,
    db_name: String,
    db_host: String,
    db_port: String,
) -> Result<PgPool, sqlx::Error> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );

    PgPool::connect(&database_url).await
}
