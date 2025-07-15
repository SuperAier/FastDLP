use clap::{Args, Parser, Subcommand};
use fastdlp::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fastdlp")]
#[command(about = "Fast Data Loss Prevention - A high-performance DLP tool")]
#[command(version = fastdlp::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze sensitive data in a CSV file
    Analyze(AnalyzeArgs),
    /// Generate regex patterns from training data
    Generate(GenerateArgs),
    /// Show version information
    Version,
}

#[derive(Args)]
struct AnalyzeArgs {
    /// Path to the CSV file to analyze
    #[arg(short, long)]
    input: PathBuf,
    
    /// Optional path to custom regex patterns JSON file
    #[arg(short, long)]
    patterns: Option<PathBuf>,
    
    /// Optional path to thresholds JSON file
    #[arg(short, long)]
    thresholds: Option<PathBuf>,
    
    /// Output path for results JSON file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct GenerateArgs {
    /// Name for the generated regex
    #[arg(short, long)]
    name: String,
    
    /// Path to training data CSV file
    #[arg(short, long)]
    training_data: PathBuf,
    
    /// Output path for generated regex
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Maximum iterations
    #[arg(long, default_value = "2000")]
    max_iterations: usize,
    
    /// Initial population size
    #[arg(long, default_value = "1000")]
    population_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    fastdlp::init_logger();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Analyze(args) => {
            info!("Starting sensitive data analysis...");
            
            let mut analyzer = TableAnalyzer::new();
            
            // Load custom patterns if provided
            if let Some(patterns_path) = &args.patterns {
                let patterns = load_custom_patterns(patterns_path)?;
                analyzer.set_custom_patterns(patterns);
            }
            
            // Load thresholds if provided
            if let Some(thresholds_path) = &args.thresholds {
                let content = std::fs::read_to_string(thresholds_path)?;
                let thresholds: ThresholdConfig = serde_json::from_str(&content)?;
                analyzer.set_thresholds(thresholds);
            }
            
            // Analyze the CSV file
            let results = analyzer.analyze_csv(&args.input).await?;
            
            // Output results
            if let Some(output_path) = &args.output {
                save_results_json(output_path, &results)?;
                info!("Results saved to: {}", output_path.display());
            } else {
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
            
            info!("Analysis completed successfully!");
        }
        
        Commands::Generate(args) => {
            info!("Starting regex generation...");
            
            let training_data = load_training_data(&args.training_data)?;
            let mut generator = RegexGenerator::new();
            
            let params = EvolutionParams {
                max_iterations: args.max_iterations,
                init_population_size: args.population_size,
                ..Default::default()
            };
            
            let result = generator.generate(&args.name, &training_data, &params).await?;
            
            // Output result
            if let Some(output_path) = &args.output {
                let json = serde_json::to_string_pretty(&result)?;
                std::fs::write(output_path, json)?;
                info!("Generated regex saved to: {}", output_path.display());
            } else {
                println!("Generated regex: {}", result.regex_pattern);
            }
            
            info!("Regex generation completed successfully!");
        }
        
        Commands::Version => {
            println!("FastDLP v{}", fastdlp::VERSION);
        }
    }
    
    Ok(())
} 