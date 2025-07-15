use crate::common::*;
use crate::error::Result;

/// Filter candidate entities based on various criteria
pub fn filter_entities(entities: &std::collections::HashMap<String, Vec<String>>) -> std::collections::HashMap<String, Vec<String>> {
    let mut filtered = std::collections::HashMap::new();
    
    for (text, candidates) in entities {
        let mut filtered_candidates = Vec::new();
        
        for candidate in candidates {
            // Basic filtering logic
            if is_likely_sensitive(text) {
                filtered_candidates.push(candidate.clone());
            }
        }
        
        filtered.insert(text.clone(), filtered_candidates);
    }
    
    filtered
}

/// Additional utility functions for sensitive data analysis
pub mod analyzer_utils {
    use super::*;
    
    /// Check if text appears to be sensitive data
    pub fn appears_sensitive(text: &str) -> bool {
        // Simple heuristics
        if text.is_empty() {
            return false;
        }
        
        // Check for common sensitive patterns
        if text.contains('@') || // Email
           text.len() == 18 || // ID card
           text.len() == 11 || // Mobile phone
           text.chars().all(|c| c.is_ascii_digit()) // Numeric
        {
            return true;
        }
        
        false
    }
    
    /// Calculate confidence score for detection
    pub fn calculate_confidence(matched: usize, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        matched as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_filter_entities() {
        let mut entities = HashMap::new();
        entities.insert("test@example.com".to_string(), vec!["EMAIL".to_string()]);
        entities.insert("".to_string(), vec!["EMAIL".to_string()]);
        
        let filtered = filter_entities(&entities);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.get("test@example.com").unwrap().contains(&"EMAIL".to_string()));
        assert!(filtered.get("").unwrap().is_empty());
    }
    
    #[test]
    fn test_appears_sensitive() {
        use analyzer_utils::appears_sensitive;
        
        assert!(appears_sensitive("test@example.com"));
        assert!(appears_sensitive("13812345678"));
        assert!(appears_sensitive("123456789012345678"));
        assert!(appears_sensitive("1234567890"));
        assert!(!appears_sensitive("regular text"));
        assert!(!appears_sensitive(""));
    }
    
    #[test]
    fn test_calculate_confidence() {
        use analyzer_utils::calculate_confidence;
        
        assert_eq!(calculate_confidence(9, 10), 0.9);
        assert_eq!(calculate_confidence(10, 10), 1.0);
        assert_eq!(calculate_confidence(0, 10), 0.0);
        assert_eq!(calculate_confidence(0, 0), 0.0);
    }
} 