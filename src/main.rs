mod api;
mod config;
mod db;
mod error;
mod webhooks;
mod providers;
mod utils;

use axum::{
    routing::post,
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = config::Config::from_env();
    
    // Set up logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            &config.log_level,
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting subscription backend");
    tracing::debug!("Config: {:?}", config);
    
    // Connect to the database
    let pool = db::initialize_db(&config.database_url).await?;
    
    // Run database migrations
    db::run_migrations(&pool).await?;
    
    // Check database connection
    if db::check_db_connection(&pool).await? {
        tracing::info!("Connected to the database");
    } else {
        tracing::error!("Failed to connect to the database");
        return Err(anyhow::anyhow!("Failed to connect to the database"));
    }
    
    // Create the API routes
    let api_routes = api::routes(pool.clone());
    
    // Set up the webhook routes
    let webhook_routes = Router::new()
        .route("/webhooks/apple", post(webhooks::handle_apple_webhook))
        .route("/webhooks/google", post(webhooks::handle_google_webhook))
        .with_state(pool);
    
    // Combine all routes
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(webhook_routes);
    
    // Start the HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
