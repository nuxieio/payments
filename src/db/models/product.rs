use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub apple_product_id: Option<String>,
    pub google_product_id: Option<String>,
    pub type_: String,  // 'subscription' or 'one_time'
    pub price_usd: Option<f64>,
    pub duration_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProductType {
    Subscription,
    OneTime,
}

impl ToString for ProductType {
    fn to_string(&self) -> String {
        match self {
            ProductType::Subscription => "subscription".to_string(),
            ProductType::OneTime => "one_time".to_string(),
        }
    }
}

impl Product {
    pub fn new(
        name: String,
        description: Option<String>,
        apple_product_id: Option<String>,
        google_product_id: Option<String>,
        type_: ProductType,
        price_usd: Option<f64>,
        duration_days: Option<i32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            apple_product_id,
            google_product_id,
            type_: type_.to_string(),
            price_usd,
            duration_days,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub async fn create(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO products (
                id, name, description, apple_product_id, google_product_id, 
                type, price_usd, duration_days, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.apple_product_id)
        .bind(&self.google_product_id)
        .bind(&self.type_)
        .bind(&self.price_usd)
        .bind(&self.duration_days)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let product = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM products WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(product)
    }

    pub async fn find_by_store_product_id(
        store: &str,
        store_product_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let query = match store {
            "apple" => "SELECT * FROM products WHERE apple_product_id = ?",
            "google" => "SELECT * FROM products WHERE google_product_id = ?",
            _ => return Err(sqlx::Error::RowNotFound),
        };

        let product = sqlx::query_as::<_, Self>(query)
            .bind(store_product_id)
            .fetch_optional(pool)
            .await?;

        Ok(product)
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let products = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM products ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(products)
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE products
            SET name = ?, description = ?, apple_product_id = ?, google_product_id = ?,
                type = ?, price_usd = ?, duration_days = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.apple_product_id)
        .bind(&self.google_product_id)
        .bind(&self.type_)
        .bind(&self.price_usd)
        .bind(&self.duration_days)
        .bind(Utc::now())
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM products WHERE id = ?
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // Add or update entitlement mapping
    pub async fn add_entitlement(&self, entitlement_id: &str, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO product_entitlements (product_id, entitlement_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(entitlement_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(())
    }

    // Remove entitlement mapping
    pub async fn remove_entitlement(&self, entitlement_id: &str, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM product_entitlements 
            WHERE product_id = ? AND entitlement_id = ?
            "#,
        )
        .bind(&self.id)
        .bind(entitlement_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // Get all entitlements for this product
    pub async fn get_entitlements(&self, pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let entitlements = sqlx::query_scalar::<_, String>(
            r#"
            SELECT entitlement_id FROM product_entitlements 
            WHERE product_id = ?
            "#,
        )
        .bind(&self.id)
        .fetch_all(pool)
        .await?;

        Ok(entitlements)
    }
}
