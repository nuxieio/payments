use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::db::models::{
    User, Product, Subscription, SubscriptionStatus, UserEntitlement,
};
use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
pub struct AppleNotificationPayload {
    #[serde(rename = "notificationType")]
    notification_type: String,
    #[serde(rename = "subtype")]
    sub_type: Option<String>,
    #[serde(rename = "notificationUUID")]
    notification_uuid: String,
    #[serde(rename = "notificationVersion")]
    version: String,
    data: AppleNotificationData,
    #[serde(rename = "signedDate")]
    signed_date: i64, // Unix timestamp in milliseconds
}

#[derive(Debug, Deserialize)]
pub struct AppleNotificationData {
    #[serde(rename = "appAppleId")]
    app_apple_id: Option<String>,
    #[serde(rename = "bundleId")]
    bundle_id: Option<String>,
    #[serde(rename = "bundleVersion")]
    bundle_version: Option<String>,
    environment: Option<String>,
    #[serde(rename = "signedRenewalInfo")]
    signed_renewal_info: Option<String>,
    #[serde(rename = "signedTransactionInfo")]
    signed_transaction_info: Option<String>,
}

// Decoded transaction info after JWT validation
#[derive(Debug, Deserialize)]
pub struct AppleTransactionInfo {
    #[serde(rename = "transactionId")]
    transaction_id: String,
    #[serde(rename = "originalTransactionId")]
    original_transaction_id: String,
    #[serde(rename = "webOrderLineItemId")]
    web_order_line_item_id: Option<String>,
    #[serde(rename = "bundleId")]
    bundle_id: String,
    #[serde(rename = "productId")]
    product_id: String,
    #[serde(rename = "subscriptionGroupIdentifier")]
    subscription_group_identifier: Option<String>,
    #[serde(rename = "purchaseDate")]
    purchase_date: i64, // Unix timestamp in milliseconds
    #[serde(rename = "originalPurchaseDate")]
    original_purchase_date: i64, // Unix timestamp in milliseconds
    #[serde(rename = "expiresDate")]
    expires_date: Option<i64>, // Unix timestamp in milliseconds
    #[serde(rename = "quantity")]
    quantity: i64,
    #[serde(rename = "type")]
    transaction_type: String,
    #[serde(rename = "inAppOwnershipType")]
    in_app_ownership_type: String,
    #[serde(rename = "signedDate")]
    signed_date: i64, // Unix timestamp in milliseconds
    #[serde(rename = "appAccountToken")]
    app_account_token: Option<String>, // This can be used to identify the user
    #[serde(rename = "revocationDate")]
    revocation_date: Option<i64>, // Unix timestamp in milliseconds
    #[serde(rename = "revocationReason")]
    revocation_reason: Option<i64>,
    #[serde(rename = "offerType")]
    offer_type: Option<i64>,
    #[serde(rename = "offerIdentifier")]
    offer_identifier: Option<String>,
}

// Decoded renewal info after JWT validation
#[derive(Debug, Deserialize)]
pub struct AppleRenewalInfo {
    #[serde(rename = "autoRenewProductId")]
    auto_renew_product_id: Option<String>,
    #[serde(rename = "autoRenewStatus")]
    auto_renew_status: i32, // 1 = on, 0 = off
    #[serde(rename = "expirationIntent")]
    expiration_intent: Option<i32>,
    #[serde(rename = "gracePeriodExpiresDate")]
    grace_period_expires_date: Option<i64>, // Unix timestamp in milliseconds
    #[serde(rename = "isInBillingRetryPeriod")]
    is_in_billing_retry_period: Option<i32>, // 1 = yes, 0 = no
    #[serde(rename = "offerIdentifier")]
    offer_identifier: Option<String>,
    #[serde(rename = "offerType")]
    offer_type: Option<i32>,
    #[serde(rename = "originalTransactionId")]
    original_transaction_id: String,
    #[serde(rename = "priceIncreaseStatus")]
    price_increase_status: Option<i32>,
    #[serde(rename = "productId")]
    product_id: String,
    #[serde(rename = "signedDate")]
    signed_date: i64, // Unix timestamp in milliseconds
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    message: String,
}

