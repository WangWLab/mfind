//! Query AST (Abstract Syntax Tree)

use std::time::SystemTime;

/// File kind filter
#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub is_dir: bool,
}

/// Query node types
#[derive(Debug, Clone)]
pub enum QueryNode {
    /// Filename pattern match
    Filename { pattern: Pattern, case_sensitive: bool },
    /// Path pattern match
    Path { pattern: Pattern },
    /// Extension match
    Extension { ext: String },
    /// Size range
    Size { min: Option<u64>, max: Option<u64> },
    /// Modified time range
    Modified { after: Option<SystemTime>, before: Option<SystemTime> },
    /// File type filter
    FileType { kind: FileKind },
    /// Boolean AND
    And { left: Box<QueryNode>, right: Box<QueryNode> },
    /// Boolean OR
    Or { left: Box<QueryNode>, right: Box<QueryNode> },
    /// Boolean NOT
    Not { inner: Box<QueryNode> },
    /// Fuzzy match
    Fuzzy { term: String, threshold: f64 },
}

/// Pattern types
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Exact match
    Exact(String),
    /// Prefix match
    Prefix(String),
    /// Wildcard pattern (* and ?)
    Wildcard(String),
    /// Regular expression
    Regex(regex::Regex),
}

/// Parsed query
#[derive(Debug, Clone)]
pub struct Query {
    pub root: QueryNode,
    pub pattern: String,
}

impl Query {
    /// Create a simple prefix query
    pub fn prefix(pattern: String) -> Self {
        Self {
            root: QueryNode::Filename {
                pattern: Pattern::Prefix(pattern.clone()),
                case_sensitive: false,
            },
            pattern,
        }
    }

    /// Create a wildcard query
    pub fn wildcard(pattern: String) -> Self {
        Self {
            root: QueryNode::Filename {
                pattern: Pattern::Wildcard(pattern.clone()),
                case_sensitive: false,
            },
            pattern,
        }
    }

    /// Create a regex query
    pub fn regex(pattern: String) -> Result<Self, regex::Error> {
        let regex = regex::Regex::new(&pattern)?;
        Ok(Self {
            root: QueryNode::Filename {
                pattern: Pattern::Regex(regex),
                case_sensitive: false,
            },
            pattern,
        })
    }
}
