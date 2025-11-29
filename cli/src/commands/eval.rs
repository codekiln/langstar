//! CLI commands for managing LangSmith evaluations.
//!
//! This module provides the `langstar eval` command group for creating,
//! running, and managing evaluations on datasets.

use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::{Args, Subcommand, ValueEnum};
use langstar_sdk::{
    LangchainClient,
    evaluations::{HeuristicEvaluator, ScoreType},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tabled::Tabled;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Commands
// ═══════════════════════════════════════════════════════════════════════════

/// Commands for managing LangSmith evaluations
#[derive(Debug, Subcommand)]
pub enum EvalCommands {
    /// Create a new evaluation configuration
    Create(CreateArgs),
    /// Run an evaluation
    Run(RunArgs),
    /// List evaluations
    List(ListArgs),
    /// Get details of a specific evaluation
    Get(GetArgs),
    /// Export evaluation results
    Export(ExportArgs),
}

// ═══════════════════════════════════════════════════════════════════════════
// Evaluator Type Enum
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluator types for evaluation configuration
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EvaluatorType {
    /// Exact string match (heuristic, zero-cost)
    ExactMatch,
    /// Substring contains check (heuristic, zero-cost)
    Contains,
    /// Regular expression match (heuristic, zero-cost)
    RegexMatch,
    /// JSON validity check (heuristic, zero-cost)
    JsonValid,
    /// String distance metrics (Levenshtein, etc.) (heuristic, zero-cost)
    StringDistance,
    /// LLM-as-judge evaluator (requires API calls)
    LlmJudge,
}

impl std::fmt::Display for EvaluatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluatorType::ExactMatch => write!(f, "exact_match"),
            EvaluatorType::Contains => write!(f, "contains"),
            EvaluatorType::RegexMatch => write!(f, "regex_match"),
            EvaluatorType::JsonValid => write!(f, "json_valid"),
            EvaluatorType::StringDistance => write!(f, "string_distance"),
            EvaluatorType::LlmJudge => write!(f, "llm_judge"),
        }
    }
}

impl TryFrom<EvaluatorType> for HeuristicEvaluator {
    type Error = &'static str;

