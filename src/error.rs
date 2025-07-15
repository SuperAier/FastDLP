use thiserror::Error;

/// FastDLP error types
#[derive(Error, Debug)]
pub enum FastDlpError {
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Fancy regex error: {0}")]
    FancyRegexError(#[from] fancy_regex::Error),

    #[error("Analysis error: {message}")]
    AnalysisError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Data processing error: {message}")]
    DataProcessingError { message: String },

    #[error("Model error: {message}")]
    ModelError { message: String },

    #[error("File read error: {message}")]
    FileReadError { message: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Timeout error: {message}")]
    TimeoutError { message: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl FastDlpError {
    pub fn analysis_error(message: impl Into<String>) -> Self {
        Self::AnalysisError {
            message: message.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    pub fn data_processing_error(message: impl Into<String>) -> Self {
        Self::DataProcessingError {
            message: message.into(),
        }
    }

    pub fn model_error(message: impl Into<String>) -> Self {
        Self::ModelError {
            message: message.into(),
        }
    }

    pub fn file_read_error(message: impl Into<String>) -> Self {
        Self::FileReadError {
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::TimeoutError {
            message: message.into(),
        }
    }

    pub fn unknown_error(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }
}

/// Result type used throughout the FastDLP library
pub type Result<T> = std::result::Result<T, FastDlpError>; 