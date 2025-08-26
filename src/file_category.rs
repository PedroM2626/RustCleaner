use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    TemporaryFiles,
    CacheFiles,
    LogFiles,
    BrowserData,
    SystemJunk,
    EmptyFolders,
    LargeFiles,
    OldFiles,
    Downloads,
    RecycleBin,
}

impl FileCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::TemporaryFiles,
            Self::CacheFiles,
            Self::LogFiles,
            Self::BrowserData,
            Self::SystemJunk,
            Self::EmptyFolders,
            Self::LargeFiles,
            Self::OldFiles,
            Self::Downloads,
            Self::RecycleBin,
        ]
    }

    pub fn categorize(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();
        let filename = path.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Check file extension
        let extension = path.extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Temporary files
        if extension == "tmp" || extension == "temp" || 
           filename.starts_with("~") || filename.starts_with(".#") ||
           path_str.contains("/tmp/") || path_str.contains("\\temp\\") ||
           path_str.contains("/var/tmp/") || path_str.contains("\\windows\\temp\\") {
            return Self::TemporaryFiles;
        }

        // Cache files
        if path_str.contains("cache") || path_str.contains(".cache") ||
           extension == "cache" || 
           path_str.contains("/var/cache/") || path_str.contains("\\appdata\\local\\") {
            return Self::CacheFiles;
        }

        // Log files
        if extension == "log" || extension == "out" || extension == "err" ||
           filename.ends_with(".log") || filename.ends_with(".out") ||
           path_str.contains("/var/log/") || path_str.contains("\\logs\\") {
            return Self::LogFiles;
        }

        // Browser data
        if path_str.contains("browser") || path_str.contains("firefox") ||
           path_str.contains("chrome") || path_str.contains("safari") ||
           path_str.contains("cookies") || path_str.contains("history") ||
           extension == "sqlite" && path_str.contains("mozilla") {
            return Self::BrowserData;
        }

        // Downloads
        if path_str.contains("download") || path_str.contains("downloads") {
            return Self::Downloads;
        }

        // Recycle bin / Trash
        if path_str.contains("recycle") || path_str.contains("trash") ||
           path_str.contains(".trash") || path_str.contains("$recycle.bin") {
            return Self::RecycleBin;
        }

        // System junk
        if extension == "bak" || extension == "old" || extension == "backup" ||
           filename.starts_with("core.") || filename == "thumbs.db" ||
           filename == ".ds_store" || filename == "desktop.ini" {
            return Self::SystemJunk;
        }

        // Large files (over 100MB)
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > 100 * 1024 * 1024 {
                return Self::LargeFiles;
            }
        }

        // Old files (over 30 days)
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.elapsed() {
                    let age_days = duration.as_secs() / (24 * 60 * 60);
                    if age_days > 30 {
                        return Self::OldFiles;
                    }
                }
            }
        }

        // Default category
        Self::SystemJunk
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::TemporaryFiles => "Temporary files that can be safely deleted",
            Self::CacheFiles => "Application cache files",
            Self::LogFiles => "Log files from applications and system",
            Self::BrowserData => "Browser cache, cookies, and temporary data",
            Self::SystemJunk => "System junk files and backups",
            Self::EmptyFolders => "Empty directories",
            Self::LargeFiles => "Large files over 100MB",
            Self::OldFiles => "Files older than 30 days",
            Self::Downloads => "Files in download directories",
            Self::RecycleBin => "Files in trash/recycle bin",
        }
    }

    pub fn is_safe_to_delete(&self) -> bool {
        match self {
            Self::TemporaryFiles | Self::CacheFiles | Self::LogFiles | 
            Self::SystemJunk | Self::EmptyFolders | Self::RecycleBin => true,
            Self::BrowserData | Self::LargeFiles | Self::OldFiles | Self::Downloads => false,
        }
    }
}