pub async fn handle_apple_webhook(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(payload): Json<AppleNotificationPayload>,
) -> Result<(StatusCode, Json<WebhookResponse>)> {
    // In a real implementation, verify the webhook signature
    // For now, we'll just process the notification

    // Process based on notification type
    match payload.notification_type.as_str() {
        "CONSUMPTION_REQUEST" => {
            // Handle consumption request (e.g., check entitlement)
            // This is typically used for consumable products
        }
        "DID_CHANGE_RENEWAL_PREF" => {
            // Handle subscription renewal preference change
            process_renewal_change(&payload, &pool).await?;
        }
        "DID_CHANGE_RENEWAL_STATUS" => {
            // Handle subscription renewal status change
            process_renewal_status_change(&payload, &pool).await?;
        }
        "DID_FAIL_TO_RENEW" => {
            // Handle subscription renewal failure
            process_renewal_failure(&payload, &pool).await?;
        }
        "DID_RENEW" => {
            // Handle subscription renewal
            process_subscription_renewal(&payload, &pool).await?;
        }
        "EXPIRED" => {
            // Handle subscription expiration
            process_subscription_expiration(&payload, &pool).await?;
        }
        "GRACE_PERIOD_EXPIRED" => {
            // Handle grace period expiration
            process_grace_period_expiration(&payload, &pool).await?;
        }
        "OFFER_REDEEMED" => {
            // Handle offer redemption
            process_offer_redemption(&payload, &pool).await?;
        }
        "PRICE_INCREASE" => {
            // Handle price increase
            process_price_increase(&payload, &pool).await?;
        }
        "REFUND" => {
            // Handle refund
            process_refund(&payload, &pool).await?;
        }
        "REFUND_DECLINED" => {
            // Handle refund decline
            process_refund_declined(&payload, &pool).await?;
        }
        "RENEWAL_EXTENDED" => {
            // Handle renewal extension
            process_renewal_extension(&payload, &pool).await?;
        }
        "REVOKE" => {
            // Handle subscription revocation
            process_subscription_revocation(&payload, &pool).await?;
        }
        "SUBSCRIBED" => {
            // Handle new subscription
            process_new_subscription(&payload, &pool).await?;
        }
        _ => {
            // Unknown notification type
            return Err(AppError::BadRequest(format!(
                "Unknown notification type: {}",
                payload.notification_type
            )));
        }
    }

    // Return success response
    Ok((
        StatusCode::OK,
        Json(WebhookResponse {
            message: "Webhook processed successfully".to_string(),
        }),
    ))
}

// Helper function to decode and verify the transaction info JWT
async fn decode_transaction_info(signed_transaction_info: &str) -> Result<AppleTransactionInfo> {
    // In a real implementation, decode and verify the JWT signature
    // For now, we'll just pretend to do that and return mock data
    // You would use the jsonwebtoken crate for this

    Err(AppError::InternalServerError(
        "JWT decoding not implemented in this example".to_string(),
    ))
}

// Helper function to decode and verify the renewal info JWT
async fn decode_renewal_info(signed_renewal_info: &str) -> Result<AppleRenewalInfo> {
    // In a real implementation, decode and verify the JWT signature
    // For now, we'll just pretend to do that and return mock data
    // You would use the jsonwebtoken crate for this

    Err(AppError::InternalServerError(
        "JWT decoding not implemented in this example".to_string(),
    ))
}

// Process a new subscription
async fn process_new_subscription(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // In a real implementation, we would decode the JWT token
        // For demo purposes, let's assume we have the decoded data
        
        // Mock the decoded data
        let transaction_id = "mock_transaction_id";
        let original_transaction_id = "mock_original_transaction_id";
        let apple_product_id = "mock_product_id";
        let app_account_token = Some("mock_app_account_token"); // This could be used to identify the user
        let purchase_date = Utc::now();
        let expires_date = Some(Utc::now() + chrono::Duration::days(30)); // 30 days subscription
        
        // Find the product by Apple product ID
        let product = Product::find_by_store_product_id("apple", apple_product_id, pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", apple_product_id)))?;
        
        // Find or create the user
        // In a real app, you'd have a way to map app_account_token to your own user IDs
        // For this example, we'll just create a user if none exists
        let user_id = if let Some(token) = app_account_token {
            // Try to find a user by app_account_token
            // This is just a mock - in a real app, you'd have your own mapping
            let user = User::find_by_app_user_id(token, pool).await?;
            
            match user {
                Some(user) => user.id,
                None => {
                    // Create a new user
                    let new_user = User::new(token.to_string(), None);
                    new_user.create(pool).await?;
                    new_user.id
                }
            }
        } else {
            // Without an app_account_token, we can't identify the user
            return Err(AppError::BadRequest("Missing app_account_token".to_string()));
        };
        
        // Create a new subscription
        let subscription = Subscription::new(
            user_id.clone(),
            product.id.clone(),
            Some(original_transaction_id.to_string()),
            Some(transaction_id.to_string()),
            "apple".to_string(),
            purchase_date,
            expires_date,
            SubscriptionStatus::Active,
            Some(true), // Auto-renew is on for new subscriptions
            None,      // Price paid (not available in this mock)
            None,      // Currency (not available in this mock)
            false,     // Is trial
            false,     // Is intro offer
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
                purchase_date,
                expires_date,
            );
            
            user_entitlement.create(pool).await?;
        }
    }
    
    Ok(())
}

