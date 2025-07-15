use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::sensitive_analyze::entity_classify::EntityClassifier;
use crate::sensitive_analyze::entity_recognize::EntityRecognizer;
use std::collections::HashMap;

/// AnalyzerEngine is the core component for sensitive data analysis
pub struct AnalyzerEngine {
    entity_classifier: EntityClassifier,
    entity_recognizer: EntityRecognizer,
    custom_patterns: Vec<RegexPattern>,
}

impl AnalyzerEngine {
    /// Create a new AnalyzerEngine
    pub fn new() -> Self {
        Self {
            entity_classifier: EntityClassifier::new(),
            entity_recognizer: EntityRecognizer::new(),
            custom_patterns: Vec::new(),
        }
    }

    /// Set custom regex patterns
    pub fn set_custom_patterns(&mut self, patterns: Vec<RegexPattern>) {
        self.custom_patterns = patterns;
        self.entity_recognizer.set_custom_patterns(patterns.clone());
    }

    /// Analyze a list of texts and return the detected sensitive data types
    pub async fn analyze(&self, texts: &[String], thresholds: &Option<ThresholdConfig>) -> Result<Vec<Option<String>>> {
        info!("Starting analysis of {} texts", texts.len());

        // First, try predefined analysis
        let predefined_results = self.analyze_predefined(texts).await?;

        // If no custom patterns, return predefined results
        if self.custom_patterns.is_empty() {
            return Ok(predefined_results);
        }

        // Check if predefined results are good enough
        let (most_common_type, count) = most_common(&predefined_results)
            .unwrap_or((None, 0));

        if let Some(ref entity_type) = most_common_type {
            let threshold = get_threshold(thresholds, entity_type);
            if count as f64 / predefined_results.len() as f64 >= threshold {
                return Ok(predefined_results);
            }
        }

        // Try user-defined analysis
        let user_defined_results = self.analyze_user_defined(texts, &predefined_results).await?;
        Ok(user_defined_results)
    }

    /// Analyze texts using predefined sensitive data types
    async fn analyze_predefined(&self, texts: &[String]) -> Result<Vec<Option<String>>> {
        info!("Analyzing with predefined patterns");

        let mut results = vec![None; texts.len()];
        let mut tried_entities = Vec::new();

        // Get candidate entities from classifier
        let candidate_entities = self.entity_classifier.predict(texts).await?;

        // Keep track of which texts still need analysis
        let mut text_analyze_flags = vec![true; texts.len()];

        while text_analyze_flags.iter().any(|&flag| flag) {
            // Get texts that still need analysis
            let texts_to_analyze: Vec<String> = texts.iter()
                .zip(text_analyze_flags.iter())
                .filter_map(|(text, &flag)| if flag { Some(text.clone()) } else { None })
                .collect();

            if texts_to_analyze.is_empty() {
                break;
            }

            // Collect all candidate entities for these texts
            let mut entities = Vec::new();
            for text in &texts_to_analyze {
                if let Some(candidates) = candidate_entities.get(text) {
                    entities.extend(candidates.clone());
                }
            }

            if entities.is_empty() {
                break;
            }

            // Find the most common entity that we haven't tried yet
            let entity_counts = most_common(&entities);
            let mut found_new_entity = false;

            if let Some((entity_type, _)) = entity_counts {
                if !tried_entities.contains(&entity_type) {
                    tried_entities.push(entity_type.clone());

                    // Analyze with this entity type
                    let recognition_results = self.entity_recognizer
                        .analyze(&texts_to_analyze, &entity_type).await?;

                    // Update results
                    let mut result_idx = 0;
                    for (i, &flag) in text_analyze_flags.iter().enumerate() {
                        if flag {
                            if let Some(ref detected_type) = recognition_results[result_idx] {
                                results[i] = Some(detected_type.clone());
                                text_analyze_flags[i] = false;
                            }
                            result_idx += 1;
                        }
                    }

                    found_new_entity = true;
                }
            }

            if !found_new_entity {
                break;
            }
        }

        Ok(results)
    }

    /// Analyze texts using user-defined patterns
    async fn analyze_user_defined(&self, texts: &[String], predefined_results: &[Option<String>]) -> Result<Vec<Option<String>>> {
        info!("Analyzing with user-defined patterns");

        let mut results = predefined_results.to_vec();
        let mut text_analyze_flags = vec![false; texts.len()];

        // Mark texts that need analysis (those without predefined results)
        for (i, result) in predefined_results.iter().enumerate() {
            if result.is_none() {
                text_analyze_flags[i] = true;
            }
        }

        // Try each custom pattern
        for pattern in &self.custom_patterns {
            if !text_analyze_flags.iter().any(|&flag| flag) {
                break; // All texts have been analyzed
            }

            // Get texts that still need analysis
            let texts_to_analyze: Vec<String> = texts.iter()
                .zip(text_analyze_flags.iter())
                .filter_map(|(text, &flag)| if flag { Some(text.clone()) } else { None })
                .collect();

            if texts_to_analyze.is_empty() {
                break;
            }

            // Analyze with this pattern
            let recognition_results = self.entity_recognizer
                .analyze_with_pattern(&texts_to_analyze, pattern).await?;

            // Update results
            let mut result_idx = 0;
            for (i, &flag) in text_analyze_flags.iter().enumerate() {
                if flag {
                    if let Some(ref detected_type) = recognition_results[result_idx] {
                        results[i] = Some(detected_type.clone());
                        text_analyze_flags[i] = false;
                    }
                    result_idx += 1;
                }
            }
        }

        Ok(results)
    }
}

impl Default for AnalyzerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_engine_basic() {
        let engine = AnalyzerEngine::new();
        let texts = vec![
            "test@example.com".to_string(),
            "13812345678".to_string(),
            "regular text".to_string(),
        ];

        let results = engine.analyze(&texts, &None).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_analyzer_engine_with_custom_patterns() {
        let mut engine = AnalyzerEngine::new();
        let patterns = vec![
            RegexPattern {
                name: "TEST_PATTERN".to_string(),
                pattern: r"^test_\d+$".to_string(),
                description: Some("Test pattern".to_string()),
            }
        ];
        engine.set_custom_patterns(patterns);

        let texts = vec![
            "test_123".to_string(),
            "test_456".to_string(),
            "regular text".to_string(),
        ];

        let results = engine.analyze(&texts, &None).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_analyzer_engine_creation() {
        let engine = AnalyzerEngine::new();
        assert_eq!(engine.custom_patterns.len(), 0);
    }
} 