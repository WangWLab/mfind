//! Query executor

use crate::query::ast::{Query, QueryNode, Pattern};
use crate::query::{SearchResult, SearchResultItem};
use crate::Result;

use super::super::index::FSTIndex;

/// Query executor
pub struct QueryExecutor<'a> {
    index: &'a FSTIndex,
}

impl<'a> QueryExecutor<'a> {
    /// Create a new executor
    pub fn new(index: &'a FSTIndex) -> Self {
        Self { index }
    }

    /// Execute a query
    pub fn execute(&self, query: &Query) -> Result<SearchResult> {
        let start = std::time::Instant::now();

        let matches = self.execute_node(&query.root)?;

        Ok(SearchResult {
            matches,
            total: matches.len(),
            time_ms: start.elapsed().as_millis(),
        })
    }

    /// Execute a query node
    fn execute_node(&self, node: &QueryNode) -> Result<Vec<String>> {
        match node {
            QueryNode::Filename { pattern, .. } => self.execute_pattern(pattern),
            QueryNode::Path { pattern } => {
                // For path queries, search full paths
                self.execute_pattern(pattern)
            }
            QueryNode::Extension { ext } => {
                // Find all files with given extension
                let pattern = format!("*.{}", ext);
                self.execute_pattern(&Pattern::Wildcard(pattern))
            }
            QueryNode::And { left, right } => {
                let left_results = self.execute_node(left)?;
                let right_results = self.execute_node(right)?;
                Ok(left_results
                    .into_iter()
                    .filter(|item| right_results.contains(item))
                    .collect())
            }
            QueryNode::Or { left, right } => {
                let mut results = self.execute_node(left)?;
                results.extend(self.execute_node(right)?);
                results.sort();
                results.dedup();
                Ok(results)
            }
            QueryNode::Not { inner } => {
                let inner_results = self.execute_node(inner)?;
                let all = self.index.stream();
                Ok(all.into_iter()
                    .filter(|item| !inner_results.contains(item))
                    .collect())
            }
            QueryNode::Size { .. } => {
                // Size filtering requires metadata lookup
                // TODO: Implement with metadata
                Ok(vec![])
            }
            QueryNode::Modified { .. } => {
                // Time filtering requires metadata lookup
                // TODO: Implement with metadata
                Ok(vec![])
            }
            QueryNode::FileType { .. } => {
                // File type filtering
                // TODO: Implement
                Ok(vec![])
            }
            QueryNode::Fuzzy { term, threshold } => {
                // Fuzzy search
                self.execute_fuzzy(term, *threshold)
            }
        }
    }

    /// Execute pattern match
    fn execute_pattern(&self, pattern: &Pattern) -> Result<Vec<String>> {
        match pattern {
            Pattern::Prefix(prefix) => self.index.prefix_search(prefix),
            Pattern::Regex(regex) => self.index.regex_search(regex),
            Pattern::Wildcard(pattern) => {
                let regex = Pattern::wildcard_to_regex(pattern);
                self.index.regex_search(&regex)
            }
            Pattern::Exact(s) => {
                if self.index.contains(s.as_bytes()) {
                    Ok(vec![s.clone()])
                } else {
                    Ok(vec![])
                }
            }
        }
    }

    /// Execute fuzzy search
    fn execute_fuzzy(&self, term: &str, _threshold: f64) -> Result<Vec<String>> {
        // Simple fallback: prefix search
        // TODO: Implement proper fuzzy matching
        self.index.prefix_search(term)
    }
}
