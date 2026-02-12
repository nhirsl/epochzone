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

use crate::models::{ConvertRequest, ConvertResponse, ConvertTimezoneInfo, TimezoneInfo, TimezoneListItem};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use chrono_tz::OffsetComponents;

// Core timezone service handling all timezone operations
pub struct EpochZoneService;

impl EpochZoneService {
    // Get current time and metadata for a specific timezone
    pub fn get_timezone_info(timezone_name: &str) -> Result<TimezoneInfo, String> {
        // Parse the timezone
        let tz: Tz = timezone_name
            .parse()
            .map_err(|_| format!("Invalid timezone: {}", timezone_name))?;

        // Get current time in UTC
        let utc_now: DateTime<Utc> = Utc::now();

        // Convert to the requested timezone
        let local_time = utc_now.with_timezone(&tz);

        // Get UTC offset string using format (always works)
        let offset_str = format!("{}", local_time.format("%z"));
        // Parse it: +0530 or -0800
        let offset_string = if offset_str.len() >= 5 {
            let sign = &offset_str[0..1];
            let hours = &offset_str[1..3];
            let minutes = &offset_str[3..5];
            format!("UTC{}{}:{}", sign, hours, minutes)
        } else {
            "UTC+00:00".to_string()
        };

        // Get timezone abbreviation (e.g., PST, EST)
        let abbreviation = Self::format_abbreviation(&local_time);

        // Determine if DST is active
        let is_dst = Self::is_daylight_saving_time(&tz, &utc_now);

        Ok(TimezoneInfo {
            timezone: timezone_name.to_string(),
            current_time: local_time.to_rfc3339(),
            utc_offset: offset_string,
            abbreviation,
            is_dst,
            timestamp: utc_now.timestamp(),
        })
    }

    // Get a list of all available timezones
    pub fn get_all_timezones() -> Vec<TimezoneListItem> {
        TZ_VARIANTS
            .iter()
            .map(|tz| {
                let name = tz.name().to_string();
                let display_name = name.replace('_', " ");
                TimezoneListItem { name, display_name }
            })
            .collect()
    }

    // Return timezone abbreviation, or "N/A" if chrono only provides a numeric offset
    fn format_abbreviation<T: chrono::TimeZone>(dt: &DateTime<T>) -> String
    where
        T::Offset: std::fmt::Display,
    {
        let abbr = format!("{}", dt.format("%Z"));
        if abbr.starts_with('+') || abbr.starts_with('-') {
            "N/A".to_string()
        } else {
            abbr
        }
    }

    // Check if a timezone is currently observing daylight saving time
    fn is_daylight_saving_time(tz: &Tz, utc_now: &DateTime<Utc>) -> bool {
        utc_now.with_timezone(tz).offset().dst_offset().num_seconds() != 0
    }

    // Look up timezone from geographic coordinates and return full timezone info
    pub fn get_timezone_by_coordinates(
        finder: &tzf_rs::DefaultFinder,
        lat: f64,
        lng: f64,
    ) -> Result<TimezoneInfo, String> {
        let tz_name = finder.get_tz_name(lng, lat);
        Self::get_timezone_info(tz_name)
    }

    // Validate if a timezone name is valid
    pub fn is_valid_timezone(timezone_name: &str) -> bool {
        timezone_name.parse::<Tz>().is_ok()
    }

    // Convert a time between timezones
    pub fn convert_timezone(request: &ConvertRequest) -> Result<ConvertResponse, String> {
        // Parse target timezone
        let to_tz: Tz = request
            .to
            .parse()
            .map_err(|_| format!("Invalid target timezone: {}", request.to))?;

        // Determine the UTC instant and source timezone
        let (utc_instant, from_tz): (DateTime<Utc>, Tz) = match (
            request.timestamp,
            request.datetime.as_deref(),
            request.from.as_deref(),
        ) {
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
                return Err(
                    "Provide either 'timestamp' or 'datetime'+'from', not both".to_string(),
                );
            }
            (Some(ts), None, None) => {
                let utc = DateTime::from_timestamp(ts, 0)
                    .ok_or_else(|| format!("Invalid timestamp: {}", ts))?;
                (utc, chrono_tz::UTC)
            }
            (None, Some(dt_str), Some(from_str)) => {
                let from_tz: Tz = from_str
                    .parse()
                    .map_err(|_| format!("Invalid source timezone: {}", from_str))?;
                let naive = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S")
                    .or_else(|_| NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M"))
                    .map_err(|e| format!("Invalid datetime '{}': {}", dt_str, e))?;
                let local = from_tz
                    .from_local_datetime(&naive)
                    .single()
                    .ok_or_else(|| {
                        format!("Ambiguous or invalid local time '{}' in {}", dt_str, from_str)
                    })?;
                (local.with_timezone(&Utc), from_tz)
            }
            (None, None, _) => {
                return Err("Either 'timestamp' or 'datetime'+'from' is required".to_string());
            }
            (None, Some(_), None) => {
                return Err("'from' timezone is required when using 'datetime'".to_string());
            }
        };

        let from_info = Self::build_convert_info(&utc_instant, &from_tz);
        let to_info = Self::build_convert_info(&utc_instant, &to_tz);

        Ok(ConvertResponse {
            from: from_info,
            to: to_info,
        })
    }

