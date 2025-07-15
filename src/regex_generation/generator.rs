use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::regex_generation::dataset::Dataset;
use crate::regex_generation::evolution::Evolution;
use crate::regex_generation::fitness::Fitness;
use crate::regex_generation::bpe::BPE;
use std::collections::HashMap;

/// RegexGenerator generates regex patterns from positive and negative examples
pub struct RegexGenerator {
    config: GeneratorConfig,
}

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub max_depth: usize,
    pub bpe_pair_threshold: f64,
    pub bpe_char_threshold: f64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            max_depth: MAX_REGEX_TREE_DEPTH,
            bpe_pair_threshold: BPE_PAIR_PERCENT_THRESHOLD,
            bpe_char_threshold: BPE_CHAR_PERCENT_THRESHOLD,
        }
    }
}

impl RegexGenerator {
    /// Create a new RegexGenerator with default configuration
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
        }
    }

    /// Create a new RegexGenerator with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate a regex pattern from training data
    pub async fn generate(
        &self,
        regex_name: &str,
        training_data: &TrainingData,
        params: &EvolutionParams,
    ) -> Result<RegexGenerationResult> {
        info!("Starting regex generation for: {}", regex_name);

        // Create dataset from training data
        let mut dataset = Dataset::new(training_data.clone());
        dataset.build()?;

        let original_positive_count = dataset.positive_examples().len();
        info!("Original positive examples: {}", original_positive_count);

        // Learn BPE tokens
        let bpe_tokens = self.learn_bpe_tokens(&dataset).await?;
        info!("Learned {} BPE tokens", bpe_tokens.len());

        // Initialize evolution
        let mut evolution = Evolution::new(params.clone());
        let mut population = evolution.initialize_population(
            &dataset,
            &bpe_tokens,
            params.init_population_size,
        )?;

        let mut result_patterns = Vec::new();
        let mut iteration_count = 0;

        // Main evolution loop
        for generation in 0..params.max_iterations {
            info!("Generation {}: Population size: {}", generation, population.len());

            // Evaluate fitness
            let fitness_scores = self.evaluate_population(&population, &dataset).await?;

            // Find best individual
            let best_individual = self.find_best_individual(&population, &fitness_scores)?;
            let best_precision = fitness_scores[best_individual.0].precision;

            info!("Generation {}: Best precision: {:.4}", generation, best_precision);

            iteration_count += 1;

            // Check if we should extract this pattern
            if best_precision >= params.precision_divide_conquer
                && iteration_count >= params.iteration_divide_conquer
            {
                let pattern = population[best_individual.0].to_regex_string();
                result_patterns.push(pattern.clone());
                info!("Extracted pattern: {}", pattern);

                // Remove matched positive examples
                dataset.remove_matched_examples(&population[best_individual.0])?;

                // Check if we should stop
                let remaining_ratio = dataset.positive_examples().len() as f64 / original_positive_count as f64;
                if remaining_ratio < params.noise_positive_sample_ratio {
                    info!("Stopping: remaining ratio {} < threshold {}", remaining_ratio, params.noise_positive_sample_ratio);
                    break;
                }

                // Reinitialize population
                population = evolution.initialize_population(
                    &dataset,
                    &bpe_tokens,
                    population.len(),
                )?;
                iteration_count = 0;
            } else {
                // Evolve population
                population = evolution.evolve_population(population, &fitness_scores)?;
            }
        }

        // Combine patterns
        let final_pattern = if result_patterns.is_empty() {
            ".*".to_string() // Fallback pattern
        } else {
            result_patterns.join("|")
        };

        info!("Final pattern for {}: {}", regex_name, final_pattern);

        Ok(RegexGenerationResult {
            regex_name: regex_name.to_string(),
            regex_pattern: final_pattern,
        })
    }

    /// Learn BPE tokens from dataset
    async fn learn_bpe_tokens(&self, dataset: &Dataset) -> Result<HashMap<String, f64>> {
        let bpe = BPE::new(self.config.bpe_pair_threshold, self.config.bpe_char_threshold);
        bpe.learn_tokens(dataset.positive_examples()).await
    }

    /// Evaluate fitness for entire population
    async fn evaluate_population(
        &self,
        population: &[RegexIndividual],
        dataset: &Dataset,
    ) -> Result<Vec<FitnessScore>> {
        let mut fitness_scores = Vec::new();

        for individual in population {
            let fitness = Fitness::new();
            let score = fitness.evaluate(individual, dataset).await?;
            fitness_scores.push(score);
        }

        Ok(fitness_scores)
    }

    /// Find the best individual in the population
    fn find_best_individual(
        &self,
        population: &[RegexIndividual],
        fitness_scores: &[FitnessScore],
    ) -> Result<(usize, FitnessScore)> {
        if population.is_empty() {
            return Err(FastDlpError::analysis_error("Empty population"));
        }

        let mut best_idx = 0;
        let mut best_score = fitness_scores[0].clone();

        for (i, score) in fitness_scores.iter().enumerate() {
            if score.is_better_than(&best_score) {
                best_idx = i;
                best_score = score.clone();
            }
        }

        Ok((best_idx, best_score))
    }
}