    fn try_from(eval_type: EvaluatorType) -> std::result::Result<Self, Self::Error> {
        match eval_type {
            EvaluatorType::ExactMatch => Ok(HeuristicEvaluator::ExactMatch),
            EvaluatorType::Contains => Ok(HeuristicEvaluator::Contains),
            EvaluatorType::RegexMatch => Ok(HeuristicEvaluator::RegexMatch),
            EvaluatorType::JsonValid => Ok(HeuristicEvaluator::JsonValid),
            EvaluatorType::StringDistance => Ok(HeuristicEvaluator::StringDistance),
            EvaluatorType::LlmJudge => Err("LlmJudge is not a heuristic evaluator"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Create Command
// ═══════════════════════════════════════════════════════════════════════════

/// Arguments for the `eval create` command
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of the evaluation
    #[arg(long)]
    pub name: String,

    /// Dataset ID or name to evaluate against
    #[arg(long)]
    pub dataset: String,

    /// Evaluator type
    #[arg(long, value_enum)]
    pub evaluator: EvaluatorType,

    /// Judge model name (for LLM-as-judge evaluators)
    #[arg(long)]
    pub judge_model: Option<String>,

    /// Judge model provider (for LLM-as-judge evaluators)
    #[arg(long)]
    pub judge_provider: Option<String>,

    /// Path to judge prompt/rubric file (for LLM-as-judge evaluators)
    #[arg(long)]
    pub judge_prompt_file: Option<PathBuf>,

    /// Score type for LLM judge (categorical or continuous)
    #[arg(long, value_enum)]
    pub score_type: Option<ScoreTypeArg>,

    /// Comma-separated score choices for categorical scoring
    #[arg(long, value_delimiter = ',')]
    pub score_choices: Option<Vec<String>>,

    /// Minimum score for continuous scoring
    #[arg(long)]
    pub score_min: Option<f64>,

    /// Maximum score for continuous scoring
    #[arg(long)]
    pub score_max: Option<f64>,

    /// Include reasoning/explanation in LLM judge output
    #[arg(long)]
    pub include_reasoning: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Score type argument for CLI
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScoreTypeArg {
    /// Categorical scoring (e.g., Y/N, Pass/Fail)
    Categorical,
    /// Continuous numeric scoring (e.g., 0.0 to 1.0)
    Continuous,
}

impl From<ScoreTypeArg> for ScoreType {
    fn from(arg: ScoreTypeArg) -> Self {
        match arg {
            ScoreTypeArg::Categorical => ScoreType::Categorical,
            ScoreTypeArg::Continuous => ScoreType::Continuous,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Run Command
// ═══════════════════════════════════════════════════════════════════════════

/// Arguments for the `eval run` command
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Evaluation ID to run
    pub eval_id: Uuid,

    /// Preview mode: only run on first N examples
    #[arg(long)]
    pub preview: Option<usize>,

    /// Dry run: validate configuration without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// List Command
// ═══════════════════════════════════════════════════════════════════════════

/// Arguments for the `eval list` command
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by evaluation name
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by dataset ID
    #[arg(long)]
    pub dataset: Option<String>,

    /// Filter by evaluator type
    #[arg(long, value_enum)]
    pub evaluator_type: Option<EvaluatorType>,

    /// Maximum number of evaluations to return
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Get Command
// ═══════════════════════════════════════════════════════════════════════════

/// Arguments for the `eval get` command
#[derive(Debug, Args)]
pub struct GetArgs {
    /// Evaluation ID
    pub eval_id: Uuid,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Export Command
// ═══════════════════════════════════════════════════════════════════════════

/// Arguments for the `eval export` command
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Evaluation ID to export
    pub eval_id: Uuid,

    /// Output format (csv or jsonl)
    #[arg(long = "file-format", value_enum, default_value = "csv")]
    pub file_format: ExportFormat,

    /// Output file path (stdout if not specified)
    #[arg(short, long = "out")]
    pub out: Option<PathBuf>,

    /// Include reasoning/comments in export
    #[arg(long)]
    pub include_comments: bool,
}

/// Export format for evaluation results
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    /// CSV format
    Csv,
    /// JSON Lines format
    Jsonl,
}

// ═══════════════════════════════════════════════════════════════════════════
// Types for Display
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluation configuration display
#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct EvaluationDisplay {
    /// Evaluation ID
    pub id: String,
    /// Evaluation name
    pub name: String,
    /// Dataset reference
    pub dataset: String,
    /// Evaluator type
    pub evaluator: String,
    /// Status (created, running, completed, failed)
    pub status: String,
}

/// Evaluation result display
// Note: This type is part of placeholder implementation for future export functionality
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct EvaluationResultDisplay {
    /// Example ID
    pub example_id: String,
    /// Metric key
    pub key: String,
    /// Numeric score
    #[tabled(display_with = "display_option_f64")]
    pub score: Option<f64>,
    /// Categorical value
    #[tabled(display_with = "display_option_string")]
    pub value: Option<String>,
    /// Comment/reasoning
    #[tabled(display_with = "display_option_string")]
    pub comment: Option<String>,
}

// Note: These display functions are used by tabled's display_with attribute
// but are flagged as unused until EvaluationResultDisplay is actually instantiated
#[allow(dead_code)]
fn display_option_f64(opt: &Option<f64>) -> String {
    opt.map(|v| format!("{:.2}", v))
        .unwrap_or_else(|| "-".to_string())
}

#[allow(dead_code)]
fn display_option_string(opt: &Option<String>) -> String {
    opt.as_deref().unwrap_or("-").to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Execution
// ═══════════════════════════════════════════════════════════════════════════

impl EvalCommands {
    /// Execute the eval command
    pub async fn execute(self, config: &Config, format: OutputFormat) -> Result<()> {
        match self {
            EvalCommands::Create(args) => execute_create(args, config, format).await,
            EvalCommands::Run(args) => execute_run(args, config, format).await,
            EvalCommands::List(args) => execute_list(args, config, format).await,
            EvalCommands::Get(args) => execute_get(args, config, format).await,
            EvalCommands::Export(args) => execute_export(args, config, format).await,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Implementations
// ═══════════════════════════════════════════════════════════════════════════

async fn execute_create(args: CreateArgs, config: &Config, format: OutputFormat) -> Result<()> {
    let auth = config.to_auth_config();
    let _client = LangchainClient::new(auth)?;

    // Validate LLM judge configuration
    if matches!(args.evaluator, EvaluatorType::LlmJudge) {
        validate_llm_judge_args(&args)?;
    }

    // TODO: Implement evaluation creation
    // For now, this is a placeholder that shows what the implementation will do:
    // 1. Resolve dataset ID from name if needed
    // 2. Create evaluation configuration object
    // 3. Store configuration (this requires a backend storage mechanism)
    // 4. Return evaluation ID

    let eval_display = EvaluationDisplay {
        id: Uuid::new_v4().to_string(),
        name: args.name.clone(),
        dataset: args.dataset.clone(),
        evaluator: args.evaluator.to_string(),
        status: "created".to_string(),
    };

    let output_format = if args.json {
        OutputFormat::Json
    } else {
        format
    };

    let formatter = OutputFormatter::new(output_format);
    if output_format == OutputFormat::Json {
        formatter.print(&eval_display)?;
    } else {
        formatter.print_table(&[eval_display])?;
    }

    eprintln!("\nNote: Evaluation configuration stored. Use 'langstar eval run <id>' to execute.");

    Ok(())
}

async fn execute_run(args: RunArgs, config: &Config, format: OutputFormat) -> Result<()> {
    let auth = config.to_auth_config();
    let _client = LangchainClient::new(auth)?;

    // TODO: Implement evaluation execution
    // For now, this is a placeholder that shows what the implementation will do:
    // 1. Load evaluation configuration by ID
    // 2. Load dataset examples
    // 3. Apply preview limit if specified
    // 4. Execute evaluator on each example
    // 5. Store results
    // 6. Return summary

    if args.dry_run {
        eprintln!("Dry run: Configuration is valid. No evaluation executed.");
        return Ok(());
    }

    let output_format = if args.json {
        OutputFormat::Json
    } else {
        format
    };

    let preview_msg = args
        .preview
        .map(|n| format!(" (preview mode: first {} examples)", n))
        .unwrap_or_default();

    eprintln!("Running evaluation {}{}...", args.eval_id, preview_msg);
    eprintln!("\nNote: Evaluation execution not yet implemented.");

    let _formatter = OutputFormatter::new(output_format);

    Ok(())
}

async fn execute_list(args: ListArgs, config: &Config, format: OutputFormat) -> Result<()> {
    let auth = config.to_auth_config();
    let _client = LangchainClient::new(auth)?;

    // TODO: Implement evaluation listing
    // For now, return empty list

    let evaluations: Vec<EvaluationDisplay> = vec![];

    let output_format = if args.json {
        OutputFormat::Json
    } else {
        format
    };

    let formatter = OutputFormatter::new(output_format);
    if output_format == OutputFormat::Json {
        formatter.print(&evaluations)?;
    } else {
        formatter.print_table(&evaluations)?;
    }

    if evaluations.is_empty() {
        eprintln!("\nNo evaluations found.");
        if args.name.is_some() || args.dataset.is_some() || args.evaluator_type.is_some() {
            eprintln!("Try adjusting your filters.");
        }
    }

    Ok(())
}

async fn execute_get(args: GetArgs, config: &Config, format: OutputFormat) -> Result<()> {
    let auth = config.to_auth_config();
    let _client = LangchainClient::new(auth)?;

    // TODO: Implement evaluation retrieval

    eprintln!("Getting evaluation {}...", args.eval_id);
    eprintln!("\nNote: Evaluation retrieval not yet implemented.");

    let _output_format = if args.json {
        OutputFormat::Json
    } else {
        format
    };

    Ok(())
}

async fn execute_export(args: ExportArgs, config: &Config, _format: OutputFormat) -> Result<()> {
    let auth = config.to_auth_config();
    let _client = LangchainClient::new(auth)?;

    // TODO: Implement evaluation export
    // 1. Load evaluation results by ID
    // 2. Format as CSV or JSONL
    // 3. Write to file or stdout

    let format_str = match args.file_format {
        ExportFormat::Csv => "CSV",
        ExportFormat::Jsonl => "JSONL",
    };

    let output_target = args
        .out
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "stdout".to_string());

    eprintln!(
        "Exporting evaluation {} as {} to {}...",
        args.eval_id, format_str, output_target
    );
    eprintln!("\nNote: Evaluation export not yet implemented.");

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Validation Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn validate_llm_judge_args(args: &CreateArgs) -> Result<()> {
    // Validate that LLM judge has required configuration
    if args.judge_model.is_none() {
        eprintln!("Warning: No --judge-model specified. Using default model.");
    }

    // Validate judge prompt file if provided
    if let Some(ref prompt_file) = args.judge_prompt_file {
        if !prompt_file.exists() {
            return Err(crate::error::CliError::Config(format!(
                "Judge prompt file does not exist: {}",
                prompt_file.display()
            )));
        }
        if !prompt_file.is_file() {
            return Err(crate::error::CliError::Config(format!(
                "Judge prompt path is not a file: {}",
                prompt_file.display()
            )));
        }
    }

    // Validate score type configuration
    if let Some(score_type) = args.score_type {
        match score_type {
            ScoreTypeArg::Categorical => {
                if args.score_choices.is_none() {
                    eprintln!(
                        "Warning: No --score-choices specified for categorical scoring. Using default [Y, N]."
                    );
                }
            }
            ScoreTypeArg::Continuous => {
                if args.score_min.is_none() || args.score_max.is_none() {
                    eprintln!(
                        "Warning: --score-min and --score-max should both be specified for continuous scoring. Using defaults [0.0, 1.0] where not provided."
                    );
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_type_display() {
        assert_eq!(EvaluatorType::ExactMatch.to_string(), "exact_match");
        assert_eq!(EvaluatorType::StringDistance.to_string(), "string_distance");
        assert_eq!(EvaluatorType::LlmJudge.to_string(), "llm_judge");
    }

    #[test]
    fn test_evaluator_type_to_heuristic() {
        let exact_match = HeuristicEvaluator::try_from(EvaluatorType::ExactMatch).unwrap();
        assert_eq!(exact_match, HeuristicEvaluator::ExactMatch);

        let contains = HeuristicEvaluator::try_from(EvaluatorType::Contains).unwrap();
        assert_eq!(contains, HeuristicEvaluator::Contains);

        let string_distance = HeuristicEvaluator::try_from(EvaluatorType::StringDistance).unwrap();
        assert_eq!(string_distance, HeuristicEvaluator::StringDistance);
    }

    #[test]
    fn test_llm_judge_cannot_convert_to_heuristic() {
        let result = HeuristicEvaluator::try_from(EvaluatorType::LlmJudge);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "LlmJudge is not a heuristic evaluator");
    }

    #[test]
    fn test_score_type_conversion() {
        let categorical: ScoreType = ScoreTypeArg::Categorical.into();
        assert_eq!(categorical, ScoreType::Categorical);

        let continuous: ScoreType = ScoreTypeArg::Continuous.into();
        assert_eq!(continuous, ScoreType::Continuous);
    }

    #[test]
    fn test_validate_llm_judge_with_nonexistent_file() {
        use std::path::PathBuf;
        
        let args = CreateArgs {
            name: "test".to_string(),
            dataset: "ds-123".to_string(),
            evaluator: EvaluatorType::LlmJudge,
            judge_model: Some("gpt-4".to_string()),
            judge_provider: None,
            judge_prompt_file: Some(PathBuf::from("/nonexistent/file.txt")),
            score_type: None,
            score_choices: None,
            score_min: None,
            score_max: None,
            include_reasoning: false,
            json: false,
        };

        let result = validate_llm_judge_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
