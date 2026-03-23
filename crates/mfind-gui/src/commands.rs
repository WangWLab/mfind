//! Tauri commands for mfind GUI

use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::{IndexEngine, QueryParser};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{State, Manager};

/// Shared state for the GUI application
pub struct GuiState {
    pub engine: Arc<RwLock<IndexEngine>>,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub query_time_ms: f64,
}

/// Stats response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_bytes: u64,
}

/// Build index response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIndexResponse {
    pub total_files: u64,
    pub total_dirs: u64,
    pub build_time_ms: f64,
}

/// File preview response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreviewResponse {
    pub path: String,
    pub r#type: String, // text, image, binary
    pub content: Option<String>, // for text files
    pub data_uri: Option<String>, // for images
    pub size: u64,
    pub mime: String,
}

/// Search command - called from frontend
#[tauri::command]
pub async fn search(
    pattern: String,
    limit: Option<usize>,
    state: State<'_, GuiState>,
) -> Result<SearchResponse, String> {
    let start = std::time::Instant::now();

    let engine = state.engine.read().await;
    let limit = limit.unwrap_or(100);

    // Parse query
    let query = QueryParser::parse(&pattern).map_err(|e| e.to_string())?;

    // Execute search
    let result = engine.search(&query).map_err(|e| e.to_string())?;

    let results: Vec<SearchResultItem> = result.matches
        .into_iter()
        .take(limit)
        .map(|path| {
            let name = std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            SearchResultItem {
                path,
                name,
                is_dir: false,
                size: None,
            }
        })
        .collect();

    let total = results.len();
    let query_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(SearchResponse {
        results,
        total,
        query_time_ms,
    })
}

/// Get index statistics
#[tauri::command]
pub async fn get_stats(state: State<'_, GuiState>) -> Result<StatsResponse, String> {
    let engine = state.engine.read().await;
    let stats = engine.stats();

    Ok(StatsResponse {
        total_files: stats.total_files,
        total_dirs: stats.total_dirs,
        total_bytes: stats.total_bytes,
    })
}

/// Build index from paths
#[tauri::command]
pub async fn build_index(
    paths: Vec<String>,
    state: State<'_, GuiState>,
) -> Result<BuildIndexResponse, String> {
    let start = std::time::Instant::now();

    let mut engine = state.engine.write().await;
    let root_paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    engine.build(&root_paths).await.map_err(|e| e.to_string())?;

    let stats = engine.stats();
    let build_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(BuildIndexResponse {
        total_files: stats.total_files,
        total_dirs: stats.total_dirs,
        build_time_ms,
    })
}

/// Get file preview (text or image)
#[tauri::command]
pub async fn get_file_preview(path: String) -> Result<FilePreviewResponse, String> {
    use std::fs;
    use std::path::Path;

    let path_obj = Path::new(&path);

    // Get file metadata
    let metadata = fs::metadata(&path_obj)
        .map_err(|e| format!("无法访问文件：{}", e))?;

    let size = metadata.len();
    let extension = path_obj.extension()
        .map(|e| e.to_string_lossy().to_string().to_lowercase())
        .unwrap_or_default();

    // Determine file type
    let mime_type = get_mime_type(&extension);
    let (file_type, content, data_uri) = if is_text_file(&extension) {
        // Read text file
        let content = fs::read_to_string(&path_obj)
            .map_err(|e| format!("无法读取文本文件：{}", e))?;
        ("text".to_string(), Some(content), None)
    } else if is_image_file(&extension) {
        // Read image and convert to data URI
        let data = fs::read(&path_obj)
            .map_err(|e| format!("无法读取图片：{}", e))?;
        let base64 = base64_encode(&data);
        let data_uri = format!("data:{};base64,{}", mime_type, base64);
        ("image".to_string(), None, Some(data_uri))
    } else {
        // Binary file - no preview
        ("binary".to_string(), None, None)
    };

    Ok(FilePreviewResponse {
        path,
        r#type: file_type,
        content,
        data_uri,
        size,
        mime: mime_type,
    })
}

/// Open file in Finder (macOS only)
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn open_in_finder(path: String) -> Result<(), String> {
    use std::process::Command;

    Command::new("open")
        .arg("-R")
        .arg(&path)
        .output()
        .map_err(|e| format!("无法在 Finder 中显示：{}", e))?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn open_in_finder(path: String) -> Result<(), String> {
    Err("仅在 macOS 上支持此功能".to_string())
}

/// Toggle window visibility
#[tauri::command]
pub fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(visible) = window.is_visible() {
            if visible {
                window.hide().map_err(|e| e.to_string())?;
            } else {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
    }
    Err("无法找到主窗口".to_string())
}

/// Hide the main window
#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("无法找到主窗口".to_string())
    }
}

/// Show and focus the main window
#[tauri::command]
pub fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("无法找到主窗口".to_string())
    }
}

// Helper functions
pub fn is_text_file(ext: &str) -> bool {
    matches!(ext,
        "txt" | "md" | "rs" | "toml" | "json" | "yaml" | "yml" | "xml" | "html" | "htm" |
        "css" | "js" | "ts" | "jsx" | "tsx" | "py" | "java" | "c" | "cpp" | "h" | "hpp" |
        "go" | "rb" | "php" | "sh" | "bash" | "zsh" | "fish" | "sql" | "log" | "csv" |
        "ini" | "conf" | "config" | "env" | "gitignore" | "gitattributes" | "makefile" |
        "cargo" | "lock" | "cfg" | "properties" | "rst" | "adoc" | "org" | "vim" | "lua"
    )
}

pub fn is_image_file(ext: &str) -> bool {
    matches!(ext,
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif"
    )
}

pub fn get_mime_type(ext: &str) -> String {
    match ext {
        "txt" => "text/plain".to_string(),
        "md" => "text/markdown".to_string(),
        "rs" => "text/rust".to_string(),
        "json" => "application/json".to_string(),
        "xml" => "application/xml".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "css" => "text/css".to_string(),
        "js" => "application/javascript".to_string(),
        "ts" => "application/typescript".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "bmp" => "image/bmp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "webp" => "image/webp".to_string(),
        "pdf" => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

// Simple base64 encoding (for small images)
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}
