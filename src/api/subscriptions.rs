use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::db::models::{Subscription, SubscriptionStatus, UserEntitlement};
use crate::error::{AppError, Result};

#[derive(Debug, Serialize)]
pub struct SubscriptionDetailResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub original_transaction_id: Option<String>,
    pub store_transaction_id: Option<String>,
    pub store: String,
    pub purchase_date: chrono::DateTime<chrono::Utc>,
    pub expires_date: Option<chrono::DateTime<chrono::Utc>>,
    pub cancellation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub renewal_grace_period_expires_date: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub auto_renew_status: Option<bool>,
    pub price_paid: Option<f64>,
    pub currency: Option<String>,
    pub is_trial: bool,
    pub is_intro_offer: bool,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionsResponse {
    pub subscriptions: Vec<SubscriptionDetailResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CancelSubscriptionRequest {
    pub cancellation_date: Option<chrono::DateTime<chrono::Utc>>,
}

// Get all subscriptions (with pagination)
pub async fn get_subscriptions(
    State(pool): State<SqlitePool>,
) -> Result<Json<SubscriptionsResponse>> {
    // In a real application, you'd implement pagination
    // For simplicity, we'll just limit to the first 100 subscriptions
    let subscriptions = sqlx::query_as::<_, Subscription>(
        r#"
        SELECT * FROM subscriptions
        ORDER BY purchase_date DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&pool)
    .await?;
    
    let subscription_responses = subscriptions
        .into_iter()
        .map(|subscription| SubscriptionDetailResponse {
            id: subscription.id,
            user_id: subscription.user_id,
            product_id: subscription.product_id,
            original_transaction_id: subscription.original_transaction_id,
            store_transaction_id: subscription.store_transaction_id,
            store: subscription.store,
            purchase_date: subscription.purchase_date,
            expires_date: subscription.expires_date,
            cancellation_date: subscription.cancellation_date,
            renewal_grace_period_expires_date: subscription.renewal_grace_period_expires_date,
            status: subscription.status,
            auto_renew_status: subscription.auto_renew_status,
            price_paid: subscription.price_paid,
            currency: subscription.currency,
            is_trial: subscription.is_trial,
            is_intro_offer: subscription.is_intro_offer,
        })
        .collect();
    
    Ok(Json(SubscriptionsResponse {
        subscriptions: subscription_responses,
    }))
}

// Get a specific subscription
pub async fn get_subscription(
    Path(subscription_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<SubscriptionDetailResponse>> {
    let subscription = Subscription::find_by_id(&subscription_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Subscription not found: {}", subscription_id)))?;
    
    Ok(Json(SubscriptionDetailResponse {
        id: subscription.id,
        user_id: subscription.user_id,
        product_id: subscription.product_id,
        original_transaction_id: subscription.original_transaction_id,
        store_transaction_id: subscription.store_transaction_id,
        store: subscription.store,
        purchase_date: subscription.purchase_date,
        expires_date: subscription.expires_date,
        cancellation_date: subscription.cancellation_date,
        renewal_grace_period_expires_date: subscription.renewal_grace_period_expires_date,
        status: subscription.status,
        auto_renew_status: subscription.auto_renew_status,
        price_paid: subscription.price_paid,
        currency: subscription.currency,
        is_trial: subscription.is_trial,
        is_intro_offer: subscription.is_intro_offer,
    }))
}

// Cancel a subscription
pub async fn cancel_subscription(
    Path(subscription_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(request): Json<CancelSubscriptionRequest>,
) -> Result<StatusCode> {
    let mut subscription = Subscription::find_by_id(&subscription_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Subscription not found: {}", subscription_id)))?;
    
    // Only active subscriptions can be canceled
    if subscription.status != SubscriptionStatus::Active.to_string() {
        return Err(AppError::BadRequest(format!(
            "Subscription is not active, current status: {}",
            subscription.status
        )));
    }
    
    // Use provided cancellation date or current time
    let cancellation_date = request.cancellation_date.unwrap_or_else(Utc::now);
    
    // Cancel the subscription
    subscription.cancel(cancellation_date, &pool).await?;
    
    // Note: We don't immediately revoke entitlements on cancellation
    // They remain active until the expiration date
    
    Ok(StatusCode::OK)
}

// Refund a subscription
pub async fn refund_subscription(
    Path(subscription_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    let mut subscription = Subscription::find_by_id(&subscription_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Subscription not found: {}", subscription_id)))?;
    
    // Update subscription status
    subscription.update_status(SubscriptionStatus::Refunded, &pool).await?;
    
    // Revoke user entitlements immediately
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        &pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.revoke(&pool).await?;
            }
        }
    }
    
    Ok(StatusCode::OK)
}
