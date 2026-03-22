//! Path utilities

use std::path::{Component, Path, PathBuf};

/// Normalize a path by resolving . and .. components
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(c) => {
                components.push(c);
            }
            Component::Prefix(_) | Component::RootDir => {
                components.push(component.as_os_str());
            }
        }
    }

    if path.has_root() {
        #[cfg(unix)]
        {
            let mut result = PathBuf::from("/");
            result.push(components.join(std::ffi::OsStr::new("/")));
            result
        }
        #[cfg(windows)]
        {
            let mut result = PathBuf::from("\\");
            result.push(components.join(std::ffi::OsStr::new("\\")));
            result
        }
    } else {
        components.iter().collect()
    }
}

/// Get the parent directory of a path
pub fn parent_dir(path: &Path) -> Option<&Path> {
    path.parent()
}

/// Check if path is hidden (starts with .)
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let path = Path::new("/a/b/../c/./d");
        let normalized = normalize_path(path);
        assert_eq!(normalized, Path::new("/a/c/d"));
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".git")));
        assert!(is_hidden(Path::new("/home/user/.bashrc")));
        assert!(!is_hidden(Path::new("file.txt")));
    }
}