    // Build a ConvertTimezoneInfo for a given UTC instant in a given timezone
    fn build_convert_info(utc: &DateTime<Utc>, tz: &Tz) -> ConvertTimezoneInfo {
        let local = utc.with_timezone(tz);

        let offset_str = format!("{}", local.format("%z"));
        let utc_offset = if offset_str.len() >= 5 {
            let sign = &offset_str[0..1];
            let hours = &offset_str[1..3];
            let minutes = &offset_str[3..5];
            format!("UTC{}{}:{}", sign, hours, minutes)
        } else {
            "UTC+00:00".to_string()
        };

        let abbreviation = Self::format_abbreviation(&local);
        let is_dst = Self::is_daylight_saving_time(tz, utc);

        ConvertTimezoneInfo {
            timezone: tz.name().to_string(),
            datetime: local.to_rfc3339(),
            utc_offset,
            abbreviation,
            is_dst,
            timestamp: utc.timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_timezone_info_valid() {
        let result = EpochZoneService::get_timezone_info("America/New_York");
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert_eq!(info.timezone, "America/New_York");
        assert!(!info.current_time.is_empty());
    }

    #[test]
    fn test_get_timezone_info_invalid() {
        let result = EpochZoneService::get_timezone_info("Invalid/Timezone");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_timezone_info_utc() {
        let result = EpochZoneService::get_timezone_info("UTC");
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert_eq!(info.utc_offset, "UTC+00:00");
        assert_eq!(info.is_dst, false);
    }

    #[test]
    fn test_get_all_timezones() {
        let timezones = EpochZoneService::get_all_timezones();
        assert!(!timezones.is_empty());
        assert!(timezones.len() > 500); // Should have 600+ timezones
    }

    #[test]
    fn test_is_valid_timezone() {
        assert!(EpochZoneService::is_valid_timezone("America/New_York"));
        assert!(EpochZoneService::is_valid_timezone("Europe/London"));
        assert!(EpochZoneService::is_valid_timezone("UTC"));
        assert!(!EpochZoneService::is_valid_timezone("Invalid/Timezone"));
    }

    #[test]
    fn test_timezone_list_format() {
        let timezones = EpochZoneService::get_all_timezones();
        let ny = timezones.iter().find(|tz| tz.name == "America/New_York");

        assert!(ny.is_some());
        let ny = ny.unwrap();
        assert_eq!(ny.display_name, "America/New York");
    }

    #[test]
    fn test_convert_timezone_with_timestamp() {
        let request = ConvertRequest {
            timestamp: Some(1707580800),
            datetime: None,
            from: None,
            to: "America/New_York".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.from.timezone, "UTC");
        assert_eq!(resp.to.timezone, "America/New_York");
        assert_eq!(resp.from.timestamp, resp.to.timestamp);
    }

    #[test]
    fn test_convert_timezone_with_datetime() {
        let request = ConvertRequest {
            timestamp: None,
            datetime: Some("2025-02-10T15:30:00".to_string()),
            from: Some("Europe/Belgrade".to_string()),
            to: "America/New_York".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.from.timezone, "Europe/Belgrade");
        assert_eq!(resp.to.timezone, "America/New_York");
        assert_eq!(resp.from.timestamp, resp.to.timestamp);
        // Belgrade is CET (UTC+1) in February, NY is EST (UTC-5), difference is 6 hours
        assert!(resp.to.datetime.contains("09:30:00"));
    }

    #[test]
    fn test_convert_timezone_invalid_target() {
        let request = ConvertRequest {
            timestamp: Some(1707580800),
            datetime: None,
            from: None,
            to: "Invalid/Zone".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid target timezone"));
    }

    #[test]
    fn test_convert_timezone_missing_fields() {
        let request = ConvertRequest {
            timestamp: None,
            datetime: None,
            from: None,
            to: "UTC".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required"));
    }

    #[test]
    fn test_convert_timezone_both_timestamp_and_datetime() {
        let request = ConvertRequest {
            timestamp: Some(1707580800),
            datetime: Some("2025-02-10T15:30:00".to_string()),
            from: Some("UTC".to_string()),
            to: "America/New_York".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not both"));
    }

    #[test]
    fn test_get_timezone_by_coordinates_tokyo() {
        let finder = tzf_rs::DefaultFinder::new();
        let result = EpochZoneService::get_timezone_by_coordinates(&finder, 35.6762, 139.6503);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().timezone, "Asia/Tokyo");
    }

    #[test]
    fn test_get_timezone_by_coordinates_new_york() {
        let finder = tzf_rs::DefaultFinder::new();
        let result = EpochZoneService::get_timezone_by_coordinates(&finder, 40.7128, -74.0060);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().timezone, "America/New_York");
    }

    #[test]
    fn test_get_timezone_by_coordinates_london() {
        let finder = tzf_rs::DefaultFinder::new();
        let result = EpochZoneService::get_timezone_by_coordinates(&finder, 51.5074, -0.1278);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().timezone, "Europe/London");
    }

    #[test]
    fn test_get_timezone_by_coordinates_ocean() {
        let finder = tzf_rs::DefaultFinder::new();
        // Middle of the Pacific Ocean
        let result = EpochZoneService::get_timezone_by_coordinates(&finder, 0.0, -160.0);
        // tzf-rs returns a timezone even for ocean points (nearest land timezone)
        // so we just verify it doesn't error
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_timezone_datetime_without_from() {
        let request = ConvertRequest {
            timestamp: None,
            datetime: Some("2025-02-10T15:30:00".to_string()),
            from: None,
            to: "America/New_York".to_string(),
        };
        let result = EpochZoneService::convert_timezone(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("'from' timezone is required"));
    }
}
