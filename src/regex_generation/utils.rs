use crate::common::*;
use crate::error::Result;
use std::collections::HashMap;

/// Utility functions for regex generation
pub mod regex_utils {
    use super::*;

    /// Escape special regex characters
    pub fn escape_regex(text: &str) -> String {
        regex::escape(text)
    }

    /// Check if a string is a valid regex pattern
    pub fn is_valid_regex(pattern: &str) -> bool {
        regex::Regex::new(pattern).is_ok()
    }

    /// Simplify a regex pattern by removing redundant elements
    pub fn simplify_pattern(pattern: &str) -> String {
        let mut simplified = pattern.to_string();
        
        // Remove redundant parentheses
        simplified = simplified.replace("((", "(");
        simplified = simplified.replace("))", ")");
        
        // Simplify multiple dots
        simplified = simplified.replace("..+", ".+");
        simplified = simplified.replace(".*.*", ".*");
        
        // Remove empty alternations
        simplified = simplified.replace("(|)", "");
        simplified = simplified.replace("()", "");
        
        simplified
    }

    /// Calculate pattern complexity score
    pub fn pattern_complexity(pattern: &str) -> f64 {
        let length_score = pattern.len() as f64 * 0.1;
        let special_chars = pattern.chars().filter(|&c| ".*+?{}[]()^$|\\".contains(c)).count() as f64;
        let nesting_level = calculate_nesting_level(pattern);
        
        length_score + special_chars + nesting_level * 2.0
    }

