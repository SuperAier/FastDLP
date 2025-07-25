# FastDLP - Fast Data Loss Prevention

A high-performance Rust implementation of data loss prevention (DLP) system

## Overview

FastDLP is a comprehensive data loss prevention tool that provides:

- **Sensitive Data Analysis**: Identify sensitive data in structured and semi-structured data
- **Regex Generation**: Automatically generate regex patterns from positive/negative samples using evolutionary algorithms
- **High Performance**: Leverages Rust's speed and memory safety
- **Extensible**: Support for custom sensitive data types and patterns

## Features

### Sensitive Data Analysis
- **Built-in Types**: Supports 17+ common sensitive data types including:
  - Personal identifiers (ID cards, passports, mobile phones)
  - Financial data (bank cards, social credit codes)
  - Contact information (emails, addresses, postal codes)
  - Network data (IPv4, IPv6, MAC addresses, domains)
  - And more...

- **Custom Patterns**: Support for user-defined regex patterns
- **Neural Network Classification**: Uses AI models for entity classification
- **Data Validation**: Advanced validation beyond regex matching (e.g., Luhn algorithm for credit cards)

### Regex Generation
- **Evolutionary Algorithm**: Uses genetic algorithms to evolve regex patterns
- **BPE Integration**: Byte Pair Encoding for pattern discovery
- **Multi-objective Optimization**: Balances precision, recall, and pattern complexity
- **Divide and Conquer**: Iteratively extracts sub-patterns for complex data

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo package manager

### From Source
```bash
git clone https://github.com/fastdlp/fastdlp.git
cd fastdlp
cargo build --release
```

## Quick Start

### Command Line Usage

#### Analyze sensitive data in CSV files
```bash
# Basic analysis
fastdlp analyze -i data.csv

# With custom patterns
fastdlp analyze -i data.csv -p patterns.json -o results.json

# With custom thresholds
fastdlp analyze -i data.csv -t thresholds.json
```

#### Generate regex patterns
```bash
# Generate from training data
fastdlp generate -n EMAIL -t email_training.csv -o email_regex.json

# With custom parameters
fastdlp generate -n PHONE -t phone_data.csv --max-iterations 1000 --population-size 500
```

### Library Usage

#### Sensitive Data Analysis
```rust
use fastdlp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    fastdlp::init_logger();
    
    // Create analyzer
    let analyzer = TableAnalyzer::new();
    
    // Analyze CSV file
    let results = analyzer.analyze_csv("data.csv").await?;
    
    // Print results
    for (column, result) in results {
        println!("{}: {} ({})", column, result.data_type, result.fraction);
    }
    
    Ok(())
}
```

#### Regex Generation
```rust
use fastdlp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create generator
    let generator = RegexGenerator::new();
    
    // Prepare training data
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
    
    // Generate regex
    let params = EvolutionParams::default();
    let result = generator.generate("EMAIL", &training_data, &params).await?;
    
    println!("Generated regex: {}", result.regex_pattern);
    
    Ok(())
}
```

## Configuration

### Custom Patterns
Create a JSON file with custom regex patterns:

```json
[
  {
    "name": "CUSTOM_ID",
    "pattern": "^[A-Z]{2}\\d{6}$",
    "description": "Custom ID format"
  }
]
```

### Thresholds
Configure detection thresholds:

```json
{
  "EMAIL": 0.9,
  "PHONE": 0.8,
  "CUSTOM_ID": 0.7
}
```

## Performance

FastDLP is designed for high performance:

- **Parallel Processing**: Uses Rayon for CPU-intensive operations
- **Memory Efficient**: Streaming CSV processing with configurable memory limits
- **Optimized Regex**: Compiled regex patterns with validation caching
- **Async Support**: Non-blocking I/O operations

## Architecture

```
FastDLP
├── sensitive_analyze/     # Data analysis engine
│   ├── analyzer_engine.rs # Core analysis logic
│   ├── table_analyzer.rs  # CSV table analysis
│   ├── entity_classify/   # Entity classification
│   └── entity_recognize/  # Entity recognition
├── regex_generation/      # Pattern generation
│   ├── generator.rs       # Main generator
│   ├── evolution.rs       # Evolutionary algorithm
│   ├── fitness.rs         # Fitness evaluation
│   ├── dataset.rs         # Training data management
│   └── bpe.rs            # Byte Pair Encoding
└── common/               # Shared utilities
    ├── types.rs          # Common data types
    ├── constants.rs      # Configuration constants
    └── utils.rs          # Utility functions
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Testing

Run the test suite:
```bash
cargo test
```

Run with coverage:
```bash
cargo test --all-features
```

## Benchmarks

Run performance benchmarks:
```bash
cargo bench
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by various DLP and regex generation research
- Built with the Rust ecosystem's excellent libraries

## Support

- 📖 [Documentation](https://docs.rs/fastdlp)
- 🐛 [Issue Tracker](https://github.com/fastdlp/fastdlp/issues)
- 💬 [Discussions](https://github.com/fastdlp/fastdlp/discussions)

---

**FastDLP** - Fast, Safe, Reliable Data Loss Prevention 