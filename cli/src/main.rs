mod commands;
mod config;
mod error;
mod output;
pub mod time;

use clap::{Parser, Subcommand};
use commands::{
    AssistantCommands, ConfigCommands, DatasetCommands, EvalCommands, GraphCommands,
    ModelConfigCommands, PromptCommands, QueueCommands, RunsCommands, SecretsCommands,
};
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

    /// Manage LangGraph assistants
    #[command(subcommand)]
    Assistant(AssistantCommands),

    /// Manage LangGraph deployments
    #[command(subcommand)]
    Graph(GraphCommands),

    /// Query and manage LangSmith runs/traces
    #[command(subcommand)]
    Runs(RunsCommands),

    /// Manage LangSmith annotation queues
    #[command(subcommand)]
    Queue(QueueCommands),

    /// Manage LangSmith datasets
    #[command(subcommand)]
    Dataset(DatasetCommands),

    /// Manage LangSmith evaluations
    #[command(subcommand)]
    Eval(EvalCommands),

    /// Manage LangSmith model configurations
    #[command(name = "model-config", subcommand)]
    ModelConfig(ModelConfigCommands),

    /// Manage LangSmith workspace secrets
    #[command(subcommand)]
    Secrets(SecretsCommands),

    /// Manage configuration settings
    #[command(subcommand)]
    Config(ConfigCommands),

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
        Commands::Assistant(assistant_cmd) => {
            assistant_cmd.execute(&config, format).await?;
        }
        Commands::Graph(graph_cmd) => {
            graph_cmd.execute(&config, format).await?;
        }
        Commands::Runs(runs_cmd) => {
            runs_cmd.execute(&config, format).await?;
        }
        Commands::Queue(queue_cmd) => {
            queue_cmd.execute(&config, format).await?;
        }
        Commands::Dataset(dataset_cmd) => {
            dataset_cmd.execute(&config, format).await?;
        }
        Commands::Eval(eval_cmd) => {
            eval_cmd.execute(&config, format).await?;
        }
        Commands::ModelConfig(model_config_cmd) => {
            model_config_cmd.execute(&config, format).await?;
        }
        Commands::Secrets(secrets_cmd) => {
            secrets_cmd.execute(&config, format).await?;
        }
        Commands::Config(config_cmd) => {
            config_cmd.execute().await?;
        }
        Commands::Version => {
            println!("langstar {}", env!("CARGO_PKG_VERSION"));
            println!("Rust SDK for LangChain ecosystem");
        }
    }

    Ok(())
}
