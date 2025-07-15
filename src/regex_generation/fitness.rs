use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::regex_generation::dataset::Dataset;
use crate::regex_generation::generator::{RegexIndividual, FitnessScore};
use regex::Regex;

/// Fitness evaluation for regex individuals
pub struct Fitness {
    // Configuration could be added here
}

impl Fitness {
    /// Create a new Fitness evaluator
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate fitness of a regex individual against a dataset
    pub async fn evaluate(&self, individual: &RegexIndividual, dataset: &Dataset) -> Result<FitnessScore> {
        // Compile the regex pattern
        let regex = match Regex::new(&individual.pattern) {
            Ok(r) => r,
            Err(_) => {
                // Invalid regex pattern gets zero fitness
                return Ok(FitnessScore::new(0, 0, dataset.positive_examples().len(), dataset.negative_examples().len()));
            }
        };

        // Test against positive examples
        let mut matched_positive = 0;
        for example in dataset.positive_examples() {
            if regex.is_match(example) {
                matched_positive += 1;
            }
        }

        // Test against negative examples
        let mut matched_negative = 0;
        for example in dataset.negative_examples() {
            if regex.is_match(example) {
                matched_negative += 1;
            }
        }

        let total_positive = dataset.positive_examples().len();
        let total_negative = dataset.negative_examples().len();

        Ok(FitnessScore::new(
            matched_positive,
            matched_negative,
            total_positive,
            total_negative,
        ))
    }

    /// Evaluate fitness with custom weights
    pub async fn evaluate_weighted(
        &self,
        individual: &RegexIndividual,
        dataset: &Dataset,
        precision_weight: f64,
        recall_weight: f64,
    ) -> Result<f64> {
        let fitness = self.evaluate(individual, dataset).await?;
        
        // Weighted fitness score
        let weighted_score = fitness.precision * precision_weight + fitness.recall * recall_weight;
        Ok(weighted_score)
    }

    /// Evaluate multiple individuals in parallel
    pub async fn evaluate_batch(
        &self,
        individuals: &[RegexIndividual],
        dataset: &Dataset,
    ) -> Result<Vec<FitnessScore>> {
        let mut results = Vec::new();
        
        for individual in individuals {
            let fitness = self.evaluate(individual, dataset).await?;
            results.push(fitness);
        }
        
        Ok(results)
    }

    /// Get detailed fitness report
    pub async fn detailed_report(
        &self,
        individual: &RegexIndividual,
        dataset: &Dataset,
    ) -> Result<FitnessReport> {
        let fitness = self.evaluate(individual, dataset).await?;
        
        // Additional analysis
        let pattern_complexity = self.calculate_pattern_complexity(&individual.pattern);
        let generalization_score = self.calculate_generalization_score(individual, dataset).await?;
        
        Ok(FitnessReport {
            fitness,
            pattern_complexity,
            generalization_score,
            pattern: individual.pattern.clone(),
        })
    }

    /// Calculate pattern complexity (simpler patterns are better)
    fn calculate_pattern_complexity(&self, pattern: &str) -> f64 {
        let length_penalty = pattern.len() as f64 * 0.01;
        let special_chars = pattern.chars().filter(|&c| ".*+?{}[]()^$|\\".contains(c)).count() as f64;
        let complexity = length_penalty + special_chars * 0.1;
        
        // Normalize to 0-1 range
        complexity / (complexity + 1.0)
    }

    /// Calculate generalization score (how well it might work on unseen data)
    async fn calculate_generalization_score(&self, individual: &RegexIndividual, dataset: &Dataset) -> Result<f64> {
        // Simple heuristic: patterns that are too specific (match exactly) get lower scores
        let regex = match Regex::new(&individual.pattern) {
            Ok(r) => r,
            Err(_) => return Ok(0.0),
        };

        let mut exact_matches = 0;
        let total_positive = dataset.positive_examples().len();
        
        for example in dataset.positive_examples() {
            if regex.find(example).map_or(false, |m| m.as_str() == example) {
                exact_matches += 1;
            }
        }

        // Lower score for patterns that match entire strings exactly
        let exact_match_ratio = exact_matches as f64 / total_positive as f64;
        let generalization_score = 1.0 - exact_match_ratio * 0.5;
        
        Ok(generalization_score)
    }
}

impl Default for Fitness {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed fitness report
#[derive(Debug, Clone)]
pub struct FitnessReport {
    pub fitness: FitnessScore,
    pub pattern_complexity: f64,
    pub generalization_score: f64,
    pub pattern: String,
}

impl FitnessReport {
    /// Get overall quality score
    pub fn overall_quality(&self) -> f64 {
        let fitness_score = self.fitness.f1_score;
        let complexity_bonus = 1.0 - self.pattern_complexity;
        let generalization_bonus = self.generalization_score;
        
        // Weighted combination
        fitness_score * 0.6 + complexity_bonus * 0.2 + generalization_bonus * 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fitness_evaluation() {
        let fitness = Fitness::new();
        
        let training_data = TrainingData {
            positive: vec![
                "test@example.com".to_string(),
                "user@domain.org".to_string(),
            ],
            negative: vec![
                "not an email".to_string(),
                "invalid format".to_string(),
            ],
        };
        
        let dataset = Dataset::new(training_data);
        let individual = RegexIndividual::new(r".+@.+\..+".to_string(), 1);
        
        let score = fitness.evaluate(&individual, &dataset).await.unwrap();
        assert!(score.precision > 0.0);
        assert!(score.recall > 0.0);
    }

    #[tokio::test]
    async fn test_invalid_regex_fitness() {
        let fitness = Fitness::new();
        
        let training_data = TrainingData {
            positive: vec!["test".to_string()],
            negative: vec!["other".to_string()],
        };
        
        let dataset = Dataset::new(training_data);
        let individual = RegexIndividual::new("[invalid".to_string(), 1); // Invalid regex
        
        let score = fitness.evaluate(&individual, &dataset).await.unwrap();
        assert_eq!(score.precision, 0.0);
        assert_eq!(score.recall, 0.0);
    }

    #[tokio::test]
    async fn test_weighted_evaluation() {
        let fitness = Fitness::new();
        
        let training_data = TrainingData {
            positive: vec!["test@example.com".to_string()],
            negative: vec!["not email".to_string()],
        };
        
        let dataset = Dataset::new(training_data);
        let individual = RegexIndividual::new(r".+@.+\..+".to_string(), 1);
        
        let weighted_score = fitness.evaluate_weighted(&individual, &dataset, 0.7, 0.3).await.unwrap();
        assert!(weighted_score > 0.0);
    }

    #[test]
    fn test_pattern_complexity() {
        let fitness = Fitness::new();
        
        let simple_pattern = "abc";
        let complex_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        
        let simple_complexity = fitness.calculate_pattern_complexity(simple_pattern);
        let complex_complexity = fitness.calculate_pattern_complexity(complex_pattern);
        
        assert!(simple_complexity < complex_complexity);
    }

    #[tokio::test]
    async fn test_detailed_report() {
        let fitness = Fitness::new();
        
        let training_data = TrainingData {
            positive: vec!["test@example.com".to_string()],
            negative: vec!["not email".to_string()],
        };
        
        let dataset = Dataset::new(training_data);
        let individual = RegexIndividual::new(r".+@.+\..+".to_string(), 1);
        
        let report = fitness.detailed_report(&individual, &dataset).await.unwrap();
        assert!(report.overall_quality() > 0.0);
        assert!(!report.pattern.is_empty());
    }
} 