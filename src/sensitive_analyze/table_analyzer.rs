use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::sensitive_analyze::analyzer_engine::AnalyzerEngine;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// TableAnalyzer provides high-level interface for analyzing CSV tables
pub struct TableAnalyzer {
    analyzer_engine: AnalyzerEngine,
    config: AnalyzerConfig,
}

impl TableAnalyzer {
    /// Create a new TableAnalyzer with default configuration
    pub fn new() -> Self {
        Self {
            analyzer_engine: AnalyzerEngine::new(),
            config: AnalyzerConfig::default(),
        }
    }

    /// Create a new TableAnalyzer with custom configuration
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self {
            analyzer_engine: AnalyzerEngine::new(),
            config,
        }
    }

    /// Set custom regex patterns
    pub fn set_custom_patterns(&mut self, patterns: Vec<RegexPattern>) {
        self.analyzer_engine.set_custom_patterns(patterns);
    }

    /// Set threshold configuration
    pub fn set_thresholds(&mut self, thresholds: ThresholdConfig) {
        self.config.custom_thresholds = thresholds;
    }

    /// Analyze a CSV file and return results for each column
    pub async fn analyze_csv<P: AsRef<Path>>(&self, csv_path: P) -> Result<TableAnalysisResult> {
        info!("Starting analysis of CSV file: {:?}", csv_path.as_ref());

        // Read CSV file using polars
        let df = LazyFrame::scan_csv(csv_path.as_ref(), ScanArgsCSV::default())
            .map_err(|e| FastDlpError::file_read_error(format!("Failed to read CSV: {}", e)))?;

        let df = df.collect()
            .map_err(|e| FastDlpError::data_processing_error(format!("Failed to collect data: {}", e)))?;

        info!("Table columns: {}", df.width());
        info!("Table rows: {}", df.height());

        // Limit number of rows to process
        let data_num = std::cmp::min(df.height(), self.config.max_rows);
        let df = df.slice(0, data_num as i64);
        
        info!("After selecting at most {} rows, table rows: {}", self.config.max_rows, df.height());

        let mut results = HashMap::new();

        // Process each column
        for column_name in df.get_column_names() {
            info!("Analyzing column: {}", column_name);

            match self.analyze_column(&df, column_name).await {
                Ok(result) => {
                    results.insert(column_name.to_string(), result);
                }
                Err(e) => {
                    error!("Failed to analyze column {}: {}", column_name, e);
                    results.insert(column_name.to_string(), AnalysisResult {
                        success: false,
                        data_type: OTHER_TYPE.to_string(),
                        fraction: "0/0".to_string(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Analyze a single column
    async fn analyze_column(&self, df: &DataFrame, column_name: &str) -> Result<AnalysisResult> {
        let column = df.column(column_name)
            .map_err(|e| FastDlpError::data_processing_error(format!("Column not found: {}", e)))?;

        // Convert column to string vector
        let texts: Vec<String> = column
            .str()
            .map_err(|e| FastDlpError::data_processing_error(format!("Failed to convert to string: {}", e)))?
            .into_iter()
            .map(|opt_s| opt_s.unwrap_or("").to_string())
            .collect();

        // Analyze using the analyzer engine
        let analysis_results = self.analyzer_engine.analyze(&texts, &Some(self.config.custom_thresholds.clone())).await?;

        // Calculate the most common result
        let (most_common_type, count) = most_common(&analysis_results)
            .unwrap_or((None, 0));

        let total_count = analysis_results.len();
        let threshold = get_threshold(&Some(self.config.custom_thresholds.clone()), 
                                     most_common_type.as_deref().unwrap_or(""));

        let detected_type = if let Some(ref entity_type) = most_common_type {
            if count as f64 / total_count as f64 >= threshold {
                entity_type.clone()
            } else {
                OTHER_TYPE.to_string()
            }
        } else {
            OTHER_TYPE.to_string()
        };

        Ok(AnalysisResult {
            success: true,
            data_type: detected_type,
            fraction: format_fraction(count, total_count),
        })
    }
}

impl Default for TableAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_analyze_csv_basic() {
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "email,phone").unwrap();
        writeln!(temp_file, "test@example.com,13812345678").unwrap();
        writeln!(temp_file, "user@domain.org,13987654321").unwrap();
        temp_file.flush().unwrap();

        let analyzer = TableAnalyzer::new();
        let results = analyzer.analyze_csv(temp_file.path()).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.contains_key("email"));
        assert!(results.contains_key("phone"));
    }

    #[test]
    fn test_table_analyzer_creation() {
        let analyzer = TableAnalyzer::new();
        assert_eq!(analyzer.config.max_rows, DEFAULT_MAX_ROWS);
        assert_eq!(analyzer.config.default_threshold, DEFAULT_THRESHOLD);
    }

    #[test]
    fn test_table_analyzer_with_config() {
        let config = AnalyzerConfig {
            max_rows: 500,
            default_threshold: 0.9,
            custom_thresholds: HashMap::new(),
        };
        let analyzer = TableAnalyzer::with_config(config);
        assert_eq!(analyzer.config.max_rows, 500);
        assert_eq!(analyzer.config.default_threshold, 0.9);
    }
} 