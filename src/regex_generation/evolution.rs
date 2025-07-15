use crate::common::*;
use crate::error::{FastDlpError, Result};
use crate::regex_generation::dataset::Dataset;
use crate::regex_generation::generator::{RegexIndividual, FitnessScore};
use std::collections::HashMap;
use rand::Rng;

/// Evolution algorithm for regex generation
pub struct Evolution {
    params: EvolutionParams,
    rng: rand::rngs::ThreadRng,
}

impl Evolution {
    /// Create a new Evolution instance
    pub fn new(params: EvolutionParams) -> Self {
        Self {
            params,
            rng: rand::thread_rng(),
        }
    }

    /// Initialize population
    pub fn initialize_population(
        &mut self,
        dataset: &Dataset,
        bpe_tokens: &HashMap<String, f64>,
        population_size: usize,
    ) -> Result<Vec<RegexIndividual>> {
        let mut population = Vec::new();

        // Generate individuals based on dataset patterns
        for _ in 0..population_size {
            let individual = self.generate_individual(dataset, bpe_tokens)?;
            population.push(individual);
        }

        Ok(population)
    }

    /// Generate a single individual
    fn generate_individual(
        &mut self,
        dataset: &Dataset,
        bpe_tokens: &HashMap<String, f64>,
    ) -> Result<RegexIndividual> {
        let strategy = self.rng.gen_range(0..4);
        
        let pattern = match strategy {
            0 => self.generate_from_bpe_tokens(bpe_tokens),
            1 => self.generate_from_common_patterns(dataset),
            2 => self.generate_from_char_classes(dataset),
            _ => self.generate_random_pattern(),
        };

        Ok(RegexIndividual::new(pattern, 1))
    }

    /// Generate pattern from BPE tokens
    fn generate_from_bpe_tokens(&mut self, bpe_tokens: &HashMap<String, f64>) -> String {
        if bpe_tokens.is_empty() {
            return ".*".to_string();
        }

        let tokens: Vec<(&String, &f64)> = bpe_tokens.iter().collect();
        let num_tokens = self.rng.gen_range(1..=3.min(tokens.len()));
        
        let mut pattern = String::new();
        for _ in 0..num_tokens {
            let idx = self.rng.gen_range(0..tokens.len());
            let token = tokens[idx].0;
            pattern.push_str(&regex::escape(token));
        }

        pattern
    }

    /// Generate pattern from common patterns
    fn generate_from_common_patterns(&mut self, dataset: &Dataset) -> String {
        let positive_examples = dataset.positive_examples();
        if positive_examples.is_empty() {
            return ".*".to_string();
        }

        let idx = self.rng.gen_range(0..positive_examples.len());
        let example = &positive_examples[idx];
        
        // Generate pattern based on example characteristics
        if example.contains('@') {
            return r".+@.+\..+".to_string();
        } else if example.chars().all(|c| c.is_ascii_digit()) {
            return r"\d+".to_string();
        } else if example.contains('.') {
            return r".+\..+".to_string();
        } else {
            return format!("^{}$", regex::escape(example));
        }
    }

    /// Generate pattern from character classes
    fn generate_from_char_classes(&mut self, dataset: &Dataset) -> String {
        let char_freq = dataset.char_frequency();
        if char_freq.is_empty() {
            return ".*".to_string();
        }

        let mut pattern = String::new();
        let num_classes = self.rng.gen_range(1..=3);
        
        for _ in 0..num_classes {
            let class_type = self.rng.gen_range(0..4);
            match class_type {
                0 => pattern.push_str(r"\d+"),
                1 => pattern.push_str(r"\w+"),
                2 => pattern.push_str(r"[a-zA-Z]+"),
                _ => pattern.push_str(r".+"),
            }
        }

        pattern
    }

    /// Generate random pattern
    fn generate_random_pattern(&mut self) -> String {
        let patterns = vec![
            r".*",
            r"\d+",
            r"\w+",
            r"[a-zA-Z]+",
            r".+@.+",
            r"\d{3,}",
            r"[a-z]+\.[a-z]+",
        ];
        
        let idx = self.rng.gen_range(0..patterns.len());
        patterns[idx].to_string()
    }

