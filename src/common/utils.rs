use crate::common::types::*;
use crate::error::{FastDlpError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load custom regex patterns from a JSON file
pub fn load_custom_patterns<P: AsRef<Path>>(path: P) -> Result<Vec<RegexPattern>> {
    let content = fs::read_to_string(path)?;
    let patterns: Vec<RegexPattern> = serde_json::from_str(&content)?;
    Ok(patterns)
}

/// Get threshold for a specific data type
pub fn get_threshold(thresholds: &Option<ThresholdConfig>, data_type: &str) -> f64 {
    if let Some(thresholds) = thresholds {
        thresholds.get(data_type).copied().unwrap_or(crate::common::constants::DEFAULT_THRESHOLD)
    } else {
        crate::common::constants::DEFAULT_THRESHOLD
    }
}

/// Calculate the most common element in a vector
pub fn most_common<T: Clone + std::hash::Hash + Eq>(items: &[T]) -> Option<(T, usize)> {
    if items.is_empty() {
        return None;
    }

    let mut counts = HashMap::new();
    for item in items {
        *counts.entry(item.clone()).or_insert(0) += 1;
    }

    counts.into_iter().max_by_key(|(_, count)| *count)
}

/// Validate that a string is a valid regex pattern
pub fn validate_regex_pattern(pattern: &str) -> Result<()> {
    regex::Regex::new(pattern)?;
    Ok(())
}

/// Clean and normalize text data
pub fn normalize_text(text: &str) -> String {
    text.trim().to_string()
}

/// Check if a string is likely to be a sensitive data type
pub fn is_likely_sensitive(text: &str) -> bool {
    // Simple heuristics - can be enhanced
    !text.is_empty() && text.len() > 3
}

/// Format fraction string (e.g., "950/1000")
pub fn format_fraction(matched: usize, total: usize) -> String {
    format!("{}/{}", matched, total)
}

/// Parse fraction string to get ratio
pub fn parse_fraction(fraction: &str) -> Result<f64> {
    let parts: Vec<&str> = fraction.split('/').collect();
    if parts.len() != 2 {
        return Err(FastDlpError::invalid_input("Invalid fraction format"));
    }

    let numerator: usize = parts[0].parse()
        .map_err(|_| FastDlpError::invalid_input("Invalid numerator"))?;
    let denominator: usize = parts[1].parse()
        .map_err(|_| FastDlpError::invalid_input("Invalid denominator"))?;

    if denominator == 0 {
        return Err(FastDlpError::invalid_input("Division by zero"));
    }

    Ok(numerator as f64 / denominator as f64)
}

/// Load training data from CSV file
pub fn load_training_data<P: AsRef<Path>>(path: P) -> Result<TrainingData> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut positive = Vec::new();
    let mut negative = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.len() >= 2 {
            let pos = record.get(0).unwrap_or("").trim();
            let neg = record.get(1).unwrap_or("").trim();
            
            if !pos.is_empty() {
                positive.push(pos.to_string());
            }
            if !neg.is_empty() {
                negative.push(neg.to_string());
            }
        }
    }

    Ok(TrainingData { positive, negative })
}

/// Save results to JSON file
pub fn save_results_json<P: AsRef<Path>>(path: P, results: &TableAnalysisResult) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write(path, json)?;
    Ok(())
} 