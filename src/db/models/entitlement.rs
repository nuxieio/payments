use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entitlement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntitlement {
    pub id: String,
    pub user_id: String,
    pub entitlement_id: String,
    pub subscription_id: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entitlement {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub async fn create(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO entitlements (id, name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let entitlement = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM entitlements WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(entitlement)
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let entitlements = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM entitlements ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(entitlements)
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE entitlements
            SET name = ?, description = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.name)
        .bind(&self.description)
        .bind(Utc::now())
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM entitlements WHERE id = ?
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // Get all products that grant this entitlement
    pub async fn get_products(&self, pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let products = sqlx::query_scalar::<_, String>(
            r#"
            SELECT product_id FROM product_entitlements 
            WHERE entitlement_id = ?
            "#,
        )
        .bind(&self.id)
        .fetch_all(pool)
        .await?;

        Ok(products)
    }
}

impl UserEntitlement {
    pub fn new(
        user_id: String,
        entitlement_id: String,
        subscription_id: Option<String>,
        starts_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            entitlement_id,
            subscription_id,
            starts_at,
            expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub async fn create(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_entitlements (
                id, user_id, entitlement_id, subscription_id, 
                starts_at, expires_at, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.user_id)
        .bind(&self.entitlement_id)
        .bind(&self.subscription_id)
        .bind(&self.starts_at)
        .bind(&self.expires_at)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let user_entitlement = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM user_entitlements WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user_entitlement)
    }

    pub async fn find_active_for_user(
        user_id: &str, 
        entitlement_id: &str, 
        now: DateTime<Utc>,
        pool: &SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user_entitlement = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM user_entitlements 
            WHERE user_id = ? AND entitlement_id = ? 
              AND starts_at <= ?
              AND (expires_at IS NULL OR expires_at > ?)
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(entitlement_id)
        .bind(now)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        Ok(user_entitlement)
    }

    pub async fn list_active_for_user(
        user_id: &str,
        now: DateTime<Utc>,
        pool: &SqlitePool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let user_entitlements = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM user_entitlements 
            WHERE user_id = ? 
              AND starts_at <= ?
              AND (expires_at IS NULL OR expires_at > ?)
            "#,
        )
        .bind(user_id)
        .bind(now)
        .bind(now)
        .fetch_all(pool)
        .await?;

        Ok(user_entitlements)
    }

    pub async fn update_expiry(&mut self, expires_at: Option<DateTime<Utc>>, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE user_entitlements
            SET expires_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.expires_at)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn revoke(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        self.expires_at = Some(now);
        self.updated_at = now;
        
        sqlx::query(
            r#"
            UPDATE user_entitlements
            SET expires_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.expires_at)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_entitlements WHERE id = ?
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
