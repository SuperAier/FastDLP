use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::regex_generation::generator::RegexIndividual;
use std::collections::HashSet;
use regex::Regex;

/// Dataset manages training data for regex generation
pub struct Dataset {
    positive_examples: Vec<String>,
    negative_examples: Vec<String>,
    original_positive_count: usize,
    is_fixed_length: bool,
}

impl Dataset {
    /// Create a new dataset from training data
    pub fn new(training_data: TrainingData) -> Self {
        let positive_examples = training_data.positive;
        let negative_examples = training_data.negative;
        let original_positive_count = positive_examples.len();
        
        // Check if all positive examples have the same length
        let is_fixed_length = if positive_examples.is_empty() {
            false
        } else {
            let first_len = positive_examples[0].len();
            positive_examples.iter().all(|s| s.len() == first_len)
        };

        Self {
            positive_examples,
            negative_examples,
            original_positive_count,
            is_fixed_length,
        }
    }

    /// Build the dataset (perform any necessary preprocessing)
    pub fn build(&mut self) -> Result<()> {
        // Remove empty examples
        self.positive_examples.retain(|s| !s.is_empty());
        self.negative_examples.retain(|s| !s.is_empty());

        // Remove duplicates
        let mut unique_positive = HashSet::new();
        self.positive_examples.retain(|s| unique_positive.insert(s.clone()));

        let mut unique_negative = HashSet::new();
        self.negative_examples.retain(|s| unique_negative.insert(s.clone()));

        info!("Dataset built: {} positive, {} negative examples", 
              self.positive_examples.len(), self.negative_examples.len());

        Ok(())
    }

    /// Get positive examples
    pub fn positive_examples(&self) -> &[String] {
        &self.positive_examples
    }

    /// Get negative examples
    pub fn negative_examples(&self) -> &[String] {
        &self.negative_examples
    }

    /// Check if dataset has fixed length examples
    pub fn is_fixed_length(&self) -> bool {
        self.is_fixed_length
    }

    /// Get original positive count
    pub fn original_positive_count(&self) -> usize {
        self.original_positive_count
    }

    /// Remove examples that match the given regex individual
    pub fn remove_matched_examples(&mut self, individual: &RegexIndividual) -> Result<()> {
        // Compile the regex pattern
        let regex = Regex::new(&individual.pattern)
            .map_err(|e| FastDlpError::regex_error(e))?;

        // Remove matching positive examples
        let initial_count = self.positive_examples.len();
        self.positive_examples.retain(|example| !regex.is_match(example));
        let removed_count = initial_count - self.positive_examples.len();

        info!("Removed {} positive examples that matched pattern: {}", 
              removed_count, individual.pattern);

        Ok(())
    }

    /// Get statistics about the dataset
    pub fn stats(&self) -> DatasetStats {
        DatasetStats {
            positive_count: self.positive_examples.len(),
            negative_count: self.negative_examples.len(),
            original_positive_count: self.original_positive_count,
            is_fixed_length: self.is_fixed_length,
            avg_positive_length: self.avg_length(&self.positive_examples),
            avg_negative_length: self.avg_length(&self.negative_examples),
        }
    }

    /// Calculate average length of strings
    fn avg_length(&self, strings: &[String]) -> f64 {
        if strings.is_empty() {
            return 0.0;
        }
        
        let total_length: usize = strings.iter().map(|s| s.len()).sum();
        total_length as f64 / strings.len() as f64
    }

    /// Get character frequency distribution
    pub fn char_frequency(&self) -> std::collections::HashMap<char, usize> {
        let mut frequency = std::collections::HashMap::new();
        
        for example in &self.positive_examples {
            for ch in example.chars() {
                *frequency.entry(ch).or_insert(0) += 1;
            }
        }
        
        frequency
    }

