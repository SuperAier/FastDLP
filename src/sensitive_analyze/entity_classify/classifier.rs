use crate::common::*;
use crate::error::{FastDlpError, Result};
use std::collections::HashMap;
use regex::Regex;

/// EntityClassifier predicts candidate sensitive data types for text
pub struct EntityClassifier {
    patterns: HashMap<String, Regex>,
}

impl EntityClassifier {
    /// Create a new EntityClassifier
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Load built-in patterns
        for (name, pattern) in BUILTIN_PATTERNS {
            if let Ok(regex) = Regex::new(pattern) {
                patterns.insert(name.to_string(), regex);
            } else {
                warn!("Failed to compile built-in pattern for {}: {}", name, pattern);
            }
        }

        Self { patterns }
    }

    /// Predict candidate entity types for a list of texts
    pub async fn predict(&self, texts: &[String]) -> Result<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();

        for text in texts {
            let candidates = self.predict_single(text).await?;
            results.insert(text.clone(), candidates);
        }

        Ok(results)
    }

    /// Predict candidate entity types for a single text
    async fn predict_single(&self, text: &str) -> Result<Vec<String>> {
        let mut candidates = Vec::new();
        let normalized_text = normalize_text(text);

        // Quick filtering based on text characteristics
        if normalized_text.is_empty() {
            return Ok(candidates);
        }

        // Check each pattern
        for (entity_type, pattern) in &self.patterns {
            if pattern.is_match(&normalized_text) {
                candidates.push(entity_type.clone());
            }
        }

        // Add heuristic-based candidates
        candidates.extend(self.predict_heuristic(&normalized_text));

        // Remove duplicates
        candidates.sort();
        candidates.dedup();

        Ok(candidates)
    }

    /// Predict candidate types using heuristics
    fn predict_heuristic(&self, text: &str) -> Vec<String> {
        let mut candidates = Vec::new();

        // Length-based heuristics
        match text.len() {
            18 => candidates.push("ID_CARD".to_string()),
            11 => candidates.push("MOBILE_PHONE".to_string()),
            16..=19 => candidates.push("BANK_CARD".to_string()),
            6 => candidates.push("POSTCODE".to_string()),
            _ => {}
        }

        // Character pattern heuristics
        if text.contains('@') {
            candidates.push("EMAIL".to_string());
        }

        if text.contains('.') && text.chars().all(|c| c.is_ascii_digit() || c == '.') {
            candidates.push("IPV4".to_string());
        }

        if text.contains(':') && text.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
            candidates.push("IPV6".to_string());
        }

        if text.contains('-') || text.contains(':') {
            if text.len() == 17 {
                candidates.push("MAC".to_string());
            }
        }

        // Date patterns
        if text.contains('/') || text.contains('-') {
            if text.len() >= 8 && text.len() <= 10 {
                candidates.push("DATE".to_string());
            }
        }

        // Name heuristics (contains Chinese characters or common name patterns)
        if text.chars().any(|c| c as u32 >= 0x4e00 && c as u32 <= 0x9fff) {
            candidates.push("PERSON".to_string());
            candidates.push("COMPANY_NAME".to_string());
            candidates.push("LOCATION".to_string());
        }

        candidates
    }

    /// Add custom patterns
    pub fn add_custom_patterns(&mut self, patterns: &[RegexPattern]) {
        for pattern in patterns {
            match Regex::new(&pattern.pattern) {
                Ok(regex) => {
                    self.patterns.insert(pattern.name.clone(), regex);
                }
                Err(e) => {
                    warn!("Failed to compile custom pattern {}: {}", pattern.name, e);
                }
            }
        }
    }
}

impl Default for EntityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_classification() {
        let classifier = EntityClassifier::new();
        let candidates = classifier.predict_single("test@example.com").await.unwrap();
        assert!(candidates.contains(&"EMAIL".to_string()));
    }

    #[tokio::test]
    async fn test_mobile_phone_classification() {
        let classifier = EntityClassifier::new();
        let candidates = classifier.predict_single("13812345678").await.unwrap();
        assert!(candidates.contains(&"MOBILE_PHONE".to_string()));
    }

    #[tokio::test]
    async fn test_id_card_classification() {
        let classifier = EntityClassifier::new();
        let candidates = classifier.predict_single("123456789012345678").await.unwrap();
        assert!(candidates.contains(&"ID_CARD".to_string()));
    }

    #[tokio::test]
    async fn test_ipv4_classification() {
        let classifier = EntityClassifier::new();
        let candidates = classifier.predict_single("192.168.1.1").await.unwrap();
        assert!(candidates.contains(&"IPV4".to_string()));
    }

    #[tokio::test]
    async fn test_empty_text() {
        let classifier = EntityClassifier::new();
        let candidates = classifier.predict_single("").await.unwrap();
        assert!(candidates.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_texts() {
        let classifier = EntityClassifier::new();
        let texts = vec![
            "test@example.com".to_string(),
            "13812345678".to_string(),
            "regular text".to_string(),
        ];
        let results = classifier.predict(&texts).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_custom_patterns() {
        let mut classifier = EntityClassifier::new();
        let patterns = vec![
            RegexPattern {
                name: "TEST_PATTERN".to_string(),
                pattern: r"^test_\d+$".to_string(),
                description: Some("Test pattern".to_string()),
            }
        ];
        classifier.add_custom_patterns(&patterns);
        assert!(classifier.patterns.contains_key("TEST_PATTERN"));
    }
} 