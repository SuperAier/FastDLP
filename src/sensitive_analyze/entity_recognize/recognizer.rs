use crate::common::*;
use crate::error::{FastDlpError, Result};
use std::collections::HashMap;
use regex::Regex;

/// EntityRecognizer performs actual recognition of sensitive data in text
pub struct EntityRecognizer {
    builtin_patterns: HashMap<String, Regex>,
    custom_patterns: HashMap<String, Regex>,
}

impl EntityRecognizer {
    /// Create a new EntityRecognizer
    pub fn new() -> Self {
        let mut builtin_patterns = HashMap::new();
        
        // Load built-in patterns
        for (name, pattern) in BUILTIN_PATTERNS {
            if let Ok(regex) = Regex::new(pattern) {
                builtin_patterns.insert(name.to_string(), regex);
            } else {
                warn!("Failed to compile built-in pattern for {}: {}", name, pattern);
            }
        }

        Self {
            builtin_patterns,
            custom_patterns: HashMap::new(),
        }
    }

    /// Set custom patterns
    pub fn set_custom_patterns(&mut self, patterns: Vec<RegexPattern>) {
        self.custom_patterns.clear();
        
        for pattern in patterns {
            match Regex::new(&pattern.pattern) {
                Ok(regex) => {
                    self.custom_patterns.insert(pattern.name.clone(), regex);
                }
                Err(e) => {
                    warn!("Failed to compile custom pattern {}: {}", pattern.name, e);
                }
            }
        }
    }

    /// Analyze texts for a specific entity type
    pub async fn analyze(&self, texts: &[String], entity_type: &str) -> Result<Vec<Option<String>>> {
        let mut results = Vec::new();

        // Get the pattern for this entity type
        let pattern = self.builtin_patterns.get(entity_type);
        
        if let Some(pattern) = pattern {
            for text in texts {
                let result = self.recognize_single(text, entity_type, pattern).await?;
                results.push(result);
            }
        } else {
            // Return None for all texts if pattern not found
            results.resize(texts.len(), None);
        }

        Ok(results)
    }

    /// Analyze texts with a specific custom pattern
    pub async fn analyze_with_pattern(&self, texts: &[String], pattern: &RegexPattern) -> Result<Vec<Option<String>>> {
        let mut results = Vec::new();

        match Regex::new(&pattern.pattern) {
            Ok(regex) => {
                for text in texts {
                    let result = self.recognize_single(text, &pattern.name, &regex).await?;
                    results.push(result);
                }
            }
            Err(e) => {
                warn!("Failed to compile pattern {}: {}", pattern.name, e);
                results.resize(texts.len(), None);
            }
        }

        Ok(results)
    }

    /// Recognize a single text with a specific pattern
    async fn recognize_single(&self, text: &str, entity_type: &str, pattern: &Regex) -> Result<Option<String>> {
        let normalized_text = normalize_text(text);

        if normalized_text.is_empty() {
            return Ok(None);
        }

        // Check if pattern matches
        if pattern.is_match(&normalized_text) {
            // Additional validation based on entity type
            if self.validate_entity(&normalized_text, entity_type).await? {
                return Ok(Some(entity_type.to_string()));
            }
        }

        Ok(None)
    }

    /// Validate detected entity with additional checks
    async fn validate_entity(&self, text: &str, entity_type: &str) -> Result<bool> {
        match entity_type {
            "BANK_CARD" => Ok(self.validate_bank_card(text)),
            "ID_CARD" => Ok(self.validate_id_card(text)),
            "MOBILE_PHONE" => Ok(self.validate_mobile_phone(text)),
            "EMAIL" => Ok(self.validate_email(text)),
            "IPV4" => Ok(self.validate_ipv4(text)),
            "IPV6" => Ok(self.validate_ipv6(text)),
            "DATE" => Ok(self.validate_date(text)),
            _ => Ok(true), // Default validation passes
        }
    }

    /// Validate bank card using Luhn algorithm
    fn validate_bank_card(&self, text: &str) -> bool {
        if text.len() < 16 || text.len() > 19 {
            return false;
        }

        // Check if all characters are digits
        if !text.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Luhn algorithm validation
        let mut sum = 0;
        let mut double = false;

        for c in text.chars().rev() {
            if let Some(mut digit) = c.to_digit(10) {
                if double {
                    digit *= 2;
                    if digit > 9 {
                        digit -= 9;
                    }
                }
                sum += digit;
                double = !double;
            } else {
                return false;
            }
        }

        sum % 10 == 0
    }

    /// Validate Chinese ID card
    fn validate_id_card(&self, text: &str) -> bool {
        if text.len() != 18 {
            return false;
        }

        // Check first 17 characters are digits
        if !text[..17].chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Check last character is digit or 'X'/'x'
        let last_char = text.chars().last().unwrap();
        if !last_char.is_ascii_digit() && last_char != 'X' && last_char != 'x' {
            return false;
        }

        // Additional ID card validation logic could be added here
        // (birth date validation, checksum validation, etc.)
        true
    }

