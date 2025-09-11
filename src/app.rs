use egui::{Context, Ui, Vec2, Color32};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, error};
use dirs;

use crate::scanner::{Scanner, ScanResult};
use crate::duplicate_finder::DuplicateFinder;
use crate::cleaner::Cleaner;
use crate::config::Config;
use crate::file_category::FileCategory;
use crate::progress::{ProgressTracker, ProgressState};

#[derive(Default)]
pub struct DiskCleanerApp {
    config: Config,
    scan_path: String,
    scan_results: Option<ScanResult>,
    duplicates: Vec<Vec<PathBuf>>,
    selected_categories: HashMap<FileCategory, bool>,
    progress: Arc<Mutex<ProgressTracker>>,
    is_scanning: bool,
    is_cleaning: bool,
    cleaned_space: u64,
    show_settings: bool,
    show_duplicates: bool,
    confirmation_dialog: bool,
    files_to_delete: Vec<PathBuf>,
}

impl DiskCleanerApp {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.scan_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        
        // Initialize all categories as selected by default
        for category in FileCategory::all() {
            app.selected_categories.insert(category, true);
        }
        
        app
    }

    fn start_scan(&mut self) {
        if self.is_scanning {
            return;
        }

        let path = PathBuf::from(&self.scan_path);
        if !path.exists() {
            error!("Scan path does not exist: {}", self.scan_path);
            return;
        }

        self.is_scanning = true;
        self.scan_results = None;
        self.duplicates.clear();
        
        let progress = Arc::clone(&self.progress);
        progress.lock().unwrap().reset();
        
        let _scan_path = self.scan_path.clone();
        let config = self.config.clone();
        
        thread::spawn(move || {
            let mut scanner = Scanner::new(config);
            match scanner.scan(&path, progress.clone()) {
                Ok(results) => {
                    progress.lock().unwrap().set_scan_complete(results);
                }
                Err(e) => {
                    error!("Scan failed: {}", e);
                    progress.lock().unwrap().set_error(format!("Scan failed: {}", e));
                }
            }
        });
    }

    fn start_duplicate_scan(&mut self) {
        if let Some(ref results) = self.scan_results {
            let progress = Arc::clone(&self.progress);
            let files = results.files_by_category.values()
                .flatten()
                .cloned()
                .collect::<Vec<_>>();
            
            thread::spawn(move || {
                let mut finder = DuplicateFinder::new();
                match finder.find_duplicates(&files, progress.clone()) {
                    Ok(duplicates) => {
                        progress.lock().unwrap().set_duplicates_complete(duplicates);
                    }
                    Err(e) => {
                        error!("Duplicate scan failed: {}", e);
                        progress.lock().unwrap().set_error(format!("Duplicate scan failed: {}", e));
                    }
                }
            });
        }
    }

    fn prepare_cleanup(&mut self) {
        self.files_to_delete.clear();
        
        if let Some(ref results) = self.scan_results {
            for (category, selected) in &self.selected_categories {
                if *selected {
                    if let Some(files) = results.files_by_category.get(category) {
                        self.files_to_delete.extend(files.iter().cloned());
                    }
                }
            }
        }
        
        // Add selected duplicates (keep first file in each group)
        for duplicate_group in &self.duplicates {
            if duplicate_group.len() > 1 {
                self.files_to_delete.extend(duplicate_group.iter().skip(1).cloned());
            }
        }
        
        if !self.files_to_delete.is_empty() {
            self.confirmation_dialog = true;
        }
    }

    fn execute_cleanup(&mut self) {
        if self.files_to_delete.is_empty() {
            return;
        }

        self.is_cleaning = true;
        self.confirmation_dialog = false;
        
        let files = self.files_to_delete.clone();
        let progress = Arc::clone(&self.progress);
        let use_trash = self.config.use_trash;
        
        thread::spawn(move || {
            let cleaner = Cleaner::new(use_trash);
            match cleaner.clean_files(&files, progress.clone()) {
                Ok(cleaned_bytes) => {
                    progress.lock().unwrap().set_cleanup_complete(cleaned_bytes);
                }
                Err(e) => {
                    error!("Cleanup failed: {}", e);
                    progress.lock().unwrap().set_error(format!("Cleanup failed: {}", e));
                }
            }
        });
    }

    fn draw_scan_section(&mut self, ui: &mut Ui) {
        ui.heading("Disk Scanner");
        
        ui.horizontal(|ui| {
            ui.label("Scan Path:");
            ui.text_edit_singleline(&mut self.scan_path);
            
            if ui.button("Browse").clicked() {
                // In a real implementation, you'd use a file dialog here
                info!("File dialog would open here");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("Start Scan").clicked() && !self.is_scanning {
                self.start_scan();
            }
            
            if ui.button("Find Duplicates").clicked() && self.scan_results.is_some() {
                self.start_duplicate_scan();
            }
            
            if ui.button("Settings").clicked() {
                self.show_settings = !self.show_settings;
            }
        });
    }

    fn draw_progress(&self, ui: &mut Ui) {
        let progress = self.progress.lock().unwrap();
        
        match &progress.state {
            ProgressState::Idle => {},
            ProgressState::Scanning { current_path, files_processed } => {
                ui.label(format!("Scanning: {} files processed", files_processed));
                ui.label(format!("Current: {}", current_path));
                ui.add(egui::ProgressBar::new(0.5).show_percentage());
            },
            ProgressState::FindingDuplicates { files_processed, total_files } => {
                let progress_value = *files_processed as f32 / *total_files as f32;
                ui.label(format!("Finding duplicates: {}/{}", files_processed, total_files));
                ui.add(egui::ProgressBar::new(progress_value).show_percentage());
            },
            ProgressState::Cleaning { files_processed, total_files } => {
                let progress_value = *files_processed as f32 / *total_files as f32;
                ui.label(format!("Cleaning: {}/{}", files_processed, total_files));
                ui.add(egui::ProgressBar::new(progress_value).show_percentage());
            },
            ProgressState::Complete { .. } => {
                ui.label("Operation completed successfully");
            },
            ProgressState::Error(msg) => {
                ui.colored_label(Color32::RED, format!("Error: {}", msg));
            },
        }
    }

    fn draw_results(&mut self, ui: &mut Ui) {
        if let Some(ref results) = self.scan_results {
            ui.heading("Scan Results");
            
            // Summary
            ui.label(format!("Total files scanned: {}", results.total_files));
            ui.label(format!("Total size: {}", humansize::format_size(results.total_size, humansize::DECIMAL)));
            
            ui.separator();
            
            // Category breakdown
            ui.heading("File Categories");
            
            let mut total_selected_size = 0u64;
            
            for category in FileCategory::all() {
                if let Some(files) = results.files_by_category.get(&category) {
                    let category_size: u64 = files.iter()
                        .filter_map(|path| std::fs::metadata(path).ok())
                        .map(|metadata| metadata.len())
                        .sum();
                    
                    ui.horizontal(|ui| {
                        let mut selected = self.selected_categories.get(&category).copied().unwrap_or(false);
                        if ui.checkbox(&mut selected, "").changed() {
                            self.selected_categories.insert(category, selected);
                        }
                        
                        ui.label(format!("{:?}", category));
                        ui.label(format!("{} files", files.len()));
                        ui.label(format!("{}", humansize::format_size(category_size, humansize::DECIMAL)));
                    });
                    
                    if self.selected_categories.get(&category).copied().unwrap_or(false) {
                        total_selected_size += category_size;
                    }
                }
            }
            
            ui.separator();
            ui.label(format!("Selected for cleaning: {}", humansize::format_size(total_selected_size, humansize::DECIMAL)));
            
            ui.horizontal(|ui| {
                if ui.button("Clean Selected").clicked() {
                    self.prepare_cleanup();
                }
                
                if ui.button("View Duplicates").clicked() {
                    self.show_duplicates = !self.show_duplicates;
                }
            });
        }
    }

    fn draw_duplicates_window(&mut self, ctx: &Context) {
        if self.show_duplicates {
            egui::Window::new("Duplicate Files")
                .default_size(Vec2::new(600.0, 400.0))
                .show(ctx, |ui| {
                    if self.duplicates.is_empty() {
                        ui.label("No duplicate files found. Run duplicate scan first.");
                    } else {
                        ui.label(format!("Found {} groups of duplicate files", self.duplicates.len()));
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (group_idx, group) in self.duplicates.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.label(format!("Group {}: {} files", group_idx + 1, group.len()));
                                    
                                    for (file_idx, file) in group.iter().enumerate() {
                                        let color = if file_idx == 0 { 
                                            Color32::GREEN 
                                        } else { 
                                            Color32::RED 
                                        };
                                        
                                        ui.colored_label(color, file.to_string_lossy());
                                        
                                        if file_idx == 0 {
                                            ui.label("  (will be kept)");
                                        } else {
                                            ui.label("  (will be deleted)");
                                        }
                                    }
                                });
                                ui.separator();
                            }
                        });
                    }
                });
        }
    }

    fn draw_settings_window(&mut self, ctx: &Context) {
        if self.show_settings {
            egui::Window::new("Settings")
                .default_size(Vec2::new(400.0, 300.0))
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.config.use_trash, "Use Trash/Recycle Bin (safer)");
                    ui.checkbox(&mut self.config.include_hidden_files, "Include hidden files");
                    ui.checkbox(&mut self.config.follow_symlinks, "Follow symbolic links");
                    
                    ui.separator();
                    
                    ui.label("File size limits:");
                    ui.add(egui::Slider::new(&mut self.config.min_file_size, 0..=1000000)
                        .text("Minimum file size (bytes)")
                        .logarithmic(true));
                    
                    ui.add(egui::Slider::new(&mut self.config.max_file_age_days, 1..=365)
                        .text("Maximum file age (days)"));
                });
        }
    }

    fn draw_confirmation_dialog(&mut self, ctx: &Context) {
        if self.confirmation_dialog {
            egui::Window::new("Confirm Cleanup")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete {} files?", self.files_to_delete.len()));
                    
                    let total_size: u64 = self.files_to_delete.iter()
                        .filter_map(|path| std::fs::metadata(path).ok())
                        .map(|metadata| metadata.len())
                        .sum();
                    
                    ui.label(format!("Total size: {}", humansize::format_size(total_size, humansize::DECIMAL)));
                    
                    if self.config.use_trash {
                        ui.label("Files will be moved to trash (can be recovered)");
                    } else {
                        ui.colored_label(Color32::RED, "Files will be permanently deleted!");
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.confirmation_dialog = false;
                            self.files_to_delete.clear();
                        }
                        
                        if ui.button("Confirm Delete").clicked() {
                            self.execute_cleanup();
                        }
                    });
                });
        }
    }

    fn check_background_tasks(&mut self) {
        let mut progress = self.progress.lock().unwrap();
        
        match &progress.state {
            ProgressState::Complete { scan_result: Some(results), .. } => {
                let results = results.clone();
                self.scan_results = Some(results);
                self.is_scanning = false;
                progress.state = ProgressState::Idle;
            },
            ProgressState::Complete { duplicates: Some(duplicates), .. } => {
                let duplicates = duplicates.clone();
                self.duplicates = duplicates;
                progress.state = ProgressState::Idle;
            },
            ProgressState::Complete { cleaned_bytes: Some(bytes), .. } => {
                self.cleaned_space = *bytes;
                self.is_cleaning = false;
                self.files_to_delete.clear();
                progress.state = ProgressState::Idle;
                
                // Refresh scan results after cleanup
                if self.scan_results.is_some() {
                    drop(progress); // Release the lock before calling start_scan
                    self.start_scan();
                    return;
                }
            },
            ProgressState::Error(_) => {
                self.is_scanning = false;
                self.is_cleaning = false;
            },
            _ => {}
        }
    }
}

impl eframe::App for DiskCleanerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.check_background_tasks();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Intelligent Disk Cleaner");
            ui.separator();
            
            self.draw_scan_section(ui);
            ui.separator();
            
            self.draw_progress(ui);
            ui.separator();
            
            self.draw_results(ui);
            
            if self.cleaned_space > 0 {
                ui.separator();
                ui.colored_label(
                    Color32::GREEN,
                    format!("Successfully cleaned: {}", humansize::format_size(self.cleaned_space, humansize::DECIMAL))
                );
            }
        });
        
        self.draw_duplicates_window(ctx);
        self.draw_settings_window(ctx);
        self.draw_confirmation_dialog(ctx);
        
        // Request repaint for animations and progress updates
        if self.is_scanning || self.is_cleaning {
            ctx.request_repaint();
        }
    }
}
