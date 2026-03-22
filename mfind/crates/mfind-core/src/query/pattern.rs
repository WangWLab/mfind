//! Query pattern matching

use regex::Regex;

/// Pattern matching utilities
#[derive(Debug, Clone)]
pub enum Pattern {
    Exact(String),
    Prefix(String),
    Wildcard(String),
    Regex(Regex),
}

impl Pattern {
    /// Check if a string matches the pattern
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Pattern::Exact(s) => text == s,
            Pattern::Prefix(prefix) => text.starts_with(prefix),
            Pattern::Wildcard(pattern) => wildcard_match(pattern, text),
            Pattern::Regex(regex) => regex.is_match(text),
        }
    }

    /// Convert wildcard pattern to regex
    pub fn wildcard_to_regex(wildcard: &str) -> Regex {
        let regex_pattern = regex::escape(wildcard)
            .replace("\\*", ".*")
            .replace("\\?", ".");
        Regex::new(&format!("^{}$", regex_pattern)).unwrap()
    }
}

/// Match wildcard pattern (* and ?)
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let regex = Pattern::wildcard_to_regex(pattern);
    regex.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_star() {
        assert!(wildcard_match("*.txt", "file.txt"));
        assert!(wildcard_match("*.txt", "document.txt"));
        assert!(!wildcard_match("*.txt", "file.pdf"));
    }

    #[test]
    fn test_wildcard_question() {
        assert!(wildcard_match("file?.txt", "file1.txt"));
        assert!(wildcard_match("file?.txt", "fileA.txt"));
        assert!(!wildcard_match("file?.txt", "file10.txt"));
    }

    #[test]
    fn test_prefix() {
        let pattern = Pattern::Prefix("test".to_string());
        assert!(pattern.matches("test_file"));
        assert!(pattern.matches("testing"));
        assert!(!pattern.matches("best"));
    }
}
