use std::path::PathBuf;
use crate::scanner::ScanResult;

#[derive(Debug, Clone)]
pub enum ProgressState {
    Idle,
    Scanning {
        current_path: String,
        files_processed: usize,
    },
    FindingDuplicates {
        files_processed: usize,
        total_files: usize,
    },
    Cleaning {
        files_processed: usize,
        total_files: usize,
    },
    Complete {
        scan_result: Option<ScanResult>,
        duplicates: Option<Vec<Vec<PathBuf>>>,
        cleaned_bytes: Option<u64>,
    },
    Error(String),
}

#[derive(Debug)]
pub struct ProgressTracker {
    pub state: ProgressState,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self {
            state: ProgressState::Idle,
        }
    }
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.state = ProgressState::Idle;
    }

    pub fn set_scan_complete(&mut self, result: ScanResult) {
        self.state = ProgressState::Complete {
            scan_result: Some(result),
            duplicates: None,
            cleaned_bytes: None,
        };
    }

    pub fn set_duplicates_complete(&mut self, duplicates: Vec<Vec<PathBuf>>) {
        self.state = ProgressState::Complete {
            scan_result: None,
            duplicates: Some(duplicates),
            cleaned_bytes: None,
        };
    }

    pub fn set_cleanup_complete(&mut self, cleaned_bytes: u64) {
        self.state = ProgressState::Complete {
            scan_result: None,
            duplicates: None,
            cleaned_bytes: Some(cleaned_bytes),
        };
    }

    pub fn set_error(&mut self, error: String) {
        self.state = ProgressState::Error(error);
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self.state,
            ProgressState::Scanning { .. } | 
            ProgressState::FindingDuplicates { .. } | 
            ProgressState::Cleaning { .. }
        )
    }
}
