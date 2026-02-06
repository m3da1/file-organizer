use crate::error::{OrganizerError, Result};
use crate::tui::{PreviewApp, ProgressApp, SummaryApp};
use clap::Parser;
use colored::Colorize;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    collections::HashMap,
    fs,
    io,
    path::{Path, PathBuf},
    time::Instant,
};

/// File organizer - Automatically organize files into categorized folders
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct MyOrganizer {
    /// Path to organize
    pub path: PathBuf,

    /// Dry run - show what would be done without actually moving files
    #[arg(short, long)]
    pub dry_run: bool,

    /// Verbose output - show detailed information
    #[arg(short, long)]
    pub verbose: bool,

    /// Conflict resolution strategy: skip, overwrite, or rename
    #[arg(short, long, default_value = "skip", value_parser = ["skip", "overwrite", "rename"])]
    pub conflict: String,

    /// Recursive - organize files in subdirectories as well
    #[arg(short, long)]
    pub recursive: bool,

    /// Interactive mode - show TUI dashboard
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub mime_type: Option<String>,
    pub category: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct OrganizeStats {
    pub total_files: usize,
    pub moved: usize,
    pub skipped: usize,
    pub errors: usize,
}

impl OrganizeStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            moved: 0,
            skipped: 0,
            errors: 0,
        }
    }
}