    /// Get common substrings
    pub fn common_substrings(&self, min_length: usize, min_frequency: usize) -> Vec<String> {
        let mut substring_count = std::collections::HashMap::new();
        
        for example in &self.positive_examples {
            // Generate all substrings of minimum length
            for i in 0..=example.len().saturating_sub(min_length) {
                for j in i + min_length..=example.len() {
                    let substring = example[i..j].to_string();
                    *substring_count.entry(substring).or_insert(0) += 1;
                }
            }
        }
        
        // Filter by minimum frequency
        substring_count.into_iter()
            .filter(|(_, count)| *count >= min_frequency)
            .map(|(substring, _)| substring)
            .collect()
    }
}

/// Dataset statistics
#[derive(Debug, Clone)]
pub struct DatasetStats {
    pub positive_count: usize,
    pub negative_count: usize,
    pub original_positive_count: usize,
    pub is_fixed_length: bool,
    pub avg_positive_length: f64,
    pub avg_negative_length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_creation() {
        let training_data = TrainingData {
            positive: vec![
                "test@example.com".to_string(),
                "user@domain.org".to_string(),
            ],
            negative: vec![
                "not an email".to_string(),
                "invalid@".to_string(),
            ],
        };
        
        let dataset = Dataset::new(training_data);
        assert_eq!(dataset.positive_examples().len(), 2);
        assert_eq!(dataset.negative_examples().len(), 2);
        assert_eq!(dataset.original_positive_count(), 2);
        assert!(!dataset.is_fixed_length());
    }

    #[test]
    fn test_dataset_build() {
        let training_data = TrainingData {
            positive: vec![
                "test@example.com".to_string(),
                "test@example.com".to_string(), // duplicate
                "".to_string(), // empty
                "user@domain.org".to_string(),
            ],
            negative: vec![
                "not an email".to_string(),
                "".to_string(), // empty
            ],
        };
        
        let mut dataset = Dataset::new(training_data);
        dataset.build().unwrap();
        
        assert_eq!(dataset.positive_examples().len(), 2); // duplicates and empty removed
        assert_eq!(dataset.negative_examples().len(), 1); // empty removed
    }

    #[test]
    fn test_fixed_length_detection() {
        let training_data = TrainingData {
            positive: vec![
                "12345".to_string(),
                "67890".to_string(),
                "abcde".to_string(),
            ],
            negative: vec![],
        };
        
        let dataset = Dataset::new(training_data);
        assert!(dataset.is_fixed_length());
    }

    #[test]
    fn test_remove_matched_examples() {
        let training_data = TrainingData {
            positive: vec![
                "test@example.com".to_string(),
                "user@domain.org".to_string(),
                "notanemail".to_string(),
            ],
            negative: vec![],
        };
        
        let mut dataset = Dataset::new(training_data);
        dataset.build().unwrap();
        
        let individual = RegexIndividual::new(r".+@.+\..+".to_string(), 3);
        dataset.remove_matched_examples(&individual).unwrap();
        
        assert_eq!(dataset.positive_examples().len(), 1);
        assert_eq!(dataset.positive_examples()[0], "notanemail");
    }

    #[test]
    fn test_char_frequency() {
        let training_data = TrainingData {
            positive: vec![
                "aab".to_string(),
                "abc".to_string(),
            ],
            negative: vec![],
        };
        
        let dataset = Dataset::new(training_data);
        let frequency = dataset.char_frequency();
        
        assert_eq!(frequency.get(&'a'), Some(&3));
        assert_eq!(frequency.get(&'b'), Some(&2));
        assert_eq!(frequency.get(&'c'), Some(&1));
    }

    #[test]
    fn test_common_substrings() {
        let training_data = TrainingData {
            positive: vec![
                "prefix123".to_string(),
                "prefix456".to_string(),
                "prefix789".to_string(),
            ],
            negative: vec![],
        };
        
        let dataset = Dataset::new(training_data);
        let substrings = dataset.common_substrings(3, 2);
        
        assert!(substrings.contains(&"prefix".to_string()));
    }
} 