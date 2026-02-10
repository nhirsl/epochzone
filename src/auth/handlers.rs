use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::models::ErrorResponse;
use crate::AppState;

use super::models::{ApiKeyListItem, CreateApiKeyRequest, CreateApiKeyResponse};
use super::service;

fn verify_admin_key(headers: &HeaderMap, admin_key: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let provided = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(key) if key == admin_key => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("Invalid admin API key")),
        )),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Missing X-API-Key header")),
        )),
    }
}

pub async fn create_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), (StatusCode, Json<ErrorResponse>)> {
    verify_admin_key(&headers, &state.config.admin_api_key)?;

    let response = service::create_api_key(&state.db, payload.name, payload.expires_at)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e)),
            )
        })?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ApiKeyListItem>>, (StatusCode, Json<ErrorResponse>)> {
    verify_admin_key(&headers, &state.config.admin_api_key)?;

    let keys = service::list_api_keys(&state.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e)),
        )
    })?;

    Ok(Json(keys))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    verify_admin_key(&headers, &state.config.admin_api_key)?;

    let revoked = service::revoke_api_key(&state.db, id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e)),
        )
    })?;

    if revoked {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("API key not found")),
        ))
    }
}
