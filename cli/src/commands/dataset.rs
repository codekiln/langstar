//! CLI commands for managing LangSmith datasets.
//!
//! This module provides the `langstar dataset` command group for creating,
//! listing, and managing datasets and their examples.

use crate::config::Config;
use crate::error::Result;
use crate::output::{ExportFormat, OutputFormat, OutputFormatter};
use clap::{Args, Subcommand};
use langstar_sdk::{
    DataType, Dataset, DatasetCreate, DatasetUpdate, Example, ExampleCreate, LangchainClient,
    ListDatasetsParams, ListExamplesParams,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tabled::Tabled;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Commands
// ═══════════════════════════════════════════════════════════════════════════

/// Commands for managing LangSmith datasets
#[derive(Debug, Subcommand)]
pub enum DatasetCommands {
    /// Create a new dataset
    Create(CreateArgs),
    /// List datasets
    List(ListArgs),
    /// Get details of a specific dataset
    Get(GetArgs),
    /// Update a dataset
    Update(UpdateArgs),
    /// Delete a dataset
    Delete(DeleteArgs),
    /// Import examples from a file (JSONL or CSV)
    Import(ImportArgs),
    /// List examples in a dataset
    ListExamples(ListExamplesArgs),
    /// Export examples to a file (JSONL or CSV)
    Export(ExportArgs),
}

/// Arguments for the `dataset create` command
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of the dataset (required)
    #[arg(long)]
    pub name: String,

    /// Data type: kv, llm, or chat
    #[arg(long, value_name = "TYPE", default_value = "kv")]
    pub data_type: String,

    /// Description of the dataset
    #[arg(long)]
    pub description: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `dataset list` command
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by exact name match
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by name substring
    #[arg(long)]
    pub name_contains: Option<String>,

    /// Filter by data type
    #[arg(long, value_name = "TYPE")]
    pub data_type: Option<String>,

    /// Maximum number of datasets to return
    #[arg(short, long, default_value = "100")]
    pub limit: i64,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `dataset get` command
#[derive(Debug, Args)]
pub struct GetArgs {
    /// Dataset ID (UUID)
    pub dataset_id: Uuid,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `dataset update` command
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Dataset ID (UUID)
    pub dataset_id: Uuid,

    /// New name for the dataset
    #[arg(long)]
    pub name: Option<String>,

    /// New description for the dataset
    #[arg(long)]
    pub description: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `dataset delete` command
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Dataset ID (UUID)
    pub dataset_id: Uuid,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

/// Arguments for the `dataset import` command
#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Dataset ID (UUID) to import into
    pub dataset_id: Uuid,

    /// Path to the file to import (JSONL or CSV)
    #[arg(long)]
    pub file: PathBuf,

    /// File format: jsonl or csv (auto-detected from extension if not specified)
    #[arg(long)]
    pub format: Option<String>,
}

/// Arguments for the `dataset list-examples` command
#[derive(Debug, Args)]
pub struct ListExamplesArgs {
    /// Dataset ID (UUID)
    pub dataset_id: Uuid,

    /// Maximum number of examples to return
    #[arg(short, long, default_value = "100")]
    pub limit: i64,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `dataset export` command
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Dataset ID (UUID) to export
    pub dataset_id: Uuid,

    /// Output file format: jsonl or csv
    #[arg(long = "file-format", value_enum, default_value = "csv")]
    pub file_format: ExportFormat,

    /// Output file path (prints to stdout if not specified)
    #[arg(long, short)]
    pub out: Option<PathBuf>,

    /// Maximum number of examples to export (default: all)
    #[arg(short, long)]
    pub limit: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Table Display
// ═══════════════════════════════════════════════════════════════════════════

/// Simplified dataset info for table display
#[derive(Debug, Tabled, Serialize)]
struct DatasetRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    data_type: String,
    #[tabled(rename = "Examples")]
    example_count: i64,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Modified")]
    modified: String,
}

impl From<&Dataset> for DatasetRow {
    fn from(dataset: &Dataset) -> Self {
        let description = dataset
            .description
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(30)
            .collect::<String>();
        let description = if dataset.description.as_ref().is_some_and(|d| d.len() > 30) {
            format!("{}...", description)
        } else {
            description
        };

        Self {
            id: dataset.id.to_string().chars().take(8).collect(),
            name: dataset.name.clone(),
            data_type: dataset
                .data_type
                .map(|dt| format!("{:?}", dt).to_lowercase())
                .unwrap_or_else(|| "kv".to_string()),
            example_count: dataset.example_count.unwrap_or(0),
            description,
            modified: dataset
                .modified_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string()),
        }
    }
}