pub fn organizer_files(args: MyOrganizer) -> Result<()> {
    let path = &args.path;

    // Validate path
    if !path.exists() {
        return Err(OrganizerError::PathNotFound(path.clone()));
    }
    if !path.is_dir() {
        return Err(OrganizerError::PathNotDirectory(path.clone()));
    }

    println!(
        "{} {} {}",
        "Organizing".bright_cyan().bold(),
        path.display().to_string().bright_yellow(),
        if args.dry_run {
            "(DRY RUN)".bright_magenta().bold()
        } else {
            "".clear()
        }
    );

    if args.verbose {
        println!("{}", "Configuration:".bright_cyan());
        println!("  Dry run: {}", args.dry_run);
        println!("  Verbose: {}", args.verbose);
        println!("  Conflict strategy: {}", args.conflict);
        println!("  Recursive: {}", args.recursive);
        println!();
    }

    // Scan directory and categorize files
    let files = scan_directory(path, args.recursive, args.verbose)?;

    if files.is_empty() {
        println!("{}", "No files to organize".bright_yellow());
        return Ok(());
    }

    // Interactive mode with TUI
    if args.interactive {
        return run_interactive_mode(files, path, &args);
    }

    // Create progress bar
    let pb = if args.dry_run {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );
        pb
    };

    // In dry-run mode, show table header
    if args.dry_run {
        println!();
        println!(
            "{:<50} {:<12} {:<15} {}",
            "File".bright_cyan().bold(),
            "Size".bright_cyan().bold(),
            "Category".bright_cyan().bold(),
            "MIME Type".bright_cyan().bold()
        );
        println!("{}", "─".repeat(100).bright_black());
    }

    // Move files
    let mut stats = OrganizeStats::new();
    stats.total_files = files.len();

    for file_info in files {
        if !args.dry_run {
            pb.set_message(format!(
                "Processing: {}",
                file_info
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));
        }

        match move_file(&file_info, path, &args) {
            Ok(moved) => {
                if moved {
                    stats.moved += 1;
                    if args.dry_run {
                        let filename = file_info
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let size_str = format_size(file_info.size);
                        let mime_str = file_info
                            .mime_type
                            .as_deref()
                            .unwrap_or("unknown")
                            .to_string();

                        println!(
                            "{:<50} {:<12} {:<15} {}",
                            truncate_str(&filename, 48),
                            size_str.bright_yellow(),
                            file_info.category.bright_cyan(),
                            truncate_str(&mime_str, 40).bright_black()
                        );
                    } else if args.verbose {
                        pb.println(format!(
                            "  {} {} -> {}",
                            "✓".bright_green(),
                            file_info.path.display(),
                            file_info.category.bright_cyan()
                        ));
                    }
                } else {
                    stats.skipped += 1;
                    if args.verbose {
                        pb.println(format!(
                            "  {} {} (already exists)",
                            "⊘".bright_yellow(),
                            file_info.path.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    }
                }
            }
            Err(e) => {
                stats.errors += 1;
                let msg = format!(
                    "  {} {} - {}",
                    "✗".bright_red(),
                    file_info.path.display(),
                    e.to_string().bright_red()
                );
                if args.dry_run {
                    println!("{}", msg);
                } else {
                    pb.println(msg);
                }
            }
        }

        if !args.dry_run {
            pb.inc(1);
        }
    }

    if !args.dry_run {
        pb.finish_with_message("Done!");
    }

    // Print summary
    println!();
    println!("{}", "Summary:".bright_cyan().bold());
    println!("  Total files: {}", stats.total_files);
    println!(
        "  {} {}",
        "Moved:".bright_green(),
        stats.moved.to_string().bright_green().bold()
    );
    if stats.skipped > 0 {
        println!(
            "  {} {}",
            "Skipped:".bright_yellow(),
            stats.skipped.to_string().bright_yellow().bold()
        );
    }
    if stats.errors > 0 {
        println!(
            "  {} {}",
            "Errors:".bright_red(),
            stats.errors.to_string().bright_red().bold()
        );
    }

    Ok(())
}

fn run_interactive_mode(files: Vec<FileInfo>, base_path: &Path, args: &MyOrganizer) -> Result<()> {
    if args.dry_run {
        // Show preview dashboard
        let mut app = PreviewApp::new(files);
        app.run().map_err(|e| OrganizerError::IoError(e))?;

        if app.should_quit {
            println!("{}", "Operation cancelled".bright_yellow());
        } else {
            println!("{}", "Preview mode only - no files were moved".bright_yellow());
        }
        return Ok(());
    }

    // Setup TUI for progress
    enable_raw_mode().map_err(|e| OrganizerError::IoError(e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| OrganizerError::IoError(e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| OrganizerError::IoError(e))?;

    let mut progress_app = ProgressApp::new(files.len());
    progress_app.stats.total_files = files.len();
    let mut total_size_moved = 0u64;

    // Start timer
    let start_time = Instant::now();
    let mut last_render = Instant::now();
    let render_interval = std::time::Duration::from_millis(16); // ~60 FPS

    // Process files
    for (index, file_info) in files.iter().enumerate() {
        progress_app.update_current(file_info);

        // Process the file
        match move_file(file_info, base_path, args) {
            Ok(moved) => {
                if moved {
                    progress_app.stats.moved += 1;
                    progress_app.update_category(&file_info.category, file_info.size);
                    total_size_moved += file_info.size;
                } else {
                    progress_app.stats.skipped += 1;
                }
            }
            Err(_) => {
                progress_app.stats.errors += 1;
            }
        }

        // Render at intervals or for the last file to ensure we see 100%
        let should_render = last_render.elapsed() >= render_interval || index == files.len() - 1;
        if should_render {
            terminal
                .draw(|f| progress_app.render(f))
                .map_err(|e| OrganizerError::IoError(e))?;
            last_render = Instant::now();
        }
    }

    // Final render to ensure 100% is visible
    terminal
        .draw(|f| progress_app.render(f))
        .map_err(|e| OrganizerError::IoError(e))?;

    // Show completion for a moment
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Restore terminal before showing summary
    disable_raw_mode().map_err(|e| OrganizerError::IoError(e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| OrganizerError::IoError(e))?;
    terminal.show_cursor().map_err(|e| OrganizerError::IoError(e))?;

    // Calculate elapsed time
    let elapsed_time = start_time.elapsed();

    // Clone stats before passing to summary app
    let stats_clone = progress_app.stats.clone();
    let category_progress_clone = progress_app.category_progress.clone();

    // Show comprehensive summary screen
    let summary_app = SummaryApp::new(
        progress_app.stats,
        progress_app.category_progress,
        elapsed_time,
        total_size_moved,
    );

    summary_app.run().map_err(|e| OrganizerError::IoError(e))?;

    // Print text summary to terminal for permanent record
    println!();
    println!("{}", "═".repeat(60).bright_cyan());
    println!("{}", "Organization Complete".bright_green().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    println!();

    println!("{}", "Summary:".bright_cyan().bold());
    println!("  {} {}", "Total files:".bright_white(), stats_clone.total_files.to_string().bright_yellow());
    println!("  {} {}", "✓ Moved:".bright_green(), stats_clone.moved.to_string().bright_green().bold());
    println!("  {} {}", "⊘ Skipped:".bright_yellow(), stats_clone.skipped.to_string().bright_yellow());
    println!("  {} {}", "✗ Errors:".bright_red(), stats_clone.errors.to_string().bright_red());

    let success_rate = if stats_clone.total_files > 0 {
        (stats_clone.moved as f64 / stats_clone.total_files as f64) * 100.0
    } else {
        0.0
    };
    println!("  {} {:.1}%", "Success rate:".bright_white(), success_rate);
    println!();

    println!("{}", "Performance:".bright_cyan().bold());
    println!("  {} {:.2}s", "Time elapsed:".bright_white(), elapsed_time.as_secs_f64());
    println!("  {} {}", "Data moved:".bright_white(), format_size(total_size_moved).bright_yellow());

    if elapsed_time.as_secs() > 0 {
        let speed_mbs = total_size_moved as f64 / elapsed_time.as_secs_f64() / 1_048_576.0;
        let files_per_sec = stats_clone.moved as f64 / elapsed_time.as_secs_f64();
        println!("  {} {:.2} MB/s", "Speed:".bright_white(), speed_mbs);
        println!("  {} {:.1} files/s", "Throughput:".bright_white(), files_per_sec);
    }
    println!();

    println!("{}", "Categories:".bright_cyan().bold());
    for (category, progress) in &category_progress_clone {
        if progress.count > 0 {
            println!("  {} {} files ({})",
                category.bright_white(),
                progress.count.to_string().bright_yellow(),
                format_size(progress.size).bright_cyan()
            );
        }
    }
    println!();
    println!("{}", "═".repeat(60).bright_cyan());

    Ok(())
}

fn scan_directory(dir: &Path, recursive: bool, verbose: bool) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    if verbose {
        println!("{}", "Scanning directory...".bright_cyan());
    }

    scan_directory_recursive(dir, dir, recursive, &mut files)?;

    if verbose {
        println!(
            "Found {} files\n",
            files.len().to_string().bright_green().bold()
        );
    }

    Ok(files)
}

fn scan_directory_recursive(
    base_dir: &Path,
    current_dir: &Path,
    recursive: bool,
    files: &mut Vec<FileInfo>,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            let mime_type = mime_guess::from_path(&path).first().map(|m| m.to_string());
            let category = categorize_file(&mime_type);
            let size = metadata.len();

            files.push(FileInfo {
                path,
                mime_type,
                category,
                size,
            });
        } else if metadata.is_dir() && recursive {
            // Don't recurse into category folders we create
            if let Some(dir_name) = path.file_name() {
                let dir_str = dir_name.to_string_lossy();
                if !["Multimedia", "Docs", "Compressed", "Misc"].contains(&dir_str.as_ref())
                {
                    scan_directory_recursive(base_dir, &path, recursive, files)?;
                }
            }
        }
    }

    Ok(())
}

pub fn categorize_file(mime_type: &Option<String>) -> String {
    let mime_categories: HashMap<&str, &str> = [
        // Images
        ("image/png", "Multimedia"),
        ("image/jpeg", "Multimedia"),
        ("image/jpg", "Multimedia"),
        ("image/gif", "Multimedia"),
        ("image/webp", "Multimedia"),
        ("image/svg+xml", "Multimedia"),
        ("image/bmp", "Multimedia"),
        ("image/tiff", "Multimedia"),
        ("image/x-icon", "Multimedia"),
        // Audio
        ("audio/mpeg", "Multimedia"),
        ("audio/ogg", "Multimedia"),
        ("audio/wav", "Multimedia"),
        ("audio/webm", "Multimedia"),
        ("audio/aac", "Multimedia"),
        ("audio/flac", "Multimedia"),
        ("audio/x-m4a", "Multimedia"),
        // Video
        ("video/mp4", "Multimedia"),
        ("video/mpeg", "Multimedia"),
        ("video/ogg", "Multimedia"),
        ("video/webm", "Multimedia"),
        ("video/x-msvideo", "Multimedia"),
        ("video/x-matroska", "Multimedia"),
        ("video/quicktime", "Multimedia"),
        // Archives
        ("application/zip", "Compressed"),
        ("application/x-rar-compressed", "Compressed"),
        ("application/x-7z-compressed", "Compressed"),
        ("application/gzip", "Compressed"),
        ("application/x-tar", "Compressed"),
        ("application/x-bzip", "Compressed"),
        ("application/x-bzip2", "Compressed"),
        ("application/x-xz", "Compressed"),
        // Documents
        (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "Docs",
        ),
        (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "Docs",
        ),
        (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "Docs",
        ),
        ("application/vnd.ms-excel", "Docs"),
        ("application/vnd.ms-powerpoint", "Docs"),
        ("application/msword", "Docs"),
        ("application/pdf", "Docs"),
        ("text/html", "Docs"),
        ("text/css", "Misc"),
        ("text/csv", "Docs"),
        ("text/xml", "Docs"),
        ("application/xml", "Docs"),
        ("text/plain", "Docs"),
        ("text/markdown", "Docs"),
        ("application/json", "Misc"),
        ("application/rtf", "Docs"),
        // Code files (categorized as Misc)
        ("text/x-python", "Misc"),
        ("text/x-java", "Misc"),
        ("text/x-c", "Misc"),
        ("text/x-c++", "Misc"),
        ("text/x-rust", "Misc"),
        ("text/javascript", "Misc"),
        ("application/javascript", "Misc"),
        ("application/typescript", "Misc"),
        ("text/x-go", "Misc"),
        ("text/x-php", "Misc"),
        ("text/x-ruby", "Misc"),
        ("text/x-shellscript", "Misc"),
    ]
    .iter()
    .cloned()
    .collect();

    match mime_type {
        Some(mt) => mime_categories
            .get(mt.as_str())
            .copied()
            .unwrap_or("Misc")
            .to_string(),
        None => "Misc".to_string(),
    }
}

fn move_file(file_info: &FileInfo, base_path: &Path, args: &MyOrganizer) -> Result<bool> {
    let category_dir = base_path.join(&file_info.category);

    // Create category directory if it doesn't exist
    if !args.dry_run && !category_dir.exists() {
        fs::create_dir(&category_dir)?;
    }

    let file_name = file_info
        .path
        .file_name()
        .ok_or_else(|| OrganizerError::InvalidPath("No filename".to_string()))?;

    let destination = category_dir.join(file_name);

    // Handle conflicts
    if destination.exists() {
        match args.conflict.as_str() {
            "skip" => return Ok(false),
            "overwrite" => {
                if !args.dry_run {
                    fs::remove_file(&destination)?;
                }
            }
            "rename" => {
                let new_dest = generate_unique_filename(&destination);
                if !args.dry_run {
                    fs::rename(&file_info.path, &new_dest)?;
                }
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }

    // Move the file
    if !args.dry_run {
        fs::rename(&file_info.path, &destination)?;
    }

    Ok(true)
}

pub fn generate_unique_filename(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap();
    let stem = path.file_stem().unwrap().to_string_lossy();
    let extension = path.extension().map(|e| e.to_string_lossy());

    let mut counter = 1;
    loop {
        let new_name = match &extension {
            Some(ext) => format!("{}_{}.{}", stem, counter, ext),
            None => format!("{}_{}", stem, counter),
        };

        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
