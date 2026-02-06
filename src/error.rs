use std::path::PathBuf;
use std::fmt;

#[derive(Debug)]
pub enum OrganizerError {
    IoError(std::io::Error),
    PathNotFound(PathBuf),
    PathNotDirectory(PathBuf),
    InvalidPath(String),
}

impl fmt::Display for OrganizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrganizerError::IoError(e) => write!(f, "I/O error: {}", e),
            OrganizerError::PathNotFound(path) => {
                write!(f, "Path not found: {}", path.display())
            }
            OrganizerError::PathNotDirectory(path) => {
                write!(f, "Path is not a directory: {}", path.display())
            }
            OrganizerError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

impl std::error::Error for OrganizerError {}

impl From<std::io::Error> for OrganizerError {
    fn from(error: std::io::Error) -> Self {
        OrganizerError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, OrganizerError>;
