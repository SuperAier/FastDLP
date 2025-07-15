use crate::common::*;
use crate::error::{FastDlpError, Result};
use std::collections::HashMap;

/// BPE (Byte Pair Encoding) implementation for regex generation
pub struct BPE {
    pair_threshold: f64,
    char_threshold: f64,
}

impl BPE {
    /// Create a new BPE instance
    pub fn new(pair_threshold: f64, char_threshold: f64) -> Self {
        Self {
            pair_threshold,
            char_threshold,
        }
    }

    /// Learn BPE tokens from positive examples
    pub async fn learn_tokens(&self, examples: &[String]) -> Result<HashMap<String, f64>> {
        let mut tokens = HashMap::new();
        
        if examples.is_empty() {
            return Ok(tokens);
        }

        // Learn character-level tokens
        let char_tokens = self.learn_char_tokens(examples);
        tokens.extend(char_tokens);

        // Learn pair-level tokens
        let pair_tokens = self.learn_pair_tokens(examples);
        tokens.extend(pair_tokens);

        // Learn substring tokens
        let substring_tokens = self.learn_substring_tokens(examples);
        tokens.extend(substring_tokens);

        info!("Learned {} BPE tokens", tokens.len());
        Ok(tokens)
    }

    /// Learn character-level tokens
    fn learn_char_tokens(&self, examples: &[String]) -> HashMap<String, f64> {
        let mut char_counts = HashMap::new();
        let mut total_chars = 0;

        // Count character frequencies
        for example in examples {
            for ch in example.chars() {
                *char_counts.entry(ch).or_insert(0) += 1;
                total_chars += 1;
            }
        }

        // Convert to frequencies and filter
        let mut tokens = HashMap::new();
        for (ch, count) in char_counts {
            let frequency = count as f64 / total_chars as f64;
            if frequency >= self.char_threshold {
                tokens.insert(ch.to_string(), frequency);
            }
        }

        tokens
    }

    /// Learn pair-level tokens
    fn learn_pair_tokens(&self, examples: &[String]) -> HashMap<String, f64> {
        let mut pair_counts = HashMap::new();
        let mut total_pairs = 0;

        // Count adjacent character pairs
        for example in examples {
            let chars: Vec<char> = example.chars().collect();
            for i in 0..chars.len().saturating_sub(1) {
                let pair = format!("{}{}", chars[i], chars[i + 1]);
                *pair_counts.entry(pair).or_insert(0) += 1;
                total_pairs += 1;
            }
        }

        // Convert to frequencies and filter
        let mut tokens = HashMap::new();
        for (pair, count) in pair_counts {
            let frequency = count as f64 / total_pairs as f64;
            if frequency >= self.pair_threshold {
                tokens.insert(pair, frequency);
            }
        }

        tokens
    }

    /// Learn substring tokens
    fn learn_substring_tokens(&self, examples: &[String]) -> HashMap<String, f64> {
        let mut substring_counts = HashMap::new();
        let mut total_substrings = 0;

        // Count substrings of length 3-5
        for example in examples {
            for len in 3..=5.min(example.len()) {
                for i in 0..=example.len().saturating_sub(len) {
                    let substring = example[i..i + len].to_string();
                    *substring_counts.entry(substring).or_insert(0) += 1;
                    total_substrings += 1;
                }
            }
        }

        // Convert to frequencies and filter
        let mut tokens = HashMap::new();
        for (substring, count) in substring_counts {
            let frequency = count as f64 / total_substrings as f64;
            if frequency >= self.pair_threshold && count >= 2 {
                tokens.insert(substring, frequency);
            }
        }

        tokens
    }

    /// Apply BPE tokenization to a string
    pub fn tokenize(&self, text: &str, tokens: &HashMap<String, f64>) -> Vec<String> {
        let mut result = Vec::new();
        let mut remaining = text.to_string();

        while !remaining.is_empty() {
            let mut best_match = None;
            let mut best_length = 0;

            // Find the longest matching token
            for (token, _) in tokens {
                if remaining.starts_with(token) && token.len() > best_length {
                    best_match = Some(token.clone());
                    best_length = token.len();
                }
            }

            if let Some(token) = best_match {
                result.push(token.clone());
                remaining = remaining[token.len()..].to_string();
            } else {
                // No token matches, take the first character
                let first_char = remaining.chars().next().unwrap();
                result.push(first_char.to_string());
                remaining = remaining[first_char.len_utf8()..].to_string();
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bpe_learning() {
        let bpe = BPE::new(0.1, 0.1);
        let examples = vec![
            "test@example.com".to_string(),
            "user@domain.org".to_string(),
            "admin@site.net".to_string(),
        ];

        let tokens = bpe.learn_tokens(&examples).await.unwrap();
        assert!(!tokens.is_empty());
        
        // Should learn common characters
        assert!(tokens.contains_key("@"));
        assert!(tokens.contains_key("."));
    }

    #[test]
    fn test_char_tokens() {
        let bpe = BPE::new(0.1, 0.1);
        let examples = vec![
            "aaa".to_string(),
            "aab".to_string(),
        ];

        let tokens = bpe.learn_char_tokens(&examples);
        assert!(tokens.contains_key("a"));
        assert!(tokens.contains_key("b"));
        
        // 'a' should have higher frequency than 'b'
        assert!(tokens.get("a").unwrap() > tokens.get("b").unwrap());
    }

    #[test]
    fn test_pair_tokens() {
        let bpe = BPE::new(0.1, 0.1);
        let examples = vec![
            "abc".to_string(),
            "abx".to_string(),
            "aby".to_string(),
        ];

        let tokens = bpe.learn_pair_tokens(&examples);
        assert!(tokens.contains_key("ab")); // Common pair
    }

    #[test]
    fn test_tokenize() {
        let bpe = BPE::new(0.1, 0.1);
        let mut tokens = HashMap::new();
        tokens.insert("test".to_string(), 1.0);
        tokens.insert("@".to_string(), 1.0);

        let result = bpe.tokenize("test@example", &tokens);
        assert_eq!(result[0], "test");
        assert_eq!(result[1], "@");
    }
} 