use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub original_transaction_id: Option<String>,
    pub store_transaction_id: Option<String>,
    pub store: String,  // 'apple' or 'google'
    pub purchase_date: DateTime<Utc>,
    pub expires_date: Option<DateTime<Utc>>,
    pub cancellation_date: Option<DateTime<Utc>>,
    pub renewal_grace_period_expires_date: Option<DateTime<Utc>>,
    pub status: String,  // 'active', 'expired', 'cancelled', 'grace_period', etc.
    pub auto_renew_status: Option<bool>,
    pub price_paid: Option<f64>,
    pub currency: Option<String>,
    pub is_trial: bool,
    pub is_intro_offer: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
    GracePeriod,
    Refunded,
    Paused,
}

impl ToString for SubscriptionStatus {
    fn to_string(&self) -> String {
        match self {
            SubscriptionStatus::Active => "active".to_string(),
            SubscriptionStatus::Expired => "expired".to_string(),
            SubscriptionStatus::Cancelled => "cancelled".to_string(),
            SubscriptionStatus::GracePeriod => "grace_period".to_string(),
            SubscriptionStatus::Refunded => "refunded".to_string(),
            SubscriptionStatus::Paused => "paused".to_string(),
        }
    }
}

impl Subscription {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: String,
        product_id: String,
        original_transaction_id: Option<String>,
        store_transaction_id: Option<String>,
        store: String,
        purchase_date: DateTime<Utc>,
        expires_date: Option<DateTime<Utc>>,
        status: SubscriptionStatus,
        auto_renew_status: Option<bool>,
        price_paid: Option<f64>,
        currency: Option<String>,
        is_trial: bool,
        is_intro_offer: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            product_id,
            original_transaction_id,
            store_transaction_id,
            store,
            purchase_date,
            expires_date,
            cancellation_date: None,
            renewal_grace_period_expires_date: None,
            status: status.to_string(),
            auto_renew_status,
            price_paid,
            currency,
            is_trial,
            is_intro_offer,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub async fn create(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO subscriptions (
                id, user_id, product_id, original_transaction_id, store_transaction_id,
                store, purchase_date, expires_date, cancellation_date, 
                renewal_grace_period_expires_date, status, auto_renew_status,
                price_paid, currency, is_trial, is_intro_offer,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.user_id)
        .bind(&self.product_id)
        .bind(&self.original_transaction_id)
        .bind(&self.store_transaction_id)
        .bind(&self.store)
        .bind(&self.purchase_date)
        .bind(&self.expires_date)
        .bind(&self.cancellation_date)
        .bind(&self.renewal_grace_period_expires_date)
        .bind(&self.status)
        .bind(&self.auto_renew_status)
        .bind(&self.price_paid)
        .bind(&self.currency)
        .bind(&self.is_trial)
        .bind(&self.is_intro_offer)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: &str, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
        let subscription = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM subscriptions WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    pub async fn find_by_store_transaction(
        store: &str,
        transaction_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let subscription = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM subscriptions 
            WHERE store = ? AND (store_transaction_id = ? OR original_transaction_id = ?)
            "#,
        )
        .bind(store)
        .bind(transaction_id)
        .bind(transaction_id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    pub async fn find_active_by_user_and_product(
        user_id: &str,
        product_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let subscription = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = ? AND product_id = ? AND status = 'active'
            ORDER BY expires_date DESC 
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(product_id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    pub async fn list_by_user(user_id: &str, pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let subscriptions = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = ?
            ORDER BY purchase_date DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    pub async fn list_active_by_user(user_id: &str, pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let subscriptions = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = ? AND status = 'active'
            ORDER BY expires_date DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    pub async fn update_status(&mut self, status: SubscriptionStatus, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.status = status.to_string();
        self.updated_at = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.status)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn cancel(&mut self, cancellation_date: DateTime<Utc>, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.cancellation_date = Some(cancellation_date);
        self.status = SubscriptionStatus::Cancelled.to_string();
        self.auto_renew_status = Some(false);
        self.updated_at = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET cancellation_date = ?, status = ?, auto_renew_status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.cancellation_date)
        .bind(&self.status)
        .bind(&self.auto_renew_status)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_expiry(&mut self, expires_date: DateTime<Utc>, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.expires_date = Some(expires_date);
        self.updated_at = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET expires_date = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.expires_date)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_auto_renew_status(&mut self, auto_renew: bool, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.auto_renew_status = Some(auto_renew);
        self.updated_at = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET auto_renew_status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.auto_renew_status)
        .bind(&self.updated_at)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET user_id = ?, product_id = ?, original_transaction_id = ?, 
                store_transaction_id = ?, store = ?, purchase_date = ?,
                expires_date = ?, cancellation_date = ?, 
                renewal_grace_period_expires_date = ?, status = ?,
                auto_renew_status = ?, price_paid = ?, currency = ?,
                is_trial = ?, is_intro_offer = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&self.user_id)
        .bind(&self.product_id)
        .bind(&self.original_transaction_id)
        .bind(&self.store_transaction_id)
        .bind(&self.store)
        .bind(&self.purchase_date)
        .bind(&self.expires_date)
        .bind(&self.cancellation_date)
        .bind(&self.renewal_grace_period_expires_date)
        .bind(&self.status)
        .bind(&self.auto_renew_status)
        .bind(&self.price_paid)
        .bind(&self.currency)
        .bind(&self.is_trial)
        .bind(&self.is_intro_offer)
        .bind(Utc::now())
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