/// Simplified example info for table display
#[derive(Debug, Tabled, Serialize)]
struct ExampleRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Inputs")]
    inputs: String,
    #[tabled(rename = "Outputs")]
    outputs: String,
    #[tabled(rename = "Created")]
    created: String,
}

impl From<&Example> for ExampleRow {
    fn from(example: &Example) -> Self {
        // Serialize inputs once and truncate with "..." if needed
        let inputs_json = serde_json::to_string(&example.inputs).unwrap_or_default();
        let inputs_truncated: String = inputs_json.chars().take(40).collect();
        let inputs = if inputs_json.len() > 40 {
            format!("{}...", inputs_truncated)
        } else {
            inputs_truncated
        };

        // Serialize outputs once and truncate with "..." if needed
        let outputs_json = example
            .outputs
            .as_ref()
            .map(|o| serde_json::to_string(o).unwrap_or_default())
            .unwrap_or_else(|| "-".to_string());
        let outputs_truncated: String = outputs_json.chars().take(40).collect();
        let outputs = if outputs_json.len() > 40 {
            format!("{}...", outputs_truncated)
        } else {
            outputs_truncated
        };

        // Truncate name with "..." if needed
        let name = if example.name.len() > 20 {
            format!("{}...", example.name.chars().take(20).collect::<String>())
        } else {
            example.name.clone()
        };

        Self {
            id: example.id.to_string().chars().take(8).collect(),
            name,
            inputs,
            outputs,
            created: example
                .created_at
                .map(|t| t.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// JSONL Record (for import/export)
// ═══════════════════════════════════════════════════════════════════════════

/// JSONL record format for import/export
#[derive(Debug, Serialize, Deserialize)]
struct JsonlRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    inputs: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Parse a data type string into a DataType enum.
fn parse_data_type(s: &str) -> Result<DataType> {
    match s.to_lowercase().as_str() {
        "kv" => Ok(DataType::Kv),
        "llm" => Ok(DataType::Llm),
        "chat" => Ok(DataType::Chat),
        _ => Err(crate::error::CliError::Config(
            "Invalid data type. Use 'kv', 'llm', or 'chat'".to_string(),
        )),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Implementation
// ═══════════════════════════════════════════════════════════════════════════

impl DatasetCommands {
    /// Execute the dataset command
    pub async fn execute(&self, config: &Config, _format: OutputFormat) -> Result<()> {
        match self {
            DatasetCommands::Create(args) => Self::execute_create(args, config).await,
            DatasetCommands::List(args) => Self::execute_list(args, config).await,
            DatasetCommands::Get(args) => Self::execute_get(args, config).await,
            DatasetCommands::Update(args) => Self::execute_update(args, config).await,
            DatasetCommands::Delete(args) => Self::execute_delete(args, config).await,
            DatasetCommands::Import(args) => Self::execute_import(args, config).await,
            DatasetCommands::ListExamples(args) => Self::execute_list_examples(args, config).await,
            DatasetCommands::Export(args) => Self::execute_export(args, config).await,
        }
    }

    async fn execute_create(args: &CreateArgs, config: &Config) -> Result<()> {
        let data_type = parse_data_type(&args.data_type)?;

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let request = DatasetCreate {
            name: args.name.clone(),
            description: args.description.clone(),
            data_type: Some(data_type),
            ..Default::default()
        };

        let dataset = client.create_dataset(request).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&dataset)?);
        } else {
            println!("Created dataset:");
            println!("  ID: {}", dataset.id);
            println!("  Name: {}", dataset.name);
            println!(
                "  Type: {}",
                dataset
                    .data_type
                    .map(|dt| format!("{:?}", dt).to_lowercase())
                    .unwrap_or_else(|| "kv".to_string())
            );
            if let Some(modified) = dataset.modified_at {
                println!("  Modified: {}", modified.format("%Y-%m-%dT%H:%M:%SZ"));
            }
        }

        Ok(())
    }

    async fn execute_list(args: &ListArgs, config: &Config) -> Result<()> {
        let formatter = if args.json {
            OutputFormatter::new(OutputFormat::Json)
        } else {
            OutputFormatter::new(OutputFormat::Table)
        };

        let data_type = args
            .data_type
            .as_ref()
            .map(|dt| parse_data_type(dt))
            .transpose()?;

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let params = ListDatasetsParams {
            name: args.name.clone(),
            name_contains: args.name_contains.clone(),
            data_type,
            limit: Some(args.limit),
            ..Default::default()
        };

        let datasets = client.list_datasets(params).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&datasets)?);
        } else {
            let rows: Vec<DatasetRow> = datasets.iter().map(DatasetRow::from).collect();
            formatter.print_table(&rows)?;
            println!("\nFound {} datasets", datasets.len());
        }

        Ok(())
    }

    async fn execute_get(args: &GetArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let dataset = client.get_dataset(args.dataset_id).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&dataset)?);
        } else {
            println!("Dataset: {}", dataset.name);
            println!("  ID: {}", dataset.id);
            println!(
                "  Type: {}",
                dataset
                    .data_type
                    .map(|dt| format!("{:?}", dt).to_lowercase())
                    .unwrap_or_else(|| "kv".to_string())
            );
            if let Some(desc) = &dataset.description {
                println!("  Description: {}", desc);
            }
            println!("  Examples: {}", dataset.example_count.unwrap_or(0));
            println!("  Sessions: {}", dataset.session_count.unwrap_or(0));
            if let Some(created) = dataset.created_at {
                println!("  Created: {}", created.format("%Y-%m-%dT%H:%M:%SZ"));
            }
            if let Some(modified) = dataset.modified_at {
                println!("  Modified: {}", modified.format("%Y-%m-%dT%H:%M:%SZ"));
            }
        }

        Ok(())
    }

    async fn execute_update(args: &UpdateArgs, config: &Config) -> Result<()> {
        let formatter = if args.json {
            OutputFormatter::new(OutputFormat::Json)
        } else {
            OutputFormatter::new(OutputFormat::Table)
        };

        if args.name.is_none() && args.description.is_none() {
            formatter.warning("No updates specified. Use --name or --description");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let request = DatasetUpdate {
            name: args.name.clone(),
            description: args.description.clone(),
            ..Default::default()
        };

        let dataset = client.update_dataset(args.dataset_id, request).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&dataset)?);
        } else {
            println!("Dataset {} updated successfully", args.dataset_id);
            println!("  Name: {}", dataset.name);
            if let Some(desc) = &dataset.description {
                println!("  Description: {}", desc);
            }
        }

        Ok(())
    }

    async fn execute_delete(args: &DeleteArgs, config: &Config) -> Result<()> {
        if !args.yes {
            eprintln!(
                "Are you sure you want to delete dataset {}?",
                args.dataset_id
            );
            eprintln!("Use --yes (-y) to skip this confirmation.");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        client.delete_dataset(args.dataset_id).await?;
        println!("Deleted dataset {}", args.dataset_id);

        Ok(())
    }

    async fn execute_import(args: &ImportArgs, config: &Config) -> Result<()> {
        // Determine format from extension or argument
        let format = if let Some(fmt) = &args.format {
            fmt.to_lowercase()
        } else {
            args.file
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jsonl")
                .to_lowercase()
        };

        if format != "jsonl" && format != "csv" {
            return Err(crate::error::CliError::Config(
                "Unsupported format. Use 'jsonl' or 'csv'".to_string(),
            ));
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let file = std::fs::File::open(&args.file)?;
        let reader = BufReader::new(file);

        let mut examples = Vec::new();

        if format == "jsonl" {
            for (line_num, line) in reader.lines().enumerate() {
                let line = line?;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                match serde_json::from_str::<JsonlRecord>(trimmed) {
                    Ok(record) => {
                        examples.push(ExampleCreate {
                            dataset_id: args.dataset_id,
                            inputs: Some(record.inputs),
                            outputs: record.outputs,
                            metadata: record.metadata,
                            id: record.id,
                            ..Default::default()
                        });
                    }
                    Err(e) => {
                        eprintln!("Warning: Skipping line {}: {}", line_num + 1, e);
                    }
                }
            }
        } else {
            // CSV format
            let mut csv_reader = csv::Reader::from_reader(reader);
            for (row_num, result) in csv_reader.deserialize().enumerate() {
                match result {
                    Ok(record) => {
                        let record: std::collections::HashMap<String, String> = record;

                        // Parse optional id column as UUID
                        let id = record
                            .get("id")
                            .and_then(|s| Uuid::parse_str(s.trim()).ok());

                        // Convert CSV row to JSON objects
                        let inputs: serde_json::Value = if let Some(inputs_str) =
                            record.get("inputs")
                        {
                            serde_json::from_str(inputs_str)
                                .unwrap_or(serde_json::json!({ "input": inputs_str }))
                        } else {
                            // Use all non-reserved columns as inputs
                            let inputs: std::collections::HashMap<_, _> = record
                                .iter()
                                .filter(|(k, _)| *k != "id" && *k != "outputs" && *k != "metadata")
                                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                                .collect();
                            serde_json::to_value(inputs).unwrap_or_default()
                        };

                        // Parse outputs column as JSON
                        let outputs = record.get("outputs").map(|s| {
                            serde_json::from_str(s).unwrap_or(serde_json::json!({ "output": s }))
                        });

                        // Parse metadata column as JSON
                        let metadata = record.get("metadata").and_then(|s| {
                            if s.trim().is_empty() {
                                None
                            } else {
                                serde_json::from_str(s).ok()
                            }
                        });

                        examples.push(ExampleCreate {
                            dataset_id: args.dataset_id,
                            inputs: Some(inputs),
                            outputs,
                            metadata,
                            id,
                            ..Default::default()
                        });
                    }
                    Err(e) => {
                        eprintln!("Warning: Skipping row {}: {}", row_num + 1, e);
                    }
                }
            }
        }

        if examples.is_empty() {
            eprintln!("No valid examples found in file");
            return Ok(());
        }

        // Use bulk create for efficiency
        let created = client.bulk_create_examples(examples).await?;
        println!(
            "Imported {} examples to dataset {}",
            created.len(),
            args.dataset_id
        );

        Ok(())
    }

    async fn execute_list_examples(args: &ListExamplesArgs, config: &Config) -> Result<()> {
        let formatter = if args.json {
            OutputFormatter::new(OutputFormat::Json)
        } else {
            OutputFormatter::new(OutputFormat::Table)
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let params = ListExamplesParams {
            dataset: Some(args.dataset_id),
            limit: Some(args.limit),
            ..Default::default()
        };

        let examples = client.list_examples(params).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&examples)?);
        } else {
            let rows: Vec<ExampleRow> = examples.iter().map(ExampleRow::from).collect();
            formatter.print_table(&rows)?;
            println!("\nFound {} examples", examples.len());
        }

        Ok(())
    }

    async fn execute_export(args: &ExportArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        // Fetch all examples (with pagination if needed)
        let limit = args.limit.unwrap_or(100);
        let params = ListExamplesParams {
            dataset: Some(args.dataset_id),
            limit: Some(limit),
            ..Default::default()
        };

        let examples = client.list_examples(params).await?;

        // Prepare output
        let mut output: Box<dyn Write> = if let Some(path) = &args.out {
            Box::new(std::fs::File::create(path)?)
        } else {
            Box::new(std::io::stdout())
        };

        match args.file_format {
            ExportFormat::Jsonl => {
                for example in &examples {
                    let record = JsonlRecord {
                        id: Some(example.id),
                        inputs: example.inputs.clone(),
                        outputs: example.outputs.clone(),
                        metadata: example.metadata.clone(),
                    };
                    writeln!(output, "{}", serde_json::to_string(&record)?)?;
                }
            }
            ExportFormat::Csv => {
                // CSV format
                let mut wtr = csv::Writer::from_writer(output);
                wtr.write_record(["id", "inputs", "outputs", "metadata"])?;
                for example in &examples {
                    wtr.write_record([
                        example.id.to_string(),
                        serde_json::to_string(&example.inputs)?,
                        example
                            .outputs
                            .as_ref()
                            .map(serde_json::to_string)
                            .transpose()?
                            .unwrap_or_default(),
                        example
                            .metadata
                            .as_ref()
                            .map(serde_json::to_string)
                            .transpose()?
                            .unwrap_or_default(),
                    ])?;
                }
                wtr.flush()?;
            }
        }

        if let Some(path) = &args.out {
            eprintln!("Exported {} examples to {:?}", examples.len(), path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_row_truncation() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Dataset",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "example_count": 100,
            "session_count": 5,
            "modified_at": "2024-01-01T12:00:00Z",
            "description": "A very long description that should be truncated for display purposes"
        }"#;

        let dataset: Dataset = serde_json::from_str(json).unwrap();
        let row = DatasetRow::from(&dataset);

        assert_eq!(row.id, "12345678");
        assert!(row.description.ends_with("..."));
    }

    #[test]
    fn test_jsonl_record_serialization() {
        let record = JsonlRecord {
            id: Some(Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap()),
            inputs: serde_json::json!({"question": "What is 2+2?"}),
            outputs: Some(serde_json::json!({"answer": "4"})),
            metadata: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"question\":\"What is 2+2?\""));
        assert!(json.contains("\"answer\":\"4\""));
    }
}
