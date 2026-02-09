use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use crate::handlers;

// Build the application router with all routes configured
pub fn create_router() -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with all routes
    Router::new()
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check))
        .route("/api/timezones", get(handlers::get_timezones))
        .route("/api/time/{timezone}", get(handlers::get_timezone_info))
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_router();

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
    async fn test_timezones_endpoint() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/timezones")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timezone_info_endpoint_valid() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/UTC")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_timezone_info_endpoint_valid_belgrade() {
    let app = create_router();
    let response = app
        .oneshot(
            Request::builder()
                // Use %2F for the slash in Europe/Belgrade
                .uri("/api/time/Europe%2FBelgrade")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_timezone_info_endpoint_invalid() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/time/Invalid_Zone")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