    /// Evolve population
    pub fn evolve_population(
        &mut self,
        population: Vec<RegexIndividual>,
        fitness_scores: &[FitnessScore],
    ) -> Result<Vec<RegexIndividual>> {
        let mut new_population = Vec::new();
        let population_size = population.len();

        // Selection: keep top 50% of population
        let mut indexed_scores: Vec<(usize, &FitnessScore)> = fitness_scores.iter().enumerate().collect();
        indexed_scores.sort_by(|a, b| b.1.f1_score.partial_cmp(&a.1.f1_score).unwrap());

        let elite_size = population_size / 2;
        for i in 0..elite_size {
            let idx = indexed_scores[i].0;
            new_population.push(population[idx].clone());
        }

        // Generate new individuals to fill the rest
        while new_population.len() < population_size {
            let parent1_idx = self.rng.gen_range(0..elite_size);
            let parent2_idx = self.rng.gen_range(0..elite_size);
            
            let parent1 = &new_population[parent1_idx];
            let parent2 = &new_population[parent2_idx];
            
            let child = self.crossover(parent1, parent2)?;
            let mutated_child = self.mutate(child)?;
            
            new_population.push(mutated_child);
        }

        Ok(new_population)
    }

    /// Crossover two individuals
    fn crossover(&mut self, parent1: &RegexIndividual, parent2: &RegexIndividual) -> Result<RegexIndividual> {
        // Simple crossover: combine patterns with OR
        let pattern = if self.rng.gen_bool(0.5) {
            format!("({}|{})", parent1.pattern, parent2.pattern)
        } else {
            format!("{}{}", parent1.pattern, parent2.pattern)
        };

        Ok(RegexIndividual::new(pattern, parent1.depth.max(parent2.depth) + 1))
    }

    /// Mutate an individual
    fn mutate(&mut self, individual: RegexIndividual) -> Result<RegexIndividual> {
        if self.rng.gen_bool(0.1) { // 10% mutation rate
            let mutation_type = self.rng.gen_range(0..3);
            let pattern = match mutation_type {
                0 => self.add_quantifier(&individual.pattern),
                1 => self.add_character_class(&individual.pattern),
                _ => self.simplify_pattern(&individual.pattern),
            };
            Ok(RegexIndividual::new(pattern, individual.depth))
        } else {
            Ok(individual)
        }
    }

    /// Add quantifier to pattern
    fn add_quantifier(&mut self, pattern: &str) -> String {
        let quantifiers = vec!["+", "*", "?", "{2,}", "{3,5}"];
        let idx = self.rng.gen_range(0..quantifiers.len());
        format!("({}){}", pattern, quantifiers[idx])
    }

    /// Add character class to pattern
    fn add_character_class(&mut self, pattern: &str) -> String {
        let classes = vec![r"\d", r"\w", r"[a-zA-Z]", r"[0-9]"];
        let idx = self.rng.gen_range(0..classes.len());
        format!("{}+{}", classes[idx], pattern)
    }

    /// Simplify pattern
    fn simplify_pattern(&mut self, pattern: &str) -> String {
        // Remove redundant parentheses or simplify
        pattern.replace("(", "").replace(")", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_creation() {
        let params = EvolutionParams::default();
        let evolution = Evolution::new(params);
        assert!(evolution.params.max_iterations > 0);
    }

    #[test]
    fn test_individual_generation() {
        let params = EvolutionParams::default();
        let mut evolution = Evolution::new(params);
        
        let training_data = TrainingData {
            positive: vec!["test@example.com".to_string()],
            negative: vec!["not_email".to_string()],
        };
        
        let mut dataset = Dataset::new(training_data);
        dataset.build().unwrap();
        
        let bpe_tokens = HashMap::new();
        let individual = evolution.generate_individual(&dataset, &bpe_tokens).unwrap();
        
        assert!(!individual.pattern.is_empty());
    }

    #[test]
    fn test_crossover() {
        let params = EvolutionParams::default();
        let mut evolution = Evolution::new(params);
        
        let parent1 = RegexIndividual::new("abc".to_string(), 1);
        let parent2 = RegexIndividual::new("def".to_string(), 1);
        
        let child = evolution.crossover(&parent1, &parent2).unwrap();
        assert!(!child.pattern.is_empty());
        assert!(child.pattern.contains("abc") || child.pattern.contains("def"));
    }

    #[test]
    fn test_mutation() {
        let params = EvolutionParams::default();
        let mut evolution = Evolution::new(params);
        
        let individual = RegexIndividual::new("test".to_string(), 1);
        let mutated = evolution.mutate(individual).unwrap();
        
        assert!(!mutated.pattern.is_empty());
    }
} 