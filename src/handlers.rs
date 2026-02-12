// Epoch Zone
// Copyright (C) 2026 Nemanja Hiršl
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use axum::{
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::{
    models::{ConvertRequest, ConvertResponse, ErrorResponse, GeolocationQuery, TimezoneInfo, TimezoneListItem},
    service::EpochZoneService,
    AppState,
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

// Handler for getting timezone by geographic coordinates
pub async fn get_timezone_by_coordinates(
    State(state): State<AppState>,
    params: Result<Query<GeolocationQuery>, QueryRejection>,
) -> Result<Json<TimezoneInfo>, (StatusCode, Json<ErrorResponse>)> {
    let Query(params) = params.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.body_text())),
        )
    })?;

    EpochZoneService::get_timezone_by_coordinates(&state.tz_finder, params.lat, params.lng)
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