    /// Calculate nesting level of parentheses
    fn calculate_nesting_level(pattern: &str) -> f64 {
        let mut max_depth = 0;
        let mut current_depth = 0;
        
        for ch in pattern.chars() {
            match ch {
                '(' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                ')' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        
        max_depth as f64
    }

    /// Extract common patterns from a list of strings
    pub fn extract_common_patterns(strings: &[String]) -> Vec<String> {
        let mut patterns = Vec::new();
        
        if strings.is_empty() {
            return patterns;
        }

        // Find common prefixes
        if let Some(common_prefix) = find_common_prefix(strings) {
            if common_prefix.len() > 2 {
                patterns.push(format!("^{}", escape_regex(&common_prefix)));
            }
        }

        // Find common suffixes
        if let Some(common_suffix) = find_common_suffix(strings) {
            if common_suffix.len() > 2 {
                patterns.push(format!("{}$", escape_regex(&common_suffix)));
            }
        }

        // Find common substrings
        let common_substrings = find_common_substrings(strings, 3);
        for substring in common_substrings {
            patterns.push(escape_regex(&substring));
        }

        patterns
    }

    /// Find common prefix among strings
    fn find_common_prefix(strings: &[String]) -> Option<String> {
        if strings.is_empty() {
            return None;
        }

        let first = &strings[0];
        let mut prefix_len = 0;

        for (i, ch) in first.chars().enumerate() {
            if strings.iter().all(|s| s.chars().nth(i) == Some(ch)) {
                prefix_len = i + 1;
            } else {
                break;
            }
        }

        if prefix_len > 0 {
            Some(first[..prefix_len].to_string())
        } else {
            None
        }
    }

    /// Find common suffix among strings
    fn find_common_suffix(strings: &[String]) -> Option<String> {
        if strings.is_empty() {
            return None;
        }

        let first = &strings[0];
        let mut suffix_len = 0;

        for (i, ch) in first.chars().rev().enumerate() {
            if strings.iter().all(|s| {
                s.chars().rev().nth(i) == Some(ch)
            }) {
                suffix_len = i + 1;
            } else {
                break;
            }
        }

        if suffix_len > 0 {
            let start = first.len() - suffix_len;
            Some(first[start..].to_string())
        } else {
            None
        }
    }

    /// Find common substrings
    fn find_common_substrings(strings: &[String], min_length: usize) -> Vec<String> {
        let mut substring_count = HashMap::new();
        
        for string in strings {
            let mut seen = std::collections::HashSet::new();
            for i in 0..=string.len().saturating_sub(min_length) {
                for j in i + min_length..=string.len() {
                    let substring = string[i..j].to_string();
                    if seen.insert(substring.clone()) {
                        *substring_count.entry(substring).or_insert(0) += 1;
                    }
                }
            }
        }

        substring_count.into_iter()
            .filter(|(_, count)| *count >= strings.len() / 2) // At least half of strings
            .map(|(substring, _)| substring)
            .collect()
    }
}

/// Pattern generation utilities
pub mod pattern_gen {
    use super::*;

    /// Generate character class from a set of characters
    pub fn char_class_from_chars(chars: &[char]) -> String {
        if chars.is_empty() {
            return ".".to_string();
        }

        let mut sorted_chars = chars.to_vec();
        sorted_chars.sort();
        sorted_chars.dedup();

        format!("[{}]", sorted_chars.iter().collect::<String>())
    }

    /// Generate quantifier based on frequency
    pub fn quantifier_from_frequency(min_count: usize, max_count: usize) -> String {
        match (min_count, max_count) {
            (0, 1) => "?".to_string(),
            (0, _) => "*".to_string(),
            (1, _) if max_count > 10 => "+".to_string(),
            (n, m) if n == m => format!("{{{}}}", n),
            (n, m) => format!("{{{},{}}}", n, m),
        }
    }

    /// Generate pattern for numeric sequences
    pub fn numeric_pattern(examples: &[String]) -> Option<String> {
        if examples.is_empty() {
            return None;
        }

        // Check if all examples are numeric
        if !examples.iter().all(|s| s.chars().all(|c| c.is_ascii_digit())) {
            return None;
        }

        // Analyze lengths
        let lengths: Vec<usize> = examples.iter().map(|s| s.len()).collect();
        let min_len = *lengths.iter().min().unwrap();
        let max_len = *lengths.iter().max().unwrap();

        if min_len == max_len {
            Some(format!(r"\d{{{}}}", min_len))
        } else {
            Some(format!(r"\d{{{},{}}}", min_len, max_len))
        }
    }

    /// Generate pattern for email-like strings
    pub fn email_pattern(examples: &[String]) -> Option<String> {
        if examples.is_empty() {
            return None;
        }

        // Check if all examples contain @ and .
        if !examples.iter().all(|s| s.contains('@') && s.contains('.')) {
            return None;
        }

        Some(r".+@.+\..+".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_regex() {
        assert_eq!(regex_utils::escape_regex("hello.world"), "hello\\.world");
        assert_eq!(regex_utils::escape_regex("a+b*c?"), "a\\+b\\*c\\?");
    }

    #[test]
    fn test_is_valid_regex() {
        assert!(regex_utils::is_valid_regex("hello"));
        assert!(regex_utils::is_valid_regex(r"\d+"));
        assert!(!regex_utils::is_valid_regex("[invalid"));
    }

    #[test]
    fn test_simplify_pattern() {
        assert_eq!(regex_utils::simplify_pattern("((hello))"), "(hello)");
        assert_eq!(regex_utils::simplify_pattern(".*.*"), ".*");
        assert_eq!(regex_utils::simplify_pattern("(|)"), "");
    }

    #[test]
    fn test_pattern_complexity() {
        let simple = regex_utils::pattern_complexity("hello");
        let complex = regex_utils::pattern_complexity(r"^(?:hello|world)\d+$");
        assert!(simple < complex);
    }

    #[test]
    fn test_common_prefix() {
        let strings = vec![
            "hello world".to_string(),
            "hello there".to_string(),
            "hello everyone".to_string(),
        ];
        let patterns = regex_utils::extract_common_patterns(&strings);
        assert!(patterns.iter().any(|p| p.contains("hello")));
    }

    #[test]
    fn test_char_class_generation() {
        let chars = vec!['a', 'b', 'c'];
        let pattern = pattern_gen::char_class_from_chars(&chars);
        assert_eq!(pattern, "[abc]");
    }

    #[test]
    fn test_quantifier_generation() {
        assert_eq!(pattern_gen::quantifier_from_frequency(0, 1), "?");
        assert_eq!(pattern_gen::quantifier_from_frequency(1, 100), "+");
        assert_eq!(pattern_gen::quantifier_from_frequency(3, 3), "{3}");
        assert_eq!(pattern_gen::quantifier_from_frequency(2, 5), "{2,5}");
    }

    #[test]
    fn test_numeric_pattern() {
        let examples = vec!["123".to_string(), "456".to_string(), "789".to_string()];
        let pattern = pattern_gen::numeric_pattern(&examples);
        assert_eq!(pattern, Some(r"\d{3}".to_string()));
    }

    #[test]
    fn test_email_pattern() {
        let examples = vec![
            "test@example.com".to_string(),
            "user@domain.org".to_string(),
        ];
        let pattern = pattern_gen::email_pattern(&examples);
        assert_eq!(pattern, Some(r".+@.+\..+".to_string()));
    }
} 