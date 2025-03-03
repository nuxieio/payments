use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub log_level: String,
    pub environment: Environment,
    pub apple_shared_secret: Option<String>,
    pub google_service_account_json: Option<String>,
    pub webhook_signature_secret: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/subscription.db".to_string());
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a number");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string()) // 24 hours in seconds
            .parse()
            .expect("JWT_EXPIRATION must be a number");
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        
        let environment_str = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let environment = match environment_str.to_lowercase().as_str() {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        let apple_shared_secret = env::var("APPLE_SHARED_SECRET").ok();
        let google_service_account_json = env::var("GOOGLE_SERVICE_ACCOUNT_JSON").ok();
        let webhook_signature_secret = env::var("WEBHOOK_SIGNATURE_SECRET")
            .unwrap_or_else(|_| "your-webhook-signature-secret".to_string());

        Config {
            database_url,
            host,
            port,
            jwt_secret,
            jwt_expiration,
            log_level,
            environment,
            apple_shared_secret,
            google_service_account_json,
            webhook_signature_secret,
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn is_test(&self) -> bool {
        self.environment == Environment::Test
    }
}
