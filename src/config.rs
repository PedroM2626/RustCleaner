use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use log::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub use_trash: bool,
    pub include_hidden_files: bool,
    pub follow_symlinks: bool,
    pub min_file_size: u64,
    pub max_file_age_days: u32,
    pub excluded_paths: Vec<PathBuf>,
    pub excluded_extensions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_trash: true,
            include_hidden_files: false,
            follow_symlinks: false,
            min_file_size: 0,
            max_file_age_days: 365,
            excluded_paths: vec![
                // System directories that should never be cleaned
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/usr/bin"),
                PathBuf::from("/usr/sbin"),
                PathBuf::from("/System"),
                PathBuf::from("C:\\Windows"),
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
            ],
            excluded_extensions: vec![
                // Critical system files
                ".exe".to_string(),
                ".dll".to_string(),
                ".sys".to_string(),
                ".ini".to_string(),
                ".cfg".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("intelligent-disk-cleaner").join("config.json");
            
            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => {
                        match serde_json::from_str(&content) {
                            Ok(config) => {
                                info!("Loaded configuration from: {}", config_path.display());
                                return config;
                            }
                            Err(e) => {
                                error!("Failed to parse config file: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read config file: {}", e);
                    }
                }
            }
        }
        
        info!("Using default configuration");
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("intelligent-disk-cleaner");
            std::fs::create_dir_all(&app_config_dir)?;
            
            let config_path = app_config_dir.join("config.json");
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(&config_path, content)?;
            
            info!("Saved configuration to: {}", config_path.display());
        }
        
        Ok(())
    }

    pub fn is_path_excluded(&self, path: &PathBuf) -> bool {
        // Check if path is in excluded paths
        for excluded in &self.excluded_paths {
            if path.starts_with(excluded) {
                return true;
            }
        }
        
        // Check if file extension is excluded
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            for excluded_ext in &self.excluded_extensions {
                if format!(".{}", ext_str) == excluded_ext.to_lowercase() {
                    return true;
                }
            }
        }
        
        false
    }
}
