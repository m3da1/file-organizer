pub mod cli;
pub mod error;
pub mod tui;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_categorize_image_files() {
        let mime_png = Some("image/png".to_string());
        let mime_jpeg = Some("image/jpeg".to_string());
        let mime_gif = Some("image/gif".to_string());

        assert_eq!(cli::categorize_file(&mime_png), "Multimedia");
        assert_eq!(cli::categorize_file(&mime_jpeg), "Multimedia");
        assert_eq!(cli::categorize_file(&mime_gif), "Multimedia");
    }

    #[test]
    fn test_categorize_video_files() {
        let mime_mp4 = Some("video/mp4".to_string());
        let mime_webm = Some("video/webm".to_string());

        assert_eq!(cli::categorize_file(&mime_mp4), "Multimedia");
        assert_eq!(cli::categorize_file(&mime_webm), "Multimedia");
    }

    #[test]
    fn test_categorize_archive_files() {
        let mime_zip = Some("application/zip".to_string());
        let mime_7z = Some("application/x-7z-compressed".to_string());
        let mime_tar = Some("application/x-tar".to_string());

        assert_eq!(cli::categorize_file(&mime_zip), "Compressed");
        assert_eq!(cli::categorize_file(&mime_7z), "Compressed");
        assert_eq!(cli::categorize_file(&mime_tar), "Compressed");
    }

    #[test]
    fn test_categorize_document_files() {
        let mime_pdf = Some("application/pdf".to_string());
        let mime_docx = Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string());

        assert_eq!(cli::categorize_file(&mime_pdf), "Docs");
        assert_eq!(cli::categorize_file(&mime_docx), "Docs");
    }

    #[test]
    fn test_categorize_code_files() {
        let mime_js = Some("text/javascript".to_string());
        let mime_json = Some("application/json".to_string());
        let mime_python = Some("text/x-python".to_string());

        assert_eq!(cli::categorize_file(&mime_js), "Misc");
        assert_eq!(cli::categorize_file(&mime_json), "Misc");
        assert_eq!(cli::categorize_file(&mime_python), "Misc");
    }

    #[test]
    fn test_categorize_unknown_files() {
        let mime_unknown = Some("application/x-unknown".to_string());
        let no_mime = None;

        assert_eq!(cli::categorize_file(&mime_unknown), "Misc");
        assert_eq!(cli::categorize_file(&no_mime), "Misc");
    }

    #[test]
    fn test_generate_unique_filename() {
        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("organizer_test");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a test file
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Generate unique filename
        let unique = cli::generate_unique_filename(&test_file);

        // Should have _1 suffix
        assert_eq!(unique.file_name().unwrap().to_str().unwrap(), "test_1.txt");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_organize_stats_new() {
        let stats = cli::OrganizeStats::new();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.moved, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.errors, 0);
    }
}
