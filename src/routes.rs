use axum::{
    http::{header, Method},
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::auth;
use crate::handlers;
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            state.config.cors_allowed_origins.clone(),
        ))
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::HeaderName::from_static("x-api-key")]);

    // Public routes - no auth required
    let public_routes = Router::new()
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check));

    // API routes - protected by API key middleware
    let api_routes = Router::new()
        .route("/api/timezones", get(handlers::get_timezones))
        .route("/api/time/{timezone}", get(handlers::get_timezone_info))
        .route("/api/convert", post(handlers::convert_timezone))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_api_key,
        ));

    // Admin routes - admin key checked in handlers
    let admin_routes = Router::new()
        .route(
            "/admin/api-keys",
            post(auth::handlers::create_api_key).get(auth::handlers::list_api_keys),
        )
        .route("/admin/api-keys/{id}", delete(auth::handlers::revoke_api_key));

    public_routes
        .merge(api_routes)
        .merge(admin_routes)
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::config::AppConfig;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn test_state() -> AppState {
        let db = init_db(":memory:").await;
        let config = AppConfig {
            cors_allowed_origins: vec![],
            admin_api_key: "a]".repeat(16), // 32 chars
            database_url: ":memory:".to_string(),
        };
        AppState {
            db,
            config: Arc::new(config),
        }
    }

    fn admin_key() -> String {
        "a]".repeat(16)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_timezones_requires_key() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/timezones")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_timezones_with_valid_key() {
        let state = test_state().await;

        // Create an API key via the service
        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/timezones")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_timezone_info_requires_key() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/UTC")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_timezone_info_with_valid_key() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/UTC")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timezone_info_endpoint_valid_belgrade() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/Europe%2FBelgrade")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timezone_info_endpoint_invalid() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/Invalid_Zone")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_admin_create_key_requires_admin() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/api-keys")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_admin_create_key_with_admin_key() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/api-keys")
                    .header("content-type", "application/json")
                    .header("X-API-Key", admin_key())
                    .body(Body::from(r#"{"name":"test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_admin_list_keys() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/api-keys")
                    .header("X-API-Key", admin_key())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_convert_requires_key() {
        let state = test_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/convert")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"timestamp":1707580800,"to":"America/New_York"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_convert_with_timestamp() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/convert")
                    .header("content-type", "application/json")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::from(
                        r#"{"timestamp":1707580800,"to":"America/New_York"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_convert_with_datetime_and_from() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/convert")
                    .header("content-type", "application/json")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::from(
                        r#"{"datetime":"2025-02-10T15:30:00","from":"Europe/Belgrade","to":"America/New_York"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_convert_with_invalid_timezone() {
        let state = test_state().await;

        let resp = crate::auth::service::create_api_key(&state.db, "test".to_string(), None)
            .await
            .unwrap();

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/convert")
                    .header("content-type", "application/json")
                    .header("X-API-Key", &resp.api_key)
                    .body(Body::from(
                        r#"{"timestamp":1707580800,"to":"Invalid/Zone"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
