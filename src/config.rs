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
            "http://localhost:5173,https://epochzone-ui-production.up.railway.app,https://epoch.zone".to_string()
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
