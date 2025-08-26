use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};

use crate::progress::{ProgressTracker, ProgressState};

pub struct Cleaner {
    use_trash: bool,
}

impl Cleaner {
    pub fn new(use_trash: bool) -> Self {
        Self { use_trash }
    }

    pub fn clean_files(
        &self,
        files: &[PathBuf],
        progress: Arc<Mutex<ProgressTracker>>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting cleanup of {} files", files.len());

        let mut total_cleaned_bytes = 0u64;
        let total_files = files.len();

        for (index, file_path) in files.iter().enumerate() {
            // Update progress
            {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.state = ProgressState::Cleaning {
                    files_processed: index + 1,
                    total_files,
                };
            }

            // Get file size before deletion
            let file_size = match std::fs::metadata(file_path) {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    warn!("Could not get metadata for {}: {}", file_path.display(), e);
                    continue;
                }
            };

            // Attempt to delete the file
            let result = if self.use_trash {
                self.move_to_trash(file_path)
            } else {
                self.delete_permanently(file_path)
            };

            match result {
                Ok(()) => {
                    total_cleaned_bytes += file_size;
                    info!("Successfully cleaned: {} ({} bytes)", file_path.display(), file_size);
                }
                Err(e) => {
                    error!("Failed to clean {}: {}", file_path.display(), e);
                }
            }
        }

        info!("Cleanup completed. Total cleaned: {} bytes", total_cleaned_bytes);
        Ok(total_cleaned_bytes)
    }

    fn move_to_trash(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trash::delete(file_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn delete_permanently(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        std::fs::remove_file(file_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub fn estimate_cleanup_size(&self, files: &[PathBuf]) -> u64 {
        files
            .iter()
            .filter_map(|file| std::fs::metadata(file).ok())
            .map(|metadata| metadata.len())
            .sum()
    }
}
