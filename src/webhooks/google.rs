use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::db::models::{
    User, Product, Subscription, SubscriptionStatus, UserEntitlement,
};
use crate::error::{AppError, Result};

// Google Play Real-time Developer Notifications (RTDN)
// https://developer.android.com/google/play/billing/rtdn

#[derive(Debug, Deserialize)]
pub struct GoogleNotificationPayload {
    version: String,
    #[serde(rename = "packageName")]
    package_name: String,
    #[serde(rename = "eventTimeMillis")]
    event_time_millis: i64,
    #[serde(rename = "subscriptionNotification")]
    subscription_notification: Option<GoogleSubscriptionNotification>,
    #[serde(rename = "oneTimeProductNotification")]
    one_time_product_notification: Option<GoogleOneTimeProductNotification>,
    #[serde(rename = "testNotification")]
    test_notification: Option<GoogleTestNotification>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleSubscriptionNotification {
    version: String,
    #[serde(rename = "notificationType")]
    notification_type: i32,
    #[serde(rename = "purchaseToken")]
    purchase_token: String,
    #[serde(rename = "subscriptionId")]
    subscription_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleOneTimeProductNotification {
    version: String,
    #[serde(rename = "notificationType")]
    notification_type: i32,
    #[serde(rename = "purchaseToken")]
    purchase_token: String,
    #[serde(rename = "sku")]
    sku: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleTestNotification {
    version: String,
}

// Google API response types for subscription details
#[derive(Debug, Deserialize)]
pub struct GoogleSubscriptionPurchase {
    #[serde(rename = "kind")]
    kind: String,
    #[serde(rename = "startTimeMillis")]
    start_time_millis: i64,
    #[serde(rename = "expiryTimeMillis")]
    expiry_time_millis: i64,
    #[serde(rename = "autoRenewing")]
    auto_renewing: bool,
    #[serde(rename = "priceCurrencyCode")]
    price_currency_code: Option<String>,
    #[serde(rename = "priceAmountMicros")]
    price_amount_micros: Option<i64>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    #[serde(rename = "developerPayload")]
    developer_payload: Option<String>,
    #[serde(rename = "paymentState")]
    payment_state: Option<i32>,
    #[serde(rename = "cancelReason")]
    cancel_reason: Option<i32>,
    #[serde(rename = "userCancellationTimeMillis")]
    user_cancellation_time_millis: Option<i64>,
    #[serde(rename = "orderId")]
    order_id: Option<String>,
    #[serde(rename = "purchaseType")]
    purchase_type: Option<i32>,
    #[serde(rename = "acknowledgementState")]
    acknowledgement_state: Option<i32>,
    #[serde(rename = "obfuscatedExternalAccountId")]
    obfuscated_external_account_id: Option<String>,
    #[serde(rename = "linkedPurchaseToken")]
    linked_purchase_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    message: String,
}

pub async fn handle_google_webhook(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(payload): Json<GoogleNotificationPayload>,
) -> Result<(StatusCode, Json<WebhookResponse>)> {
    // In a real implementation, verify the webhook signature
    // For now, we'll just process the notification

    // Check if this is a test notification
    if payload.test_notification.is_some() {
        return Ok((
            StatusCode::OK,
            Json(WebhookResponse {
                message: "Test notification received".to_string(),
            }),
        ));
    }

    // Process subscription notifications
    if let Some(subscription_notification) = &payload.subscription_notification {
        process_subscription_notification(
            &payload.package_name,
            subscription_notification,
            &pool,
        )
        .await?;
    }

    // Process one-time product notifications
    if let Some(one_time_notification) = &payload.one_time_product_notification {
        process_one_time_notification(
            &payload.package_name,
            one_time_notification,
            &pool,
        )
        .await?;
    }

    // Return success response
    Ok((
        StatusCode::OK,
        Json(WebhookResponse {
            message: "Webhook processed successfully".to_string(),
        }),
    ))
}

async fn process_subscription_notification(
    package_name: &str,
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    // Google subscription notification types
    // 1: SUBSCRIPTION_RECOVERED - A subscription was recovered from account hold.
    // 2: SUBSCRIPTION_RENEWED - An active subscription was renewed.
    // 3: SUBSCRIPTION_CANCELED - A subscription was canceled.
    // 4: SUBSCRIPTION_PURCHASED - A new subscription was purchased.
    // 5: SUBSCRIPTION_ON_HOLD - A subscription has entered account hold (if enabled).
    // 6: SUBSCRIPTION_IN_GRACE_PERIOD - A subscription has entered grace period (if enabled).
    // 7: SUBSCRIPTION_RESTARTED - A subscription was restarted.
    // 8: SUBSCRIPTION_PRICE_CHANGE_CONFIRMED - A subscription price change was confirmed.
    // 9: SUBSCRIPTION_DEFERRED - A subscription renewal was deferred.
    // 10: SUBSCRIPTION_PAUSED - A subscription was paused.
    // 11: SUBSCRIPTION_PAUSE_SCHEDULE_CHANGED - A subscription pause schedule was changed.
    // 12: SUBSCRIPTION_REVOKED - A subscription was revoked.
    // 13: SUBSCRIPTION_EXPIRED - A subscription expired.

    // In a real implementation, query the Google Play Developer API to get purchase details
    // For this example, we'll use mock data based on notification type

    match notification.notification_type {
        1 => process_subscription_recovered(notification, pool).await?,
        2 => process_subscription_renewed(notification, pool).await?,
        3 => process_subscription_canceled(notification, pool).await?,
        4 => process_subscription_purchased(notification, pool).await?,
        5 => process_subscription_on_hold(notification, pool).await?,
        6 => process_subscription_in_grace_period(notification, pool).await?,
        7 => process_subscription_restarted(notification, pool).await?,
        12 => process_subscription_revoked(notification, pool).await?,
        13 => process_subscription_expired(notification, pool).await?,
        _ => {
            // Other notification types can be handled as needed
            // For now, we'll just log them
        }
    }

    Ok(())
}

async fn process_one_time_notification(
    package_name: &str,
    notification: &GoogleOneTimeProductNotification,
    pool: &SqlitePool,
) -> Result<()> {
    // Google one-time product notification types
    // 1: PURCHASED - A one-time product was purchased.
    // 2: CANCELED - A one-time product was canceled.

    match notification.notification_type {
        1 => process_one_time_purchased(notification, pool).await?,
        2 => process_one_time_canceled(notification, pool).await?,
        _ => {
            // Unknown notification type
            return Err(AppError::BadRequest(format!(
                "Unknown one-time notification type: {}",
                notification.notification_type
            )));
        }
    }

    Ok(())
}

// Implementing each notification type handling function
// For brevity, we'll just implement a few key ones with mock data

async fn process_subscription_purchased(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    // Mock purchase details that would come from Google API
    let purchase_token = &notification.purchase_token;
    let google_product_id = &notification.subscription_id;
    let order_id = "GPA.1234-5678-9012-34567"; // This would come from the Google API
    let purchase_time = Utc::now();
    let expiry_time = Utc::now() + chrono::Duration::days(30); // 30 days subscription
    let auto_renewing = true;
    
    // In a real app, you'd also have a way to map the purchase to a user
    // For this example, we'll create a dummy user if needed
    let user_id = {
        // Check if we have a user associated with this purchase token
        // In a real app, you'd have a better way to do this
        let user = User::find_by_app_user_id(purchase_token, pool).await?;
        
        match user {
            Some(user) => user.id,
            None => {
                // Create a new user
                let new_user = User::new(purchase_token.to_string(), None);
                new_user.create(pool).await?;
                new_user.id
            }
        }
    };
    
    // Find the product by Google product ID
    let product = Product::find_by_store_product_id("google", google_product_id, pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", google_product_id)))?;
    
    // Create a new subscription
    let subscription = Subscription::new(
        user_id.clone(),
        product.id.clone(),
        Some(purchase_token.to_string()), // Use purchase token as original transaction ID
        Some(order_id.to_string()),
        "google".to_string(),
        purchase_time,
        Some(expiry_time),
        SubscriptionStatus::Active,
        Some(auto_renewing),
        None, // Price paid (not available in this mock)
        None, // Currency (not available in this mock)
        false, // Is trial
        false, // Is intro offer
    );
    
    subscription.create(pool).await?;
    
    // Get the entitlements for this product
    let entitlement_ids = product.get_entitlements(pool).await?;
    
    // Grant entitlements to the user
    for entitlement_id in entitlement_ids {
        let user_entitlement = UserEntitlement::new(
            user_id.clone(),
            entitlement_id,
            Some(subscription.id.clone()),
            purchase_time,
            Some(expiry_time),
        );
        
        user_entitlement.create(pool).await?;
    }
    
    Ok(())
}

async fn process_subscription_renewed(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    // Mock purchase details that would come from Google API
    let purchase_token = &notification.purchase_token;
    let new_expiry_time = Utc::now() + chrono::Duration::days(30); // 30 more days
    
    // Find the subscription by purchase token (which we used as original_transaction_id)
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription details
    subscription.expires_date = Some(new_expiry_time);
    subscription.status = SubscriptionStatus::Active.to_string();
    subscription.update(pool).await?;
    
    // Update user entitlements
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.update_expiry(Some(new_expiry_time), pool).await?;
            }
        }
    }
    
    Ok(())
}

async fn process_subscription_canceled(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status
    subscription.cancel(Utc::now(), pool).await?;
    
    // Note: We don't immediately revoke entitlements when canceled
    // They should remain active until the expiration date
    
    Ok(())
}

async fn process_subscription_expired(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status
    subscription.update_status(SubscriptionStatus::Expired, pool).await?;
    
    // Expire user entitlements
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.update_expiry(Some(Utc::now()), pool).await?;
            }
        }
    }
    
    Ok(())
}

