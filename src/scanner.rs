use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;
use walkdir::WalkDir;
use rayon::prelude::*;
use log::{info, warn, debug};
use serde::{Serialize, Deserialize};

use crate::config::Config;
use crate::file_category::FileCategory;
use crate::progress::{ProgressTracker, ProgressState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: usize,
    pub total_size: u64,
    pub files_by_category: HashMap<FileCategory, Vec<PathBuf>>,
    pub scan_duration: std::time::Duration,
}

pub struct Scanner {
    config: Config,
}

impl Scanner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn scan(&mut self, path: &Path, progress: Arc<Mutex<ProgressTracker>>) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        info!("Starting scan of path: {}", path.display());

        let mut walker = WalkDir::new(path);
        
        if !self.config.follow_symlinks {
            walker = walker.follow_links(false);
        }

        let entries: Vec<_> = walker
            .into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(entry) => Some(entry),
                    Err(e) => {
                        warn!("Error accessing file: {}", e);
                        None
                    }
                }
            })
            .filter(|entry| entry.file_type().is_file())
            .collect();

        info!("Found {} files to process", entries.len());

        let files_by_category: Arc<Mutex<HashMap<FileCategory, Vec<PathBuf>>>> = 
            Arc::new(Mutex::new(HashMap::new()));
        let processed_count = Arc::new(Mutex::new(0usize));
        let total_files = entries.len();
        let total_size = Arc::new(Mutex::new(0u64));

        // Process files in parallel
        entries.par_iter().for_each(|entry| {
            let path = entry.path();
            
            // Update progress
            {
                let mut count = processed_count.lock().unwrap();
                *count += 1;
                let current_count = *count;
                
                if current_count % 100 == 0 || current_count == total_files {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.state = ProgressState::Scanning {
                        current_path: path.to_string_lossy().to_string(),
                        files_processed: current_count,
                    };
                }
            }

            // Check file filters
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();
                
                if file_size < self.config.min_file_size {
                    return;
                }

                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.elapsed() {
                        let age_days = duration.as_secs() / (24 * 60 * 60);
                        if age_days > self.config.max_file_age_days as u64 {
                            return;
                        }
                    }
                }

                // Check if hidden file
                if !self.config.include_hidden_files {
                    if let Some(filename) = path.file_name() {
                        if filename.to_string_lossy().starts_with('.') {
                            return;
                        }
                    }
                }

                // Categorize file
                let category = FileCategory::categorize(path);
                
                // Add to results
                {
                    let mut categories = files_by_category.lock().unwrap();
                    categories.entry(category).or_insert_with(Vec::new).push(path.to_owned());
                }

                // Update total size
                {
                    let mut size = total_size.lock().unwrap();
                    *size += file_size;
                }
            }
        });

        let scan_duration = start_time.elapsed();
        let final_categories = files_by_category.lock().unwrap().clone();
        let final_size = *total_size.lock().unwrap();
        let final_count = *processed_count.lock().unwrap();

        info!("Scan completed in {:?}", scan_duration);
        info!("Processed {} files, total size: {} bytes", final_count, final_size);

        Ok(ScanResult {
            total_files: final_count,
            total_size: final_size,
            files_by_category: final_categories,
            scan_duration,
        })
    }
}
