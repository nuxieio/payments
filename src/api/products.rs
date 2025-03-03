use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::db::models::{Product, ProductType, Entitlement};
use crate::error::{AppError, Result};

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub apple_product_id: Option<String>,
    pub google_product_id: Option<String>,
    pub type_: String,
    pub price_usd: Option<f64>,
    pub duration_days: Option<i32>,
    pub entitlements: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductsResponse {
    pub products: Vec<ProductResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub apple_product_id: Option<String>,
    pub google_product_id: Option<String>,
    pub type_: String,
    pub price_usd: Option<f64>,
    pub duration_days: Option<i32>,
    pub entitlement_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub apple_product_id: Option<String>,
    pub google_product_id: Option<String>,
    pub price_usd: Option<f64>,
    pub duration_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddEntitlementRequest {
    pub entitlement_id: String,
}

// Get all products
pub async fn get_products(
    State(pool): State<SqlitePool>,
) -> Result<Json<ProductsResponse>> {
    let products = Product::list_all(&pool).await?;
    
    let mut product_responses = Vec::new();
    
    for product in products {
        let entitlements = product.get_entitlements(&pool).await?;
        
        product_responses.push(ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            apple_product_id: product.apple_product_id,
            google_product_id: product.google_product_id,
            type_: product.type_,
            price_usd: product.price_usd,
            duration_days: product.duration_days,
            entitlements,
        });
    }
    
    Ok(Json(ProductsResponse {
        products: product_responses,
    }))
}

// Get a specific product
pub async fn get_product(
    Path(product_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<ProductResponse>> {
    let product = Product::find_by_id(&product_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;
    
    let entitlements = product.get_entitlements(&pool).await?;
    
    Ok(Json(ProductResponse {
        id: product.id,
        name: product.name,
        description: product.description,
        apple_product_id: product.apple_product_id,
        google_product_id: product.google_product_id,
        type_: product.type_,
        price_usd: product.price_usd,
        duration_days: product.duration_days,
        entitlements,
    }))
}

// Delete a product
pub async fn delete_product(
    Path(product_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    let product = Product::find_by_id(&product_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;
    
    product.delete(&pool).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

// Create a new product
pub async fn create_product(
    State(pool): State<SqlitePool>,
    Json(request): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>)> {
    // Parse product type
    let product_type = match request.type_.to_lowercase().as_str() {
        "subscription" => ProductType::Subscription,
        "one_time" => ProductType::OneTime,
        _ => return Err(AppError::BadRequest("Invalid product type".to_string())),
    };
    
    // Create the product
    let product = Product::new(
        request.name,
        request.description,
        request.apple_product_id,
        request.google_product_id,
        product_type,
        request.price_usd,
        request.duration_days,
    );
    
    product.create(&pool).await?;
    
    // Verify and add entitlements
    for entitlement_id in &request.entitlement_ids {
        // Check if the entitlement exists
        let _entitlement = Entitlement::find_by_id(entitlement_id, &pool)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Entitlement not found: {}", entitlement_id))
            })?;
        
        // Add the entitlement to the product
        product.add_entitlement(entitlement_id, &pool).await?;
    }
    
    // Get all entitlements for response
    let entitlements = product.get_entitlements(&pool).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            apple_product_id: product.apple_product_id,
            google_product_id: product.google_product_id,
            type_: product.type_,
            price_usd: product.price_usd,
            duration_days: product.duration_days,
            entitlements,
        }),
    ))
}

// Update a product
pub async fn update_product(
    Path(product_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(request): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>> {
    let mut product = Product::find_by_id(&product_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;
    
    // Update fields if provided
    if let Some(name) = request.name {
        product.name = name;
    }
    
    if let Some(description) = request.description {
        product.description = Some(description);
    }
    
    if let Some(apple_product_id) = request.apple_product_id {
        product.apple_product_id = Some(apple_product_id);
    }
    
    if let Some(google_product_id) = request.google_product_id {
        product.google_product_id = Some(google_product_id);
    }
    
    if let Some(price_usd) = request.price_usd {
        product.price_usd = Some(price_usd);
    }
    
    if let Some(duration_days) = request.duration_days {
        product.duration_days = Some(duration_days);
    }
    
    product.update(&pool).await?;
    
    let entitlements = product.get_entitlements(&pool).await?;
    
    Ok(Json(ProductResponse {
        id: product.id,
        name: product.name,
        description: product.description,
        apple_product_id: product.apple_product_id,
        google_product_id: product.google_product_id,
        type_: product.type_,
        price_usd: product.price_usd,
        duration_days: product.duration_days,
        entitlements,
    }))
}

// Add an entitlement to a product
pub async fn add_product_entitlement(
    Path(product_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(request): Json<AddEntitlementRequest>,
) -> Result<StatusCode> {
    let product = Product::find_by_id(&product_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;
    
    // Check if the entitlement exists
    let _entitlement = Entitlement::find_by_id(&request.entitlement_id, &pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Entitlement not found: {}", request.entitlement_id))
        })?;
    
    // Add the entitlement to the product
    product.add_entitlement(&request.entitlement_id, &pool).await?;
    
    Ok(StatusCode::OK)
}

// Remove an entitlement from a product
pub async fn remove_product_entitlement(
    Path((product_id, entitlement_id)): Path<(String, String)>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    let product = Product::find_by_id(&product_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;
    
    // Check if the entitlement exists
    let _entitlement = Entitlement::find_by_id(&entitlement_id, &pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entitlement not found: {}", entitlement_id)))?;
    
    // Remove the entitlement from the product
    product.remove_entitlement(&entitlement_id, &pool).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