async fn process_subscription_in_grace_period(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    let grace_period_end = Utc::now() + chrono::Duration::days(16); // 16 days grace period
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status to grace period
    subscription.status = SubscriptionStatus::GracePeriod.to_string();
    subscription.renewal_grace_period_expires_date = Some(grace_period_end);
    subscription.update(pool).await?;
    
    // Note: Entitlements remain active during grace period
    
    Ok(())
}

async fn process_subscription_recovered(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    let new_expiry_time = Utc::now() + chrono::Duration::days(30); // 30 more days
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription details
    subscription.expires_date = Some(new_expiry_time);
    subscription.status = SubscriptionStatus::Active.to_string();
    subscription.renewal_grace_period_expires_date = None; // Clear grace period
    subscription.update(pool).await?;
    
    // Update user entitlements
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.update_expiry(Some(new_expiry_time), pool).await?;
            }
        }
    }
    
    Ok(())
}

async fn process_subscription_on_hold(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status to paused
    subscription.status = SubscriptionStatus::Paused.to_string();
    subscription.update(pool).await?;
    
    // Expire user entitlements
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.update_expiry(Some(Utc::now()), pool).await?;
            }
        }
    }
    
    Ok(())
}

async fn process_subscription_restarted(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    let new_expiry_time = Utc::now() + chrono::Duration::days(30); // 30 more days
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription details
    subscription.expires_date = Some(new_expiry_time);
    subscription.status = SubscriptionStatus::Active.to_string();
    subscription.update(pool).await?;
    
    // Grant entitlements again
    let product = Product::find_by_id(&subscription.product_id, pool).await?
        .ok_or_else(|| AppError::NotFound(
            format!("Product not found: {}", subscription.product_id)
        ))?;
    
    let entitlement_ids = product.get_entitlements(pool).await?;
    
    for entitlement_id in entitlement_ids {
        let user_entitlement = UserEntitlement::new(
            subscription.user_id.clone(),
            entitlement_id,
            Some(subscription.id.clone()),
            Utc::now(),
            Some(new_expiry_time),
        );
        
        user_entitlement.create(pool).await?;
    }
    
    Ok(())
}

