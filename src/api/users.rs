use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::db::models::{User, Subscription, SubscriptionStatus};
use crate::error::{AppError, Result};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub app_user_id: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub product_id: String,
    pub store: String,
    pub purchase_date: chrono::DateTime<chrono::Utc>,
    pub expires_date: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub auto_renew_status: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserSubscriptionsResponse {
    pub subscriptions: Vec<SubscriptionResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub app_user_id: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
}

// Get all users
pub async fn get_users(
    State(pool): State<SqlitePool>,
) -> Result<Json<UsersResponse>> {
    // In a real application, you'd implement pagination
    // For simplicity, we'll just limit to the first 100 users
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&pool)
    .await?;
    
    let user_responses = users
        .into_iter()
        .map(|user| UserResponse {
            id: user.id,
            app_user_id: user.app_user_id,
            email: user.email,
        })
        .collect();
    
    Ok(Json(UsersResponse {
        users: user_responses,
    }))
}

// Get a specific user
pub async fn get_user(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserResponse>> {
    let user = User::find_by_id(&user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
    
    Ok(Json(UserResponse {
        id: user.id,
        app_user_id: user.app_user_id,
        email: user.email,
    }))
}

// Get user by app_user_id
pub async fn get_user_by_app_id(
    Path(app_user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserResponse>> {
    let user = User::find_by_app_user_id(&app_user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found with app_user_id: {}", app_user_id)))?;
    
    Ok(Json(UserResponse {
        id: user.id,
        app_user_id: user.app_user_id,
        email: user.email,
    }))
}

// Create a new user
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    // Check if a user with this app_user_id already exists
    if let Some(_) = User::find_by_app_user_id(&request.app_user_id, &pool).await? {
        return Err(AppError::BadRequest(format!(
            "User with app_user_id {} already exists",
            request.app_user_id
        )));
    }
    
    // Create the user
    let user = User::new(request.app_user_id, request.email);
    user.create(&pool).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id,
            app_user_id: user.app_user_id,
            email: user.email,
        }),
    ))
}

// Update a user
pub async fn update_user(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let mut user = User::find_by_id(&user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
    
    // Update fields if provided
    if let Some(email) = request.email {
        user.email = Some(email);
    }
    
    user.update(&pool).await?;
    
    Ok(Json(UserResponse {
        id: user.id,
        app_user_id: user.app_user_id,
        email: user.email,
    }))
}

// Delete a user
pub async fn delete_user(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    let user = User::find_by_id(&user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
    
    user.delete(&pool).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

// Get all subscriptions for a user
pub async fn get_user_subscriptions(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserSubscriptionsResponse>> {
    // Check if the user exists
    let _user = User::find_by_id(&user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
    
    // Get all subscriptions for the user
    let subscriptions = Subscription::list_by_user(&user_id, &pool).await?;
    
    let subscription_responses = subscriptions
        .into_iter()
        .map(|subscription| SubscriptionResponse {
            id: subscription.id,
            product_id: subscription.product_id,
            store: subscription.store,
            purchase_date: subscription.purchase_date,
            expires_date: subscription.expires_date,
            status: subscription.status,
            auto_renew_status: subscription.auto_renew_status,
        })
        .collect();
    
    Ok(Json(UserSubscriptionsResponse {
        subscriptions: subscription_responses,
    }))
}

// Get active subscriptions for a user
pub async fn get_user_active_subscriptions(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserSubscriptionsResponse>> {
    // Check if the user exists
    let _user = User::find_by_id(&user_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
    
    // Get active subscriptions for the user
    let subscriptions = Subscription::list_active_by_user(&user_id, &pool).await?;
    
    let subscription_responses = subscriptions
        .into_iter()
        .map(|subscription| SubscriptionResponse {
            id: subscription.id,
            product_id: subscription.product_id,
            store: subscription.store,
            purchase_date: subscription.purchase_date,
            expires_date: subscription.expires_date,
            status: subscription.status,
            auto_renew_status: subscription.auto_renew_status,
        })
        .collect();
    
    Ok(Json(UserSubscriptionsResponse {
        subscriptions: subscription_responses,
    }))
}
