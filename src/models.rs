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

// Request for timezone conversion
#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub timestamp: Option<i64>,
    pub datetime: Option<String>,
    pub from: Option<String>,
    pub to: String,
}

// Timezone info for one side of a conversion
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertTimezoneInfo {
    pub timezone: String,
    pub datetime: String,
    pub utc_offset: String,
    pub abbreviation: String,
    pub is_dst: bool,
    pub timestamp: i64,
}

// Response for timezone conversion
#[derive(Debug, Serialize, Deserialize)]
pub struct ConvertResponse {
    pub from: ConvertTimezoneInfo,
    pub to: ConvertTimezoneInfo,
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
