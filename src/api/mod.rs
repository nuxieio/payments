pub mod users;
pub mod products;
pub mod subscriptions;
pub mod entitlements;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::sqlite::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

pub fn routes(pool: SqlitePool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // User routes
        .route("/users", get(users::get_users))
        .route("/users", post(users::create_user))
        .route("/users/:user_id", get(users::get_user))
        .route("/users/:user_id", put(users::update_user))
        .route("/users/:user_id", delete(users::delete_user))
        .route("/users/app_id/:app_user_id", get(users::get_user_by_app_id))
        .route("/users/:user_id/subscriptions", get(users::get_user_subscriptions))
        .route("/users/:user_id/subscriptions/active", get(users::get_user_active_subscriptions))
        
        // Entitlement routes
        .route("/entitlements", post(entitlements::create_entitlement))
        .route("/users/:user_id/entitlements", get(entitlements::get_user_entitlements))
        .route("/users/:user_id/entitlements/:entitlement_id", get(entitlements::check_entitlement_access))
        .route("/entitlements/grant", post(entitlements::grant_entitlement))
        .route("/users/:user_id/entitlements/:entitlement_id/revoke", post(entitlements::revoke_entitlement))
        
        // Product routes
        .route("/products", get(products::get_products))
        .route("/products", post(products::create_product))
        .route("/products/:product_id", get(products::get_product))
        .route("/products/:product_id", put(products::update_product))
        .route("/products/:product_id", delete(products::delete_product))
        .route("/products/:product_id/entitlements", post(products::add_product_entitlement))
        .route("/products/:product_id/entitlements/:entitlement_id", delete(products::remove_product_entitlement))
        
        // Subscription routes
        .route("/subscriptions", get(subscriptions::get_subscriptions))
        .route("/subscriptions/:subscription_id", get(subscriptions::get_subscription))
        .route("/subscriptions/:subscription_id/cancel", post(subscriptions::cancel_subscription))
        .route("/subscriptions/:subscription_id/refund", post(subscriptions::refund_subscription))
        
        .layer(cors)
        .with_state(pool)
}