    /// Validate mobile phone number
    fn validate_mobile_phone(&self, text: &str) -> bool {
        if text.len() != 11 {
            return false;
        }

        // Must start with 1 and second digit should be 3-9
        if !text.starts_with('1') {
            return false;
        }

        if let Some(second_digit) = text.chars().nth(1) {
            if !matches!(second_digit, '3'..='9') {
                return false;
            }
        }

        // All characters must be digits
        text.chars().all(|c| c.is_ascii_digit())
    }

    /// Validate email format
    fn validate_email(&self, text: &str) -> bool {
        // Basic email validation
        if !text.contains('@') {
            return false;
        }

        let parts: Vec<&str> = text.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Local part validation
        if local.is_empty() || local.len() > 64 {
            return false;
        }

        // Domain part validation
        if domain.is_empty() || !domain.contains('.') {
            return false;
        }

        true
    }

    /// Validate IPv4 address
    fn validate_ipv4(&self, text: &str) -> bool {
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        for part in parts {
            if let Ok(num) = part.parse::<u32>() {
                if num > 255 {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Validate IPv6 address
    fn validate_ipv6(&self, text: &str) -> bool {
        // Simple IPv6 validation - can be enhanced
        if text.is_empty() {
            return false;
        }

        // Check for valid IPv6 characters
        text.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
    }

    /// Validate date format
    fn validate_date(&self, text: &str) -> bool {
        // Basic date validation - can be enhanced with proper date parsing
        if text.len() < 8 || text.len() > 10 {
            return false;
        }

        // Check for date separators
        text.contains('/') || text.contains('-')
    }
}

impl Default for EntityRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_recognition() {
        let recognizer = EntityRecognizer::new();
        let texts = vec!["test@example.com".to_string()];
        let results = recognizer.analyze(&texts, "EMAIL").await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_some());
    }

    #[tokio::test]
    async fn test_mobile_phone_recognition() {
        let recognizer = EntityRecognizer::new();
        let texts = vec!["13812345678".to_string()];
        let results = recognizer.analyze(&texts, "MOBILE_PHONE").await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_some());
    }

    #[tokio::test]
    async fn test_invalid_mobile_phone() {
        let recognizer = EntityRecognizer::new();
        let texts = vec!["12345678901".to_string()]; // Invalid - doesn't start with 1[3-9]
        let results = recognizer.analyze(&texts, "MOBILE_PHONE").await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_none());
    }

    #[tokio::test]
    async fn test_custom_pattern_recognition() {
        let mut recognizer = EntityRecognizer::new();
        let patterns = vec![
            RegexPattern {
                name: "TEST_PATTERN".to_string(),
                pattern: r"^test_\d+$".to_string(),
                description: Some("Test pattern".to_string()),
            }
        ];
        recognizer.set_custom_patterns(patterns.clone());

        let texts = vec!["test_123".to_string(), "test_abc".to_string()];
        let results = recognizer.analyze_with_pattern(&texts, &patterns[0]).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_some());
        assert!(results[1].is_none());
    }

    #[test]
    fn test_bank_card_validation() {
        let recognizer = EntityRecognizer::new();
        
        // Valid bank card (passes Luhn check)
        assert!(recognizer.validate_bank_card("4111111111111111"));
        
        // Invalid bank card (fails Luhn check)
        assert!(!recognizer.validate_bank_card("4111111111111112"));
        
        // Too short
        assert!(!recognizer.validate_bank_card("411111111111111"));
        
        // Too long
        assert!(!recognizer.validate_bank_card("41111111111111111111"));
        
        // Contains non-digits
        assert!(!recognizer.validate_bank_card("4111111111111a11"));
    }

    #[test]
    fn test_id_card_validation() {
        let recognizer = EntityRecognizer::new();
        
        // Valid format
        assert!(recognizer.validate_id_card("123456789012345678"));
        assert!(recognizer.validate_id_card("12345678901234567X"));
        
        // Invalid format
        assert!(!recognizer.validate_id_card("1234567890123456789")); // Too long
        assert!(!recognizer.validate_id_card("12345678901234567")); // Too short
        assert!(!recognizer.validate_id_card("1234567890123456a8")); // Invalid character
    }

    #[test]
    fn test_ipv4_validation() {
        let recognizer = EntityRecognizer::new();
        
        // Valid IPv4
        assert!(recognizer.validate_ipv4("192.168.1.1"));
        assert!(recognizer.validate_ipv4("0.0.0.0"));
        assert!(recognizer.validate_ipv4("255.255.255.255"));
        
        // Invalid IPv4
        assert!(!recognizer.validate_ipv4("256.1.1.1")); // Out of range
        assert!(!recognizer.validate_ipv4("192.168.1")); // Too few parts
        assert!(!recognizer.validate_ipv4("192.168.1.1.1")); // Too many parts
        assert!(!recognizer.validate_ipv4("192.168.1.a")); // Non-numeric
    }
} 