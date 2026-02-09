use crate::models::{TimezoneInfo, TimezoneListItem};
use chrono::{DateTime, Utc};
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
        let abbreviation = format!("{}", local_time.format("%Z"));

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

    // Check if a timezone is currently observing daylight saving time
    fn is_daylight_saving_time(tz: &Tz, utc_now: &DateTime<Utc>) -> bool {
        utc_now.with_timezone(tz).offset().dst_offset().num_seconds() != 0
    }

    // Validate if a timezone name is valid
    pub fn is_valid_timezone(timezone_name: &str) -> bool {
        timezone_name.parse::<Tz>().is_ok()
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
}
