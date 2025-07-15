use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the result of analyzing a single column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Whether the analysis was successful
    pub success: bool,
    /// The detected sensitive data type
    pub data_type: String,
    /// The fraction of data that matches the detected type (e.g., "950/1000")
    pub fraction: String,
}

/// Represents the result of analyzing a table
pub type TableAnalysisResult = HashMap<String, AnalysisResult>;

/// Represents a custom regex pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    /// Name of the pattern
    pub name: String,
    /// The regex pattern string
    pub pattern: String,
    /// Optional description
    pub description: Option<String>,
}

/// Threshold configuration for sensitive data detection
pub type ThresholdConfig = HashMap<String, f64>;

/// Represents configuration for the analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Maximum number of rows to process
    pub max_rows: usize,
    /// Default threshold for classification
    pub default_threshold: f64,
    /// Custom thresholds for specific data types
    pub custom_thresholds: ThresholdConfig,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_rows: 1000,
            default_threshold: 0.8,
            custom_thresholds: HashMap::new(),
        }
    }
}

/// Represents a regex generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexGenerationResult {
    /// Name of the generated regex
    pub regex_name: String,
    /// The generated regex pattern
    pub regex_pattern: String,
}

/// Training data for regex generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    /// Positive examples
    pub positive: Vec<String>,
    /// Negative examples
    pub negative: Vec<String>,
}

/// Evolution parameters for regex generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionParams {
    /// Initial population size
    pub init_population_size: usize,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Precision threshold for divide and conquer
    pub precision_divide_conquer: f64,
    /// Iteration threshold for divide and conquer
    pub iteration_divide_conquer: usize,
    /// Noise positive sample ratio
    pub noise_positive_sample_ratio: f64,
    /// Population size decay rate
    pub population_size_decay_rate: f64,
    /// Minimum population size
    pub min_population_size: usize,
}

impl Default for EvolutionParams {
    fn default() -> Self {
        Self {
            init_population_size: 1000,
            max_iterations: 2000,
            precision_divide_conquer: 0.8,
            iteration_divide_conquer: 10,
            noise_positive_sample_ratio: 0.05,
            population_size_decay_rate: 0.95,
            min_population_size: 200,
        }
    }
} 