async fn process_subscription_revoked(
    notification: &GoogleSubscriptionNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Subscription not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status
    subscription.update_status(SubscriptionStatus::Refunded, pool).await?;
    
    // Revoke user entitlements immediately
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.revoke(pool).await?;
            }
        }
    }
    
    Ok(())
}

async fn process_one_time_purchased(
    notification: &GoogleOneTimeProductNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    let google_product_id = &notification.sku;
    let order_id = "GPA.9876-5432-1098-76543"; // This would come from the Google API
    let purchase_time = Utc::now();
    
    // Find or create user
    let user_id = {
        // Check if we have a user associated with this purchase token
        let user = User::find_by_app_user_id(purchase_token, pool).await?;
        
        match user {
            Some(user) => user.id,
            None => {
                // Create a new user
                let new_user = User::new(purchase_token.to_string(), None);
                new_user.create(pool).await?;
                new_user.id
            }
        }
    };
    
    // Find the product by Google product ID
    let product = Product::find_by_store_product_id("google", google_product_id, pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", google_product_id)))?;
    
    // Create a non-renewing subscription (one-time purchase)
    let subscription = Subscription::new(
        user_id.clone(),
        product.id.clone(),
        Some(purchase_token.to_string()),
        Some(order_id.to_string()),
        "google".to_string(),
        purchase_time,
        None, // No expiration for one-time purchases
        SubscriptionStatus::Active,
        Some(false), // Not auto-renewing
        None,        // Price paid (not available in this mock)
        None,        // Currency (not available in this mock)
        false,       // Is trial
        false,       // Is intro offer
    );
    
    subscription.create(pool).await?;
    
    // Get the entitlements for this product
    let entitlement_ids = product.get_entitlements(pool).await?;
    
    // Grant lifetime entitlements to the user
    for entitlement_id in entitlement_ids {
        let user_entitlement = UserEntitlement::new(
            user_id.clone(),
            entitlement_id,
            Some(subscription.id.clone()),
            purchase_time,
            None, // No expiration (lifetime)
        );
        
        user_entitlement.create(pool).await?;
    }
    
    Ok(())
}

async fn process_one_time_canceled(
    notification: &GoogleOneTimeProductNotification,
    pool: &SqlitePool,
) -> Result<()> {
    let purchase_token = &notification.purchase_token;
    
    // Find the subscription by purchase token
    let mut subscription = Subscription::find_by_store_transaction(
        "google", 
        purchase_token, 
        pool
    )
    .await?
    .ok_or_else(|| AppError::NotFound(
        format!("Purchase not found for token: {}", purchase_token)
    ))?;
    
    // Update subscription status
    subscription.update_status(SubscriptionStatus::Refunded, pool).await?;
    
    // Revoke user entitlements
    let user_entitlements = UserEntitlement::list_active_for_user(
        &subscription.user_id, 
        Utc::now(), 
        pool
    ).await?;
    
    for mut entitlement in user_entitlements {
        if let Some(sub_id) = &entitlement.subscription_id {
            if sub_id == &subscription.id {
                entitlement.revoke(pool).await?;
            }
        }
    }
    
    Ok(())
}
