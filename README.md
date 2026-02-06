# File Organizer

A fast, reliable, and feature-rich Rust application to automatically organize files in a directory by type.

## Features

- **Ranger-style TUI** - Beautiful interactive dashboard with bordered panels (inspired by ranger)
- **Category Preview** - Visual breakdown of files by category before organizing
- **Real-time Progress** - Live progress dashboard with category-by-category status
- **Permanent Summary** - Detailed text summary after TUI closes with stats, performance metrics, and category breakdown
- **Automatic categorization** - Files are organized into folders based on their MIME type
- **Comprehensive file type support** - Handles images, videos, audio, documents, archives, code files, and more
- **Dry run mode** - Preview changes before applying them
- **Conflict resolution** - Choose how to handle duplicate files (skip, overwrite, or rename)
- **Recursive processing** - Optionally organize files in subdirectories
- **Verbose mode** - See detailed information about every file operation
- **Native Rust implementation** - Fast and reliable using native filesystem operations

## Categories

Files are automatically organized into the following folders:

- **Multimedia** - Images (PNG, JPEG, GIF, WebP, SVG, etc.), videos (MP4, AVI, MKV, etc.), and audio files (MP3, WAV, FLAC, etc.)
- **Docs** - Documents (PDF, DOCX, XLSX, PPTX), text files (TXT, MD, HTML, CSV, XML, RTF)
- **Compressed** - Archives (ZIP, RAR, 7Z, TAR, GZ, BZ2, XZ)
- **Misc** - All other file types including code files (JS, TS, PY, RS, GO, PHP, JAVA, C, C++, JSON, CSS)

## Requirements

- Rust 1.70 or higher

## Installation

### From source

```bash
git clone https://github.com/yourusername/file-organizer.git
cd file-organizer
cargo build --release
```

The compiled binary will be at `target/release/organizer` (or `organizer.exe` on Windows).

### Install globally

```bash
cargo install --path .
```

## Usage

### Basic usage

```bash
organizer <path>
```

### Options

```
File organizer - Automatically organize files into categorized folders

Usage: organizer [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to organize

Options:
  -d, --dry-run              Dry run - show what would be done without actually moving files
  -v, --verbose              Verbose output - show detailed information
  -c, --conflict <CONFLICT>  Conflict resolution strategy: skip, overwrite, or rename [default: skip]
  -r, --recursive            Recursive - organize files in subdirectories as well
  -i, --interactive          Interactive mode - show TUI dashboard
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

#### Interactive mode with TUI dashboard (Recommended)
```bash
organizer --interactive ~/Downloads
```

#### Interactive preview (shows category breakdown, press Enter to proceed or q to cancel)
```bash
organizer --interactive --dry-run ~/Downloads
```

#### Basic organization (simple mode)
```bash
organizer ~/Downloads
```

#### Preview without making changes (table format)
```bash
organizer --dry-run ~/Downloads
```

#### See detailed output
```bash
organizer --verbose ~/Downloads
```

#### Automatically rename conflicting files
```bash
organizer --conflict rename ~/Downloads
```

#### Organize recursively with TUI
```bash
organizer --interactive --recursive ~/Documents
```

## Example Output

### Interactive Mode (TUI)

Preview Dashboard (--interactive --dry-run):
```
┌─────────────────────────── Preview ───────────────────────────┐
│   File Organizer v0.2.0  |  233 files  |  1.45 GB             │
└───────────────────────────────────────────────────────────────┘
┌─ Multimedia (89) ─────────────┐ ┌─ Docs (67) ───────────────┐
│ Total: 245.23 MB              │ │ Total: 89.45 MB           │
│                               │ │                           │
│ • video.mp4          5.2 MB   │ │ • report.pdf     2.1 MB   │
│ • music.mp3          3.4 MB   │ │ • data.xlsx      1.5 MB   │
│ • photo.jpg          2.1 MB   │ │ • slides.pptx    4.2 MB   │
│   ... 84 more                 │ │   ... 62 more             │
└───────────────────────────────┘ └───────────────────────────┘
┌─ Compressed (45) ─────────────┐ ┌─ Misc (32) ───────────────┐
│ Total: 215.67 MB              │ │ Total: 14.68 MB           │
│                               │ │                           │
│ • archive.zip        15 MB    │ │ • app.py          8 KB    │
│ • backup.7z          89 MB    │ │ • config.json     2 KB    │
│   ... 41 more                 │ │   ... 28 more             │
└───────────────────────────────┘ └───────────────────────────┘
┌────────────────────────────────────────────────────────────────┐
│         [1-4] View  [Enter] Organize  [q] Cancel               │
└────────────────────────────────────────────────────────────────┘
```

Progress Dashboard (--interactive):
```
┌───────────────────────── Organizing Files ────────────────────┐
│                                                                │
└────────────────────────────────────────────────────────────────┘
┌──────────────────────────── Progress ─────────────────────────┐
│ ████████████████████████████████░░░░░░░░  156/233 files (67%) │
└────────────────────────────────────────────────────────────────┘
┌─────────────────────── Category Status ───────────────────────┐
│  ✓ Multimedia    ████████████████████  89 files   245 MB     │
│  ✓ Docs          ████████████████░░░░  67 files    89 MB     │
│  ○ Compressed    ░░░░░░░░░░░░░░░░░░░░   0 files     0 MB     │
│  ○ Misc          ░░░░░░░░░░░░░░░░░░░░   0 files     0 MB     │
└────────────────────────────────────────────────────────────────┘
┌──────────────────────────── Current ──────────────────────────┐
│                                                                │
│  Processing: document.pdf → Docs                              │
│  Size: 2.5 MB | Type: application/pdf                         │
└────────────────────────────────────────────────────────────────┘
┌──────────────────────────── Summary ──────────────────────────┐
│ ✓ Moved: 156    ⊘ Skipped: 5    ✗ Errors: 0                  │
└────────────────────────────────────────────────────────────────┘
```

After the TUI closes, a permanent text summary is displayed:

```
════════════════════════════════════════════════════════════
Organization Complete
════════════════════════════════════════════════════════════

