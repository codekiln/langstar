mod commands;
mod config;
mod error;
mod output;

use clap::{Parser, Subcommand};
use commands::PromptCommands;
use config::Config;
use error::Result;
use output::OutputFormat;

/// Langstar - Unified CLI for LangChain ecosystem
///
/// Access LangSmith, LangGraph Cloud, and other LangChain services
/// from a single, ergonomic command-line interface.
#[derive(Debug, Parser)]
#[command(name = "langstar")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output format (json or table)
    #[arg(short = 'f', long, global = true, env = "LANGSTAR_OUTPUT_FORMAT")]
    format: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage LangSmith prompts
    #[command(subcommand)]
    Prompt(PromptCommands),

    /// Show configuration file location
    Config,

    /// Show version information
    Version,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load()?;

    // Determine output format
    let format = if let Some(format_str) = cli.format {
        OutputFormat::from_str(&format_str)?
    } else {
        OutputFormat::from_str(&config.output_format)?
    };

    // Execute command
    match cli.command {
        Commands::Prompt(prompt_cmd) => {
            prompt_cmd.execute(&config, format).await?;
        }
        Commands::Config => {
            let config_path = Config::config_file_path()?;
            println!("Configuration file: {}", config_path.display());
            println!("\nCurrent configuration:");
            println!("  Output format: {}", config.output_format);
            println!(
                "  LangSmith API key: {}",
                if config.langsmith_api_key.is_some() {
                    "configured"
                } else {
                    "not configured"
                }
            );
            println!(
                "  LangGraph API key: {}",
                if config.langgraph_api_key.is_some() {
                    "configured"
                } else {
                    "not configured"
                }
            );
            println!("\nEnvironment variables:");
            println!("  LANGSMITH_API_KEY: {}", if std::env::var("LANGSMITH_API_KEY").is_ok() { "set" } else { "not set" });
            println!("  LANGGRAPH_API_KEY: {}", if std::env::var("LANGGRAPH_API_KEY").is_ok() { "set" } else { "not set" });
            println!("  LANGSTAR_OUTPUT_FORMAT: {}", std::env::var("LANGSTAR_OUTPUT_FORMAT").unwrap_or_else(|_| "not set".to_string()));
        }
        Commands::Version => {
            println!("langstar {}", env!("CARGO_PKG_VERSION"));
            println!("Rust SDK for LangChain ecosystem");
        }
    }

    Ok(())
}
