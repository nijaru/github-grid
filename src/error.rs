use std::fmt;

#[derive(Debug)]
pub enum GitHubGridError {
    Git(git2::Error),
    Io(std::io::Error),
    Parse(String),
    Config(String),
    Authentication(String),
    Repository(String),
}

impl fmt::Display for GitHubGridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitHubGridError::Git(e) => write!(f, "Git error: {}", e),
            GitHubGridError::Io(e) => write!(f, "IO error: {}", e),
            GitHubGridError::Parse(msg) => write!(f, "Parse error: {}", msg),
            GitHubGridError::Config(msg) => write!(f, "Configuration error: {}", msg),
            GitHubGridError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            GitHubGridError::Repository(msg) => write!(f, "Repository error: {}", msg),
        }
    }
}

impl std::error::Error for GitHubGridError {}

impl From<git2::Error> for GitHubGridError {
    fn from(err: git2::Error) -> Self {
        GitHubGridError::Git(err)
    }
}

impl From<std::io::Error> for GitHubGridError {
    fn from(err: std::io::Error) -> Self {
        GitHubGridError::Io(err)
    }
}


impl From<chrono::ParseError> for GitHubGridError {
    fn from(err: chrono::ParseError) -> Self {
        GitHubGridError::Parse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitHubGridError>;