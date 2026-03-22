//! Time utilities

use std::time::Duration;

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs < 1 {
        format!("{}ms", duration.as_millis())
    } else if secs < 60 {
        format!("{:.1}s", secs as f64)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Parse human-readable duration to Duration
pub fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim().to_lowercase();

    if s.ends_with("ms") {
        s[..s.len() - 2]
            .parse::<u64>()
            .ok()
            .map(Duration::from_millis)
    } else if s.ends_with('s') {
        s[..s.len() - 1]
            .parse::<u64>()
            .ok()
            .map(Duration::from_secs)
    } else if s.ends_with('m') {
        s[..s.len() - 1]
            .parse::<u64>()
            .ok()
            .map(|mins| Duration::from_secs(mins * 60))
    } else if s.ends_with('h') {
        s[..s.len() - 1]
            .parse::<u64>()
            .ok()
            .map(|hours| Duration::from_secs(hours * 3600))
    } else {
        s.parse::<u64>()
            .ok()
            .map(Duration::from_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(5)), "5.0s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }
}
