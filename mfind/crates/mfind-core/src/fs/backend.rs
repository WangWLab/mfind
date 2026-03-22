//! Filesystem backend abstraction

use std::path::{Path, PathBuf};
use std::collections::HashSet;

use crate::Result;

use super::{FileSystemMonitor, FileSystemScanner};

/// Filesystem type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemType {
    Apfs,
    HfsPlus,
    ExFat,
    Fat32,
    Ntfs,
    Smb,
    Nfs,
    Unknown(String),
}

/// Filesystem capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileSystemCapability {
    SupportsEvents,
    SupportsHardLinks,
    SupportsSymlinks,
    SupportsExtendedAttributes,
    CaseSensitive,
}

/// Filesystem information
#[derive(Debug, Clone)]
pub struct FileSystemInfo {
    pub fs_type: FileSystemType,
    pub capabilities: HashSet<FileSystemCapability>,
    pub mount_point: PathBuf,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_network: bool,
    pub is_removable: bool,
}

/// Filesystem backend trait
pub trait FileSystemBackend: Send + Sync {
    /// Get filesystem info for a path
    fn get_info(&self, path: &Path) -> Result<FileSystemInfo>;

    /// Get filesystem type
    fn get_fs_type(&self, path: &Path) -> Result<FileSystemType>;

    /// Check if filesystem supports events
    fn supports_events(&self, path: &Path) -> bool {
        matches!(
            self.get_fs_type(path),
            Ok(FileSystemType::Apfs | FileSystemType::HfsPlus)
        )
    }
}

/// Default filesystem backend for the current platform
pub struct DefaultFileSystemBackend;

impl DefaultFileSystemBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFileSystemBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemBackend for DefaultFileSystemBackend {
    fn get_info(&self, path: &Path) -> Result<FileSystemInfo> {
        // TODO: Implement proper filesystem detection
        let fs_type = self.get_fs_type(path)?;

        let mut capabilities = HashSet::new();
        capabilities.insert(FileSystemCapability::SupportsSymlinks);
        capabilities.insert(FileSystemCapability::SupportsExtendedAttributes);

        #[cfg(target_os = "macos")]
        {
            capabilities.insert(FileSystemCapability::SupportsEvents);
        }

        Ok(FileSystemInfo {
            fs_type,
            capabilities,
            mount_point: PathBuf::from("/"),
            total_bytes: 0,
            free_bytes: 0,
            is_network: false,
            is_removable: false,
        })
    }

    fn get_fs_type(&self, _path: &Path) -> Result<FileSystemType> {
        // TODO: Implement proper detection
        #[cfg(target_os = "macos")]
        {
            Ok(FileSystemType::Apfs)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(FileSystemType::Unknown("unknown".to_string()))
        }
    }
}
