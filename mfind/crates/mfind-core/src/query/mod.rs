//! Query module for search parsing and execution

pub mod ast;
pub mod executor;
pub mod parser;
pub mod pattern;

pub use ast::{Query, QueryNode, FileKind, FileMetadata};
pub use parser::QueryParser;
pub use pattern::Pattern;

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort field
    pub sort_by: SortField,
    /// Sort order
    pub order: SortOrder,
    /// Include score in results
    pub include_score: bool,
    /// Highlight matches
    pub highlight: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: Some(1000),
            offset: None,
            sort_by: SortField::Relevance,
            order: SortOrder::Desc,
            include_score: false,
            highlight: false,
        }
    }
}

/// Sort field options
#[derive(Debug, Clone, Copy)]
pub enum SortField {
    Relevance,
    Name,
    Path,
    Size,
    Modified,
    Created,
}

/// Sort order
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub matches: Vec<String>,
    pub total: usize,
    pub time_ms: u128,
}

/// Individual search result item
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub path: String,
    pub score: f64,
    pub metadata: Option<FileMetadata>,
}
