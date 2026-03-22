//! Format utilities

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    match bytes {
        0..KB => format!("{} B", bytes),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        GB..TB => format!("{:.1} GB", bytes as f64 / GB as f64),
        _ => format!("{:.1} TB", bytes as f64 / TB as f64),
    }
}

/// Format count with K/M/B suffixes
pub fn format_count(count: u64) -> String {
    const K: u64 = 1000;
    const M: u64 = K * 1000;
    const B: u64 = M * 1000;

    match count {
        0..K => count.to_string(),
        K..M => format!("{:.1}K", count as f64 / K as f64),
        M..B => format!("{:.1}M", count as f64 / M as f64),
        _ => format!("{:.1}B", count as f64 / B as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_572_864), "1.5 MB");
    }

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(500), "500");
        assert_eq!(format_count(1500), "1.5K");
        assert_eq!(format_count(1_500_000), "1.5M");
    }
}
