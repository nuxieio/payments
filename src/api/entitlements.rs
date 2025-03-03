use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::db::models::{UserEntitlement, Entitlement};
use crate::error::{AppError, Result};

#[derive(Debug, Serialize)]
pub struct EntitlementResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserEntitlementResponse {
    pub id: String,
    pub entitlement: EntitlementResponse,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct UserEntitlementsResponse {
    pub entitlements: Vec<UserEntitlementResponse>,
}

#[derive(Debug, Serialize)]
pub struct EntitlementAccessResponse {
    pub has_access: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntitlementRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GrantEntitlementRequest {
    pub user_id: String,
    pub entitlement_id: String,
    pub expires_at: Option<DateTime<Utc>>,
}

// Get all entitlements for a user
pub async fn get_user_entitlements(
    Path(user_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserEntitlementsResponse>> {
    let now = Utc::now();
    
    // Get all active entitlements for the user
    let user_entitlements = UserEntitlement::list_active_for_user(&user_id, now, &pool).await?;
    
    let mut entitlement_responses = Vec::new();
    
    for user_entitlement in user_entitlements {
        let entitlement = Entitlement::find_by_id(&user_entitlement.entitlement_id, &pool)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Entitlement not found: {}", user_entitlement.entitlement_id))
            })?;
        
        let is_active = match user_entitlement.expires_at {
            Some(expires_at) => expires_at > now,
            None => true, // No expiration means lifetime access
        };
        
        entitlement_responses.push(UserEntitlementResponse {
            id: user_entitlement.id,
            entitlement: EntitlementResponse {
                id: entitlement.id,
                name: entitlement.name,
                description: entitlement.description,
            },
            expires_at: user_entitlement.expires_at,
            active: is_active,
        });
    }
    
    Ok(Json(UserEntitlementsResponse {
        entitlements: entitlement_responses,
    }))
}

// Check if a user has access to a specific entitlement
pub async fn check_entitlement_access(
    Path((user_id, entitlement_id)): Path<(String, String)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<EntitlementAccessResponse>> {
    let now = Utc::now();
    
    // Check if the user has an active entitlement
    let user_entitlement = UserEntitlement::find_active_for_user(&user_id, &entitlement_id, now, &pool).await?;
    
    let has_access = user_entitlement.is_some();
    let expires_at = user_entitlement.map(|ue| ue.expires_at).flatten();
    
    Ok(Json(EntitlementAccessResponse {
        has_access,
        expires_at,
    }))
}

// Create a new entitlement
pub async fn create_entitlement(
    State(pool): State<SqlitePool>,
    Json(request): Json<CreateEntitlementRequest>,
) -> Result<(StatusCode, Json<EntitlementResponse>)> {
    let entitlement = Entitlement::new(request.name, request.description);
    
    entitlement.create(&pool).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(EntitlementResponse {
            id: entitlement.id,
            name: entitlement.name,
            description: entitlement.description,
        }),
    ))
}

// Manually grant an entitlement to a user
pub async fn grant_entitlement(
    State(pool): State<SqlitePool>,
    Json(request): Json<GrantEntitlementRequest>,
) -> Result<StatusCode> {
    // Check if the entitlement exists
    let _entitlement = Entitlement::find_by_id(&request.entitlement_id, &pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Entitlement not found: {}", request.entitlement_id))
        })?;
    
    // Create the user entitlement
    let user_entitlement = UserEntitlement::new(
        request.user_id,
        request.entitlement_id,
        None, // Not tied to a subscription
        Utc::now(),
        request.expires_at,
    );
    
    user_entitlement.create(&pool).await?;
    
    Ok(StatusCode::CREATED)
}

// Revoke an entitlement from a user
pub async fn revoke_entitlement(
    Path((user_id, entitlement_id)): Path<(String, String)>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    let now = Utc::now();
    
    // Find the active entitlement
    let user_entitlement = UserEntitlement::find_active_for_user(&user_id, &entitlement_id, now, &pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Active entitlement not found for user {} and entitlement {}", 
                user_id, entitlement_id
            ))
        })?;
    
    // Revoke the entitlement
    let mut user_entitlement_mut = user_entitlement;
    user_entitlement_mut.revoke(&pool).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
