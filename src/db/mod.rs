pub mod models;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::time::Duration;
use anyhow::Result;

pub async fn initialize_db(database_url: &str) -> Result<SqlitePool> {
    // Create the database file if it doesn't exist
    if !database_url.starts_with("sqlite::memory:") && !Path::new(database_url.trim_start_matches("sqlite:")).exists() {
        let dir_path = Path::new(database_url.trim_start_matches("sqlite:")).parent();
        if let Some(dir) = dir_path {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }
        std::fs::File::create(database_url.trim_start_matches("sqlite:"))?;
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Create tables if they don't exist
    let migration_query = include_str!("../../migrations/00001_initial_schema.sql");
    sqlx::query(migration_query)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn check_db_connection(pool: &SqlitePool) -> Result<bool> {
    let result = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    Ok(result == 1)
}
