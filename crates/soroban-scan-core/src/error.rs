//! Error types for the scanner core.

use std::path::PathBuf;

use thiserror::Error;

/// Top-level error type for scanner operations.
#[derive(Debug, Error)]
pub enum ScanError {
    /// An I/O operation failed for a specific path.
    #[error("I/O error for {path}: {source}")]
    Io {
        /// Path that triggered the error.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A file could not be parsed.
    #[error("failed to parse {path}: {message}")]
    Parse {
        /// Path of the unparseable file.
        path: PathBuf,
        /// Human-readable explanation.
        message: String,
    },

    /// A configuration file was invalid.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// A baseline file was invalid or incompatible.
    #[error("invalid baseline: {0}")]
    Baseline(String),

    /// A requested rule id does not exist.
    #[error("unknown rule id: {0}")]
    UnknownRule(String),

    /// A path was rejected for security reasons.
    #[error("unsafe path rejected: {0}")]
    UnsafePath(String),
}

impl ScanError {
    /// Convenience constructor for [`ScanError::Io`].
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        ScanError::Io {
            path: path.into(),
            source,
        }
    }

    /// Convenience constructor for [`ScanError::Parse`].
    pub fn parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        ScanError::Parse {
            path: path.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_displays_path() {
        let err = ScanError::io(
            "/tmp/x.rs",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );
        let msg = err.to_string();
        assert!(msg.contains("/tmp/x.rs"));
        assert!(msg.contains("missing"));
    }

    #[test]
    fn parse_error_displays_message() {
        let err = ScanError::parse("a/b.rs", "unexpected token");
        assert!(err.to_string().contains("unexpected token"));
    }
}
