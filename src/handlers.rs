use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::{
    models::{ConvertRequest, ConvertResponse, ErrorResponse, TimezoneInfo, TimezoneListItem},
    service::EpochZoneService,
};

// Handler for getting timezone information
pub async fn get_timezone_info(
    Path(timezone_name): Path<String>,
) -> Result<Json<TimezoneInfo>, (StatusCode, Json<ErrorResponse>)> {
    EpochZoneService::get_timezone_info(&timezone_name)
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(e)),
            )
        })
}

// Handler for getting list of all timezones
pub async fn get_timezones() -> Json<Vec<TimezoneListItem>> {
    let timezones = EpochZoneService::get_all_timezones();
    Json(timezones)
}

// Handler for converting time between timezones
pub async fn convert_timezone(
    Json(payload): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, (StatusCode, Json<ErrorResponse>)> {
    EpochZoneService::convert_timezone(&payload)
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(e)),
            )
        })
}

// Health check handler
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "timezone-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        // Health check should always succeed
        let _ = response.into_response();
    }

    #[tokio::test]
    async fn test_get_timezones() {
        let Json(timezones) = get_timezones().await;
        assert!(!timezones.is_empty());
    }

    #[tokio::test]
    async fn test_get_timezone_info_success() {
        let result = get_timezone_info(Path("UTC".to_string())).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_get_timezone_info_success_belgrade() {
        let result = get_timezone_info(Path("Europe/Belgrade".to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_timezone_info_failure() {
        let result = get_timezone_info(Path("Invalid/Zone".to_string())).await;
        assert!(result.is_err());

        if let Err((status, _)) = result {
            assert_eq!(status, StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_convert_timezone_handler_success() {
        let payload = ConvertRequest {
            timestamp: Some(1707580800),
            datetime: None,
            from: None,
            to: "America/New_York".to_string(),
        };
        let result = convert_timezone(Json(payload)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convert_timezone_handler_error() {
        let payload = ConvertRequest {
            timestamp: None,
            datetime: None,
            from: None,
            to: "America/New_York".to_string(),
        };
        let result = convert_timezone(Json(payload)).await;
        assert!(result.is_err());

        if let Err((status, _)) = result {
            assert_eq!(status, StatusCode::BAD_REQUEST);
        }
    }
}
