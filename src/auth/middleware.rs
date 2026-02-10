use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};

use crate::models::ErrorResponse;
use crate::AppState;

use super::service::validate_api_key;

pub async fn require_api_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if validate_api_key(&state.db, key).await => Ok(next.run(request).await),
        Some(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid or expired API key")),
        )),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Missing X-API-Key header")),
        )),
    }
}