impl Default for RegexGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a regex individual in the population
#[derive(Debug, Clone)]
pub struct RegexIndividual {
    pub pattern: String,
    pub depth: usize,
}

impl RegexIndividual {
    pub fn new(pattern: String, depth: usize) -> Self {
        Self { pattern, depth }
    }

    pub fn to_regex_string(&self) -> String {
        self.pattern.clone()
    }
}

/// Represents fitness score for a regex individual
#[derive(Debug, Clone)]
pub struct FitnessScore {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub matched_positive: usize,
    pub matched_negative: usize,
    pub total_positive: usize,
    pub total_negative: usize,
}

impl FitnessScore {
    pub fn new(
        matched_positive: usize,
        matched_negative: usize,
        total_positive: usize,
        total_negative: usize,
    ) -> Self {
        let precision = if matched_positive + matched_negative > 0 {
            matched_positive as f64 / (matched_positive + matched_negative) as f64
        } else {
            0.0
        };

        let recall = if total_positive > 0 {
            matched_positive as f64 / total_positive as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        Self {
            precision,
            recall,
            f1_score,
            matched_positive,
            matched_negative,
            total_positive,
            total_negative,
        }
    }

    pub fn is_better_than(&self, other: &FitnessScore) -> bool {
        // Primary: higher precision
        if self.precision > other.precision {
            return true;
        }
        if self.precision < other.precision {
            return false;
        }

        // Secondary: higher recall
        if self.recall > other.recall {
            return true;
        }
        if self.recall < other.recall {
            return false;
        }

        // Tertiary: higher F1 score
        self.f1_score > other.f1_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_generator_basic() {
        let generator = RegexGenerator::new();
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

        let params = EvolutionParams {
            max_iterations: 10,
            init_population_size: 20,
            ..Default::default()
        };

        let result = generator.generate("TEST_EMAIL", &training_data, &params).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.regex_name, "TEST_EMAIL");
        assert!(!result.regex_pattern.is_empty());
    }

    #[test]
    fn test_fitness_score_calculation() {
        let score = FitnessScore::new(8, 2, 10, 5);
        assert_eq!(score.precision, 0.8); // 8 / (8 + 2)
        assert_eq!(score.recall, 0.8); // 8 / 10
        assert!((score.f1_score - 0.8).abs() < 0.001); // 2 * 0.8 * 0.8 / (0.8 + 0.8)
    }

    #[test]
    fn test_fitness_score_comparison() {
        let score1 = FitnessScore::new(9, 1, 10, 5);
        let score2 = FitnessScore::new(8, 2, 10, 5);
        
        assert!(score1.is_better_than(&score2)); // Higher precision
    }

    #[test]
    fn test_regex_individual() {
        let individual = RegexIndividual::new("test.*".to_string(), 3);
        assert_eq!(individual.pattern, "test.*");
        assert_eq!(individual.depth, 3);
        assert_eq!(individual.to_regex_string(), "test.*");
    }
} 