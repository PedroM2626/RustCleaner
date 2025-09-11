use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Read;
use blake3::Hasher;
use rayon::prelude::*;
use log::{info, warn};

use crate::progress::{ProgressTracker, ProgressState};

pub struct DuplicateFinder {
    hash_cache: HashMap<PathBuf, String>,
}

impl DuplicateFinder {
    pub fn new() -> Self {
        Self {
            hash_cache: HashMap::new(),
        }
    }

    pub fn find_duplicates(
        &mut self,
        files: &[PathBuf],
        progress: Arc<Mutex<ProgressTracker>>,
    ) -> Result<Vec<Vec<PathBuf>>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting duplicate detection for {} files", files.len());

        // First pass: group by file size
        let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        
        for file in files {
            if let Ok(metadata) = std::fs::metadata(file) {
                size_groups.entry(metadata.len()).or_insert_with(Vec::new).push(file.clone());
            }
        }

        // Filter groups with only one file (no duplicates possible)
        let potential_duplicates: Vec<_> = size_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .flat_map(|(_, files)| files)
            .collect();

        info!("Found {} files with matching sizes", potential_duplicates.len());

        if potential_duplicates.is_empty() {
            return Ok(Vec::new());
        }

        // Second pass: compute hashes for files with matching sizes
        let hash_map: Arc<Mutex<HashMap<String, Vec<PathBuf>>>> = Arc::new(Mutex::new(HashMap::new()));
        let processed_count = Arc::new(Mutex::new(0usize));
        let total_files = potential_duplicates.len();

        potential_duplicates.par_iter().for_each(|file_path| {
            // Update progress
            {
                let mut count = processed_count.lock().unwrap();
                *count += 1;
                let current_count = *count;
                
                if current_count % 10 == 0 || current_count == total_files {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.state = ProgressState::FindingDuplicates {
                        files_processed: current_count,
                        total_files,
                    };
                }
            }

            match self.calculate_file_hash(file_path) {
                Ok(hash) => {
                    let mut hash_groups = hash_map.lock().unwrap();
                    hash_groups.entry(hash).or_insert_with(Vec::new).push(file_path.clone());
                }
                Err(e) => {
                    warn!("Failed to hash file {}: {}", file_path.display(), e);
                }
            }
        });

        // Extract duplicate groups (groups with more than one file)
        let hash_groups = hash_map.lock().unwrap();
        let duplicates: Vec<Vec<PathBuf>> = hash_groups
            .values()
            .filter(|group| group.len() > 1)
            .cloned()
            .collect();

        info!("Found {} groups of duplicate files", duplicates.len());

        Ok(duplicates)
    }

    fn calculate_file_hash(&self, file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut file = File::open(file_path)?;
        let mut hasher = Hasher::new();
        
        // Read file in chunks to handle large files efficiently
        let mut buffer = [0; 8192];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }
}
