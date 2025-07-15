//! # FastDLP - Fast Data Loss Prevention
//!
//! A high-performance Rust implementation of data loss prevention (DLP) system,
//! translated from the Python openDLP project.
//!
//! ## Features
//!
//! - **Sensitive Data Analysis**: Identify sensitive data in structured and semi-structured data
//! - **Regex Generation**: Automatically generate regex patterns from positive/negative samples
//! - **High Performance**: Leverages Rust's speed and memory safety
//! - **Extensible**: Support for custom sensitive data types and patterns
//!
//! ## Quick Start
//!
//! ```rust
//! use fastdlp::sensitive_analyze::TableAnalyzer;
//! 
//! let analyzer = TableAnalyzer::new();
//! let result = analyzer.analyze_csv("path/to/data.csv").await?;
//! println!("Analysis result: {:?}", result);
//! ```

#[macro_use]
extern crate log;

pub mod sensitive_analyze;
pub mod regex_generation;
pub mod common;
pub mod error;

pub use error::{FastDlpError, Result};

/// Re-export common types and functions for convenience
pub mod prelude {
    pub use crate::sensitive_analyze::{AnalyzerEngine, TableAnalyzer};
    pub use crate::regex_generation::RegexGenerator;
    pub use crate::common::*;
    pub use crate::error::{FastDlpError, Result};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the logger (call once at the start of your application)
pub fn init_logger() {
    env_logger::init();
} 