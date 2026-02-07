use crate::cli::{FileInfo, OrganizeStats};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Padding, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    io,
    time::Duration,
};

pub struct PreviewApp {
    pub files: Vec<FileInfo>,
    pub total_size: u64,
    pub should_quit: bool,
    pub selected_category: Option<usize>,
    pub scroll_offset: usize,
    pub categories: Vec<String>,
}

impl PreviewApp {
    pub fn new(files: Vec<FileInfo>) -> Self {
        let total_size = files.iter().map(|f| f.size).sum();
        let categories = vec![
            "Multimedia".to_string(),
            "Docs".to_string(),
            "Compressed".to_string(),
            "Misc".to_string(),
        ];
        Self {
            files,
            total_size,
            should_quit: false,
            selected_category: None,
            scroll_offset: 0,
            categories,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the app
        let res = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        let mut last_was_esc_back = false; // Track if we just went back with ESC

        loop {
            terminal.draw(|f| self.render_preview(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Reset the ESC tracking flag if we get any key that's not ESC
                    if !matches!(key.code, KeyCode::Esc) {
                        last_was_esc_back = false;
                    }

                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                            break;
                        }
                        KeyCode::Esc => {
                            if self.selected_category.is_some() {
                                // Go back to overview
                                self.selected_category = None;
                                self.scroll_offset = 0;
                                last_was_esc_back = true;
                                // Force a redraw and skip the next ESC if it comes too quickly
                                continue;
                            } else if !last_was_esc_back {
                                // Only quit if the last action wasn't ESC going back
                                self.should_quit = true;
                                break;
                            }
                            // If last_was_esc_back is true, ignore this ESC (key repeat/held)
                            last_was_esc_back = false;
                        }
                        KeyCode::Enter => {
                            if self.selected_category.is_none() {
                                break; // Proceed to organize
                            }
                        }
                        KeyCode::Left | KeyCode::Up => {
                            if self.selected_category.is_none() {
                                // Navigate categories (not implemented in overview, but we could)
                            } else {
                                // Scroll up in detail view
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                            }
                        }
                        KeyCode::Right | KeyCode::Down => {
                            if self.selected_category.is_none() {
                                // Navigate categories (not implemented in overview, but we could)
                            } else {
                                // Scroll down in detail view
                                self.scroll_offset = self.scroll_offset.saturating_add(1);
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if self.selected_category.is_none() {
                                let digit = c.to_digit(10).unwrap() as usize;
                                if digit > 0 && digit <= self.categories.len() {
                                    self.selected_category = Some(digit - 1);
                                    self.scroll_offset = 0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn render_preview(&self, f: &mut Frame) {
        if let Some(category_idx) = self.selected_category {
            // Detail view for selected category
            self.render_category_detail(f, category_idx);
        } else {
            // Overview with all categories
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Header
            self.render_header(f, chunks[0]);

            // Categories grid
            self.render_categories(f, chunks[1]);

            // Footer
            self.render_footer(f, chunks[2]);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "File Organizer v0.2.0",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  |  "),
                Span::styled(
                    format!("{} files", self.files.len()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  |  "),
                Span::styled(
                    format_size(self.total_size),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Preview ")
                .title_alignment(Alignment::Center),
        );

        f.render_widget(title, area);
    }

    fn render_categories(&self, f: &mut Frame, area: Rect) {
        // Group files by category
        let mut categories: HashMap<String, Vec<&FileInfo>> = HashMap::new();
        for file in &self.files {
            categories
                .entry(file.category.clone())
                .or_insert_with(Vec::new)
                .push(file);
        }

        // Create grid layout (2 rows, 2 columns for 4 categories)
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(rows[1]);

        // Render each category
        let category_names = vec!["Multimedia", "Docs", "Compressed", "Misc"];
        let areas = vec![top_cols[0], top_cols[1], bottom_cols[0], bottom_cols[1]];

        for (idx, cat_name) in category_names.iter().enumerate() {
            if idx < areas.len() {
                self.render_category_box(f, areas[idx], cat_name, idx, categories.get(*cat_name));
            }
        }
    }

    fn render_category_detail(&self, f: &mut Frame, category_idx: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.area());

        let category_name = &self.categories[category_idx];

        // Group files by category
        let category_files: Vec<&FileInfo> = self
            .files
            .iter()
            .filter(|f| &f.category == category_name)
            .collect();

        let total_size: u64 = category_files.iter().map(|f| f.size).sum();

        // Header
        let color = match category_name.as_str() {
            "Multimedia" => Color::Magenta,
            "Docs" => Color::Blue,
            "Compressed" => Color::Yellow,
            _ => Color::White,
        };

        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("{} Category", category_name),
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  |  "),
                Span::styled(
                    format!("{} files", category_files.len()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  |  "),
                Span::styled(
                    format_size(total_size),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        );

        f.render_widget(header, chunks[0]);

        // File list
        let available_height = chunks[1].height.saturating_sub(2) as usize; // Subtract borders
        let visible_start = self.scroll_offset;
        let visible_end = (visible_start + available_height).min(category_files.len());

        let items: Vec<ListItem> = category_files
            .iter()
            .skip(visible_start)
            .take(available_height)
            .map(|file| {
                let filename = file
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                // Calculate max filename width: width - padding (4) - size (12) - borders (2) - spacing (2)
                let max_filename_width = chunks[1].width.saturating_sub(20) as usize;
                let truncated = truncate_str(&filename, max_filename_width);

                // Pad filename to fixed width for alignment
                let padded_filename = format!("{:<width$}", truncated, width = max_filename_width);
                let size_str = format!("{:>12}", format_size(file.size));

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(padded_filename, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(size_str, Style::default().fg(Color::Yellow)),
                ]))
            })
            .collect();

        let scroll_info = if category_files.len() > available_height {
            format!(" ({}/{}) ", visible_end, category_files.len())
        } else {
            String::new()
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" All Files{}", scroll_info))
                .border_style(Style::default().fg(color))
                .padding(Padding::new(1, 1, 0, 0)),
        );

        f.render_widget(list, chunks[1]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Scroll  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Back  "),
            Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(footer, chunks[2]);
    }

    fn render_category_box(&self, f: &mut Frame, area: Rect, name: &str, cat_idx: usize, files: Option<&Vec<&FileInfo>>) {
        let count = files.map(|f| f.len()).unwrap_or(0);
        let total_size: u64 = files
            .map(|f| f.iter().map(|fi| fi.size).sum())
            .unwrap_or(0);

        let title = format!(" [{}] {} ({}) ", cat_idx + 1, name, count);
        let color = match name {
            "Multimedia" => Color::Magenta,
            "Docs" => Color::Blue,
            "Compressed" => Color::Yellow,
            _ => Color::White,
        };

        let mut items: Vec<ListItem> = vec![
            ListItem::new(Line::from(vec![
                Span::styled("Total: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_size(total_size),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(""),
        ];

        if let Some(file_list) = files {
            // Calculate available height: area height - borders (2) - total line (1) - blank line (1) - "more" line (1 if needed)
            let available_height = area.height.saturating_sub(4) as usize;
            let max_files = if file_list.len() > available_height {
                available_height.saturating_sub(1) // Leave room for "... X more"
            } else {
                file_list.len()
            };

            for file in file_list.iter().take(max_files) {
                let filename = file
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                // Calculate max filename width: area width - bullet (2) - size (10) - padding (4) - borders (2)
                let max_filename_width = area.width.saturating_sub(18) as usize;
                let truncated = truncate_str(&filename, max_filename_width);

                // Pad filename to fixed width for alignment
                let padded_filename = format!("{:<width$}", truncated, width = max_filename_width);
                let size_str = format!("{:>10}", format_size(file.size));

                items.push(ListItem::new(Line::from(vec![
                    Span::raw("• "),
                    Span::styled(padded_filename, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(size_str, Style::default().fg(Color::DarkGray)),
                ])));
            }

            if file_list.len() > max_files {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("  ... {} more", file_list.len() - max_files),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ))));
            }
        }

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(title)
                .padding(Padding::new(1, 1, 0, 0)),
        );

        f.render_widget(list, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[1-4] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("View Category  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Organize  "),
            Span::styled("[q] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("Cancel"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(footer, area);
    }
}

pub struct ProgressApp {
    pub total_files: usize,
    pub current_file: String,
    pub current_category: String,
    pub current_mime: String,
    pub current_size: u64,
    pub stats: OrganizeStats,
    pub category_progress: HashMap<String, CategoryProgress>,
}

#[derive(Clone)]
pub struct CategoryProgress {
    pub count: usize,
    pub size: u64,
}

impl ProgressApp {
    pub fn new(total_files: usize) -> Self {
        let mut category_progress = HashMap::new();
        for cat in ["Multimedia", "Docs", "Compressed", "Misc"] {
            category_progress.insert(
                cat.to_string(),
                CategoryProgress {
                    count: 0,
                    size: 0,
                },
            );
        }

        Self {
            total_files,
            current_file: String::new(),
            current_category: String::new(),
            current_mime: String::new(),
            current_size: 0,
            stats: OrganizeStats::new(),
            category_progress,
        }
    }

    pub fn update_current(&mut self, file: &FileInfo) {
        self.current_file = file
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.current_category = file.category.clone();
        self.current_mime = file.mime_type.clone().unwrap_or_else(|| "unknown".to_string());
        self.current_size = file.size;
    }

    pub fn update_category(&mut self, category: &str, size: u64) {
        if let Some(prog) = self.category_progress.get_mut(category) {
            prog.count += 1;
            prog.size += size;
        }
    }

    pub fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Title
        self.render_title(f, chunks[0]);

        // Overall progress
        self.render_overall_progress(f, chunks[1]);

        // Category status
        self.render_category_status(f, chunks[2]);

        // Current file
        self.render_current_file(f, chunks[3]);

        // Summary
        self.render_summary(f, chunks[4]);
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "Organizing Files",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(title, area);
    }

    fn render_overall_progress(&self, f: &mut Frame, area: Rect) {
        let processed = self.stats.moved + self.stats.skipped + self.stats.errors;
        let ratio = if self.total_files > 0 {
            processed as f64 / self.total_files as f64
        } else {
            0.0
        };

        let label = format!("{}/{} files ({}%)", processed, self.total_files, (ratio * 100.0) as u8);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent((ratio * 100.0) as u16)
            .label(label);

        f.render_widget(gauge, area);
    }

    fn render_category_status(&self, f: &mut Frame, area: Rect) {
        let categories = vec!["Multimedia", "Docs", "Compressed", "Misc"];

        let items: Vec<ListItem> = categories
            .iter()
            .map(|cat| {
                let prog = self.category_progress.get(*cat).unwrap();
                let is_current = *cat == self.current_category;

                let (icon, style) = if prog.count > 0 {
                    ("✓", Style::default().fg(Color::Green))
                } else if is_current {
                    ("⊙", Style::default().fg(Color::Yellow))
                } else {
                    ("○", Style::default().fg(Color::DarkGray))
                };

                let bar_width: usize = 20;
                let filled = if self.total_files > 0 {
                    (prog.count as f64 / self.total_files as f64 * bar_width as f64) as usize
                } else {
                    0
                };
                let bar = format!(
                    "{}{}",
                    "█".repeat(filled),
                    "░".repeat(bar_width.saturating_sub(filled))
                );

                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", icon), style),
                    Span::styled(
                        format!("{:<12}", cat),
                        if is_current {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::styled(bar, Style::default().fg(Color::Cyan)),
                    Span::raw(format!("  {:>3} files  {:>8}", prog.count, format_size(prog.size))),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Category Status ")
                .border_style(Style::default().fg(Color::Blue)),
        );

        f.render_widget(list, area);
    }

    fn render_current_file(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  Processing: "),
                Span::styled(
                    &self.current_file,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" → "),
                Span::styled(&self.current_category, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("  Size: "),
                Span::styled(
                    format_size(self.current_size),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" | Type: "),
                Span::styled(&self.current_mime, Style::default().fg(Color::Magenta)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Current ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_summary(&self, f: &mut Frame, area: Rect) {
        let summary = Paragraph::new(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw(format!("Moved: {} ", self.stats.moved)),
            Span::raw("  "),
            Span::styled("⊘ ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("Skipped: {} ", self.stats.skipped)),
            Span::raw("  "),
            Span::styled("✗ ", Style::default().fg(Color::Red)),
            Span::raw(format!("Errors: {}", self.stats.errors)),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Summary ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(summary, area);
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
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub struct SummaryApp {
    pub stats: OrganizeStats,
    pub category_progress: HashMap<String, CategoryProgress>,
    pub elapsed_time: Duration,
    pub total_size_moved: u64,
}

impl SummaryApp {
    pub fn new(
        stats: OrganizeStats,
        category_progress: HashMap<String, CategoryProgress>,
        elapsed_time: Duration,
        total_size_moved: u64,
    ) -> Self {
        Self {
            stats,
            category_progress,
            elapsed_time,
            total_size_moved,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the app
        let res = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    fn run_loop(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Title
        self.render_title(f, chunks[0]);

        // Overall stats
        self.render_overall_stats(f, chunks[1]);

        // Category breakdown
        self.render_category_breakdown(f, chunks[2]);

        // Footer
        self.render_footer(f, chunks[3]);
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let success_rate = if self.stats.total_files > 0 {
            (self.stats.moved as f64 / self.stats.total_files as f64 * 100.0) as u8
        } else {
            0
        };

        let (title, color) = if self.stats.errors > 0 {
            ("Organization Completed with Errors", Color::Yellow)
        } else if self.stats.moved == self.stats.total_files {
            ("Organization Completed Successfully!", Color::Green)
        } else {
            ("Organization Completed", Color::Cyan)
        };

        let title_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    title,
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{}% Success", success_rate),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        );

        f.render_widget(title_widget, area);
    }

    fn render_overall_stats(&self, f: &mut Frame, area: Rect) {
        let elapsed_secs = self.elapsed_time.as_secs_f64();
        let speed_mbs = if elapsed_secs > 0.0 {
            self.total_size_moved as f64 / elapsed_secs / 1_000_000.0
        } else {
            0.0
        };
        let files_per_sec = if elapsed_secs > 0.0 {
            self.stats.moved as f64 / elapsed_secs
        } else {
            0.0
        };

        let items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("Total Files:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", self.stats.total_files),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✓ Moved:        ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("{}", self.stats.moved),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("({})", format_size(self.total_size_moved)),
                    Style::default().fg(Color::Yellow),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("⊘ Skipped:      ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", self.stats.skipped),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✗ Errors:       ", Style::default().fg(Color::Red)),
                Span::styled(
                    format!("{}", self.stats.errors),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Time Elapsed:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2}s", elapsed_secs),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Speed:          ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} MB/s  ({:.1} files/s)", speed_mbs, files_per_sec),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
            ])),
        ];

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Summary ")
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::new(2, 2, 0, 0)),
        );

        f.render_widget(list, area);
    }

    fn render_category_breakdown(&self, f: &mut Frame, area: Rect) {
        let categories = vec!["Multimedia", "Docs", "Compressed", "Misc"];

        let items: Vec<ListItem> = categories
            .iter()
            .filter_map(|cat| {
                self.category_progress.get(*cat).and_then(|prog| {
                    if prog.count > 0 {
                        Some(ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("  {:<12}", cat),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("  "),
                            Span::styled(
                                format!("{:>4}", prog.count),
                                Style::default().fg(Color::Green),
                            ),
                            Span::raw(" files  "),
                            Span::styled(
                                format!("({:>10})", format_size(prog.size)),
                                Style::default().fg(Color::Yellow),
                            ),
                        ])))
                    } else {
                        None
                    }
                })
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Files by Category ")
                .border_style(Style::default().fg(Color::Blue))
                .padding(Padding::new(2, 2, 1, 1)),
        );

        f.render_widget(list, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled(
                "[Enter]",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                "[q]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to exit"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(footer, area);
    }
}
