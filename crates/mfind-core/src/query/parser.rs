//! Query parser

use crate::query::ast::{Query, QueryNode};
use crate::Result;

/// Query parser
pub struct QueryParser;

impl QueryParser {
    /// Parse a query string
    pub fn parse(input: &str) -> Result<Query> {
        // Simple parsing for now - just prefix search
        // TODO: Implement full query language parsing

        let input = input.trim();

        if input.is_empty() {
            return Ok(Query::prefix("".to_string()));
        }

        // Check for regex: prefix
        if let Some(pattern) = input.strip_prefix("regex:") {
            return Query::regex(pattern.to_string())
                .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e));
        }

        // Check for ext: prefix
        if let Some(ext) = input.strip_prefix("ext:") {
            return Ok(Query {
                root: QueryNode::Extension {
                    ext: ext.to_string(),
                },
                pattern: input.to_string(),
            });
        }

        // Check for wildcard patterns
        if input.contains('*') || input.contains('?') {
            return Ok(Query::wildcard(input.to_string()));
        }

        // Default: prefix search
        Ok(Query::prefix(input.to_string()))
    }

    /// Parse with options
    pub fn parse_with_options(input: &str, _case_sensitive: bool) -> Result<Query> {
        let query = Self::parse(input)?;
        // TODO: Apply case sensitivity
        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::ast::Pattern;

    #[test]
    fn test_parse_prefix() {
        let query = QueryParser::parse("test").unwrap();
        assert_eq!(query.pattern, "test");
    }

    #[test]
    fn test_parse_wildcard() {
        let query = QueryParser::parse("*.txt").unwrap();
        assert!(query.pattern.contains('*'));
    }

    #[test]
    fn test_parse_regex() {
        let query = QueryParser::parse("regex:.*\\.txt$").unwrap();
        // Check that it created a Regex pattern
        match query.root {
            QueryNode::Filename { pattern, .. } => {
                assert!(matches!(pattern, Pattern::Regex(_)));
            }
            _ => panic!("Expected Filename node with Regex pattern"),
        }
    }

    #[test]
    fn test_parse_extension() {
        let query = QueryParser::parse("ext:txt").unwrap();
        match query.root {
            QueryNode::Extension { ext } => assert_eq!(ext, "txt"),
            _ => panic!("Expected Extension node"),
        }
    }
}
