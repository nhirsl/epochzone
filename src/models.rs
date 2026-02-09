use serde::{Deserialize, Serialize};

// Response containing timezone information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimezoneInfo {
    pub timezone: String,
    pub current_time: String,
    pub utc_offset: String,
    pub abbreviation: String,
    pub is_dst: bool,
    pub timestamp: i64,
}

// A single timezone item in the list
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimezoneListItem {
    pub name: String,
    pub display_name: String,
}

// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}
