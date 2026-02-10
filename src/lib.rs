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
}
