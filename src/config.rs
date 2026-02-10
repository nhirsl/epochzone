use axum::http::HeaderValue;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cors_allowed_origins: Vec<HeaderValue>,
    pub admin_api_key: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let origins_str = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
            "http://localhost:5173,https://epochzone-ui-production.up.railway.app".to_string()
        });

        let cors_allowed_origins: Vec<HeaderValue> = origins_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(
                        HeaderValue::from_str(trimmed)
                            .unwrap_or_else(|_| panic!("Invalid CORS origin: {}", trimmed)),
                    )
                }
            })
            .collect();

        let admin_api_key =
            env::var("ADMIN_API_KEY").expect("ADMIN_API_KEY environment variable is required");
        if admin_api_key.len() < 32 {
            panic!("ADMIN_API_KEY must be at least 32 characters");
        }

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "epochzone.db".to_string());

        Self {
            cors_allowed_origins,
            admin_api_key,
            database_url,
        }
    }
}