// Process subscription renewal
async fn process_subscription_renewal(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // In a real implementation, we would decode the JWT token
        // For demo purposes, let's assume we have the decoded data
        
        // Mock the decoded data
        let transaction_id = "mock_transaction_id";
        let original_transaction_id = "mock_original_transaction_id";
        let expires_date = Some(Utc::now() + chrono::Duration::days(30)); // 30 more days
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update subscription details
        subscription.store_transaction_id = Some(transaction_id.to_string());
        subscription.expires_date = expires_date;
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
                    entitlement.update_expiry(expires_date, pool).await?;
                }
            }
        }
    }
    
    Ok(())
}

// Process subscription expiration
async fn process_subscription_expiration(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
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
    }
    
    Ok(())
}

// Process renewal status change
async fn process_renewal_status_change(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_renewal_info) = &payload.data.signed_renewal_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        let auto_renew_status = 0; // 0 = off, 1 = on
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update auto-renew status
        subscription.update_auto_renew_status(auto_renew_status == 1, pool).await?;
    }
    
    Ok(())
}

// Process renewal change
async fn process_renewal_change(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    // Similar to process_renewal_status_change
    // but might handle product changes
    Ok(())
}

// Process renewal failure
async fn process_renewal_failure(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        let grace_period_expires_date = Some(Utc::now() + chrono::Duration::days(16)); // 16 days grace period
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update subscription status to grace period
        subscription.status = SubscriptionStatus::GracePeriod.to_string();
        subscription.renewal_grace_period_expires_date = grace_period_expires_date;
        subscription.update(pool).await?;
    }
    
    Ok(())
}

// Process grace period expiration
async fn process_grace_period_expiration(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update subscription status to expired
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
    }
    
    Ok(())
}

// Process offer redemption
async fn process_offer_redemption(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    // Handle offer redemption
    // Similar to process_new_subscription but with offer details
    Ok(())
}

// Process price increase
async fn process_price_increase(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    // Handle price increase notification
    // Typically just store the information for tracking
    Ok(())
}

// Process refund
async fn process_refund(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let transaction_id = "mock_transaction_id";
        let original_transaction_id = "mock_original_transaction_id";
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update subscription status to refunded
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
    }
    
    Ok(())
}

// Process refund declined
async fn process_refund_declined(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    // Handle refund declined notification
    // Typically just store the information for tracking
    Ok(())
}

// Process renewal extension
async fn process_renewal_extension(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        let new_expires_date = Some(Utc::now() + chrono::Duration::days(45)); // Extended period
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Update expiry date
        subscription.update_expiry(new_expires_date.unwrap(), pool).await?;
        
        // Update user entitlements
        let user_entitlements = UserEntitlement::list_active_for_user(
            &subscription.user_id, 
            Utc::now(), 
            pool
        ).await?;
        
        for mut entitlement in user_entitlements {
            if let Some(sub_id) = &entitlement.subscription_id {
                if sub_id == &subscription.id {
                    entitlement.update_expiry(new_expires_date, pool).await?;
                }
            }
        }
    }
    
    Ok(())
}

// Process subscription revocation
async fn process_subscription_revocation(
    payload: &AppleNotificationPayload,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(signed_transaction_info) = &payload.data.signed_transaction_info {
        // Mock the decoded data
        let original_transaction_id = "mock_original_transaction_id";
        
        // Find the subscription by original transaction ID
        let mut subscription = Subscription::find_by_store_transaction(
            "apple", 
            original_transaction_id, 
            pool
        )
        .await?
        .ok_or_else(|| AppError::NotFound(
            format!("Subscription not found: {}", original_transaction_id)
        ))?;
        
        // Cancel the subscription
        subscription.cancel(Utc::now(), pool).await?;
        
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
    }
    
    Ok(())
}
