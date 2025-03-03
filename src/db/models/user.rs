use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub app_user_id: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(app_user_id: String, email: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_user_id,
            email,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub async fn create(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, app_user_id, email, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.app_user_id)
        .bind(&self.email)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM users WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_app_user_id(app_user_id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM users WHERE app_user_id = ?
            "#,
        )
        .bind(app_user_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET app_user_id = ?, email = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.app_user_id)
        .bind(&self.email)
        .bind(Utc::now())
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM users WHERE id = ?
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
