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
