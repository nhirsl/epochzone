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

use std::sync::Arc;

pub mod auth;
pub mod config;
pub mod db;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod service;

pub use models::*;
pub use service::EpochZoneService;

#[derive(Clone)]
pub struct AppState {
    pub db: tokio_rusqlite::Connection,
    pub config: Arc<config::AppConfig>,
    pub tz_finder: Arc<tzf_rs::DefaultFinder>,
}