Summary:
  Total files: 254
  ✓ Moved: 254
  ⊘ Skipped: 0
  ✗ Errors: 0
  Success rate: 100.0%

Performance:
  Time elapsed: 0.69s
  Data moved: 916.29 MB
  Speed: 1327.94 MB/s
  Throughput: 368.1 files/s

Categories:
  Docs 121 files (100.17 MB)
  Compressed 12 files (13.23 MB)
  Misc 79 files (751.10 MB)
  Multimedia 42 files (51.78 MB)

════════════════════════════════════════════════════════════
```

This ensures you always have a permanent record of the organization results, even after the interactive UI disappears.

### Simple Mode

```
Organizing /Users/user/Downloads

File                                               Size         Category        MIME Type
────────────────────────────────────────────────────────────────────────────────────────────
document.pdf                                       2.45 MB      Docs            application/pdf
music.mp3                                          5.23 MB      Multimedia      audio/mpeg
...

Summary:
  Total files: 42
  Moved: 38
  Skipped: 4
  Errors: 0
```

## Motivation

My Downloads folder was always cluttered with hundreds of files of different types. Instead of manually sorting them, I built this simple command-line tool in Rust to automatically organize everything. It's fast, reliable, and makes keeping directories tidy effortless.

## Architecture

The application is built with:
- **ratatui** - Terminal UI framework for the interactive dashboard (ranger-inspired)
- **crossterm** - Cross-platform terminal manipulation
- **clap** - Modern CLI argument parsing with derive macros
- **mime_guess** - Accurate MIME type detection based on file extensions
- **indicatif** - Beautiful progress bars and spinners (simple mode)
- **colored** - Terminal color support for better UX
- Native Rust filesystem operations for reliability and cross-platform support

## Development

### Running tests

```bash
cargo test
```

### Running with cargo

```bash
cargo run -- ~/Downloads --dry-run
```

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

MIT

## Version History

### 0.2.0 (Current)
- **NEW: Interactive TUI Mode** - Ranger-inspired dashboard with bordered panels
  - Category Preview Dashboard for dry-run visualization
  - Real-time Progress Dashboard with live category tracking
  - Comprehensive Summary Dashboard with detailed statistics
  - Permanent text summary printed after TUI closes (success rate, performance metrics, category breakdown)
  - Keyboard controls (1-4 to view categories, arrows to scroll, ESC to go back, Enter to proceed, q to quit)
  - Proper file size alignment and space utilization
  - 60 FPS rendering for smooth progress updates
- Migrated from structopt to clap v4
- Added progress bars and colored terminal output
- Implemented dry-run mode with detailed table output
- Added conflict resolution strategies (skip, overwrite, rename)
- Expanded MIME type support (videos, more images, code files)
- Added recursive directory processing
- Replaced shell commands with native Rust filesystem operations
- Improved error handling with custom error types
- Added comprehensive unit tests
- Files without MIME types now categorized as "Misc"
- Four categories: Multimedia, Docs, Compressed, and Misc (code files go to Misc)

### 0.1.1
- Initial release with basic functionality
