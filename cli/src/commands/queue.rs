//! CLI commands for managing LangSmith annotation queues.
//!
//! This module provides the `langstar queue` command group for creating,
//! listing, and managing annotation queues and their runs.

use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::{Args, Subcommand};
use langstar_sdk::{
    AnnotationQueue, CreateAnnotationQueueRequest, LangchainClient, ListAnnotationQueuesParams,
    QueueType, UpdateAnnotationQueueRequest,
};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tabled::Tabled;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Commands
// ═══════════════════════════════════════════════════════════════════════════

/// Commands for managing LangSmith annotation queues
#[derive(Debug, Subcommand)]
pub enum QueueCommands {
    /// List annotation queues
    List(ListArgs),
    /// Create a new annotation queue
    Create(CreateArgs),
    /// Get details of a specific queue
    Get(GetArgs),
    /// Update an annotation queue
    Update(UpdateArgs),
    /// Delete an annotation queue
    Delete(DeleteArgs),
    /// Add runs to an annotation queue
    AddRuns(AddRunsArgs),
    /// Remove a run from an annotation queue
    RemoveRun(RemoveRunArgs),
    /// List runs in an annotation queue
    Items(ItemsArgs),
}

/// Arguments for the `queue list` command
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by exact name match
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by name substring
    #[arg(long)]
    pub name_contains: Option<String>,

    /// Maximum number of queues to return
    #[arg(short, long, default_value = "100")]
    pub limit: u32,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `queue create` command
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of the queue (required)
    #[arg(long)]
    pub name: String,

    /// Description of the queue
    #[arg(long)]
    pub description: Option<String>,

    /// Rubric instructions for annotators
    #[arg(long)]
    pub rubric: Option<String>,

    /// Queue type: single or pairwise
    #[arg(long, default_value = "single")]
    pub queue_type: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `queue get` command
#[derive(Debug, Args)]
pub struct GetArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `queue update` command
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// New name for the queue
    #[arg(long)]
    pub name: Option<String>,

    /// New description for the queue
    #[arg(long)]
    pub description: Option<String>,

    /// New rubric instructions
    #[arg(long)]
    pub rubric: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the `queue delete` command
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `queue add-runs` command
#[derive(Debug, Args)]
pub struct AddRunsArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// Run IDs to add (UUIDs)
    #[arg(required_unless_present = "runs_file")]
    pub run_ids: Vec<Uuid>,

    /// File containing run IDs (one per line)
    #[arg(long)]
    pub runs_file: Option<PathBuf>,
}

/// Arguments for the `queue remove-run` command
#[derive(Debug, Args)]
pub struct RemoveRunArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// Run ID to remove (UUID)
    pub run_id: Uuid,
}

/// Arguments for the `queue items` command
#[derive(Debug, Args)]
pub struct ItemsArgs {
    /// Queue ID (UUID)
    pub queue_id: Uuid,

    /// Maximum number of items to return
    #[arg(short, long, default_value = "100")]
    pub limit: u32,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Table Display
// ═══════════════════════════════════════════════════════════════════════════

/// Simplified queue info for table display
#[derive(Debug, Tabled, Serialize)]
struct QueueRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    queue_type: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Created")]
    created: String,
}

impl From<&AnnotationQueue> for QueueRow {
    fn from(queue: &AnnotationQueue) -> Self {
        let description = queue
            .description
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(30)
            .collect::<String>();
        let description = if queue
            .description
            .as_ref()
            .map(|d| d.len() > 30)
            .unwrap_or(false)
        {
            format!("{}...", description)
        } else {
            description
        };

        Self {
            id: queue.id.to_string().chars().take(8).collect(),
            name: queue.name.clone(),
            queue_type: format!("{:?}", queue.queue_type).to_lowercase(),
            description,
            created: queue
                .created_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string()),
        }
    }
}

/// Queue item row for table display
#[derive(Debug, Tabled, Serialize)]
struct QueueItemRow {
    #[tabled(rename = "Index")]
    index: u32,
    #[tabled(rename = "Run ID")]
    run_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Added")]
    added: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Implementation
// ═══════════════════════════════════════════════════════════════════════════

impl QueueCommands {
    /// Execute the queue command
    pub async fn execute(&self, config: &Config, _format: OutputFormat) -> Result<()> {
        match self {
            QueueCommands::List(args) => Self::execute_list(args, config).await,
            QueueCommands::Create(args) => Self::execute_create(args, config).await,
            QueueCommands::Get(args) => Self::execute_get(args, config).await,
            QueueCommands::Update(args) => Self::execute_update(args, config).await,
            QueueCommands::Delete(args) => Self::execute_delete(args, config).await,
            QueueCommands::AddRuns(args) => Self::execute_add_runs(args, config).await,
            QueueCommands::RemoveRun(args) => Self::execute_remove_run(args, config).await,
            QueueCommands::Items(args) => Self::execute_items(args, config).await,
        }
    }

    async fn execute_list(args: &ListArgs, config: &Config) -> Result<()> {
        let formatter = if args.json {
            OutputFormatter::new(OutputFormat::Json)
        } else {
            OutputFormatter::new(OutputFormat::Table)
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let params = ListAnnotationQueuesParams {
            name: args.name.clone(),
            name_contains: args.name_contains.clone(),
            limit: Some(args.limit),
            ..Default::default()
        };

        let queues = client.list_annotation_queues(params).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&queues)?);
        } else {
            let rows: Vec<QueueRow> = queues.iter().map(QueueRow::from).collect();
            formatter.print_table(&rows)?;
            println!("\nFound {} queues", queues.len());
        }

        Ok(())
    }

    async fn execute_create(args: &CreateArgs, config: &Config) -> Result<()> {
        let queue_type = match args.queue_type.to_lowercase().as_str() {
            "single" => QueueType::Single,
            "pairwise" => QueueType::Pairwise,
            _ => {
                return Err(crate::error::CliError::Config(
                    "Invalid queue type. Use 'single' or 'pairwise'".to_string(),
                ));
            }
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let request = CreateAnnotationQueueRequest {
            name: args.name.clone(),
            description: args.description.clone(),
            rubric_instructions: args.rubric.clone(),
            queue_type: Some(queue_type),
            ..Default::default()
        };

        let queue = client.create_annotation_queue(request).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&queue)?);
        } else {
            println!("Created queue:");
            println!("  ID: {}", queue.base.id);
            println!("  Name: {}", queue.base.name);
            println!("  Type: {:?}", queue.base.queue_type);
            if let Some(created_at) = queue.base.created_at {
                println!("  Created: {}", created_at.format("%Y-%m-%dT%H:%M:%SZ"));
            }
        }

        Ok(())
    }

    async fn execute_get(args: &GetArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let queue = client.read_annotation_queue(args.queue_id).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&queue)?);
        } else {
            println!("Queue: {}", queue.base.name);
            println!("  ID: {}", queue.base.id);
            println!("  Type: {:?}", queue.base.queue_type);
            if let Some(desc) = &queue.base.description {
                println!("  Description: {}", desc);
            }
            if let Some(rubric) = &queue.rubric_instructions {
                println!("  Rubric: {}", rubric);
            }
            if let Some(created_at) = queue.base.created_at {
                println!("  Created: {}", created_at.format("%Y-%m-%dT%H:%M:%SZ"));
            }
            if let Some(updated_at) = queue.base.updated_at {
                println!("  Updated: {}", updated_at.format("%Y-%m-%dT%H:%M:%SZ"));
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

        if args.name.is_none() && args.description.is_none() && args.rubric.is_none() {
            formatter.warning("No updates specified. Use --name, --description, or --rubric");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let request = UpdateAnnotationQueueRequest {
            name: args.name.clone(),
            description: args.description.clone(),
            rubric_instructions: args.rubric.clone(),
            ..Default::default()
        };

        client
            .update_annotation_queue(args.queue_id, request)
            .await?;

        if args.json {
            println!(
                "{}",
                serde_json::json!({"success": true, "queue_id": args.queue_id.to_string()})
            );
        } else {
            println!("Queue {} updated successfully", args.queue_id);
        }

        Ok(())
    }

    async fn execute_delete(args: &DeleteArgs, config: &Config) -> Result<()> {
        if !args.force {
            eprintln!("Are you sure you want to delete queue {}?", args.queue_id);
            eprintln!("Use --force to skip this confirmation.");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        client.delete_annotation_queue(args.queue_id).await?;
        println!("Deleted queue {}", args.queue_id);

        Ok(())
    }

    async fn execute_add_runs(args: &AddRunsArgs, config: &Config) -> Result<()> {
        let mut run_ids = args.run_ids.clone();

        // Read additional run IDs from file if provided
        if let Some(file_path) = &args.runs_file {
            let file = std::fs::File::open(file_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    match Uuid::parse_str(trimmed) {
                        Ok(uuid) => run_ids.push(uuid),
                        Err(_) => eprintln!("Warning: Skipping invalid UUID: {}", trimmed),
                    }
                }
            }
        }

        if run_ids.is_empty() {
            eprintln!("No run IDs provided");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        client
            .add_runs_to_annotation_queue(args.queue_id, run_ids.clone())
            .await?;

        println!("Added {} runs to queue {}", run_ids.len(), args.queue_id);

        Ok(())
    }

    async fn execute_remove_run(args: &RemoveRunArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        client
            .delete_run_from_annotation_queue(args.queue_id, args.run_id)
            .await?;

        println!("Removed run {} from queue {}", args.run_id, args.queue_id);

        Ok(())
    }

    async fn execute_items(args: &ItemsArgs, config: &Config) -> Result<()> {
        let formatter = if args.json {
            OutputFormatter::new(OutputFormat::Json)
        } else {
            OutputFormatter::new(OutputFormat::Table)
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        // Fetch items by index (API only supports get by index)
        // Note: Sequential fetching is required as the API only provides get-by-index
        let mut items = Vec::new();
        for index in 0..args.limit {
            match client
                .get_run_from_annotation_queue(args.queue_id, index)
                .await
            {
                Ok(item) => items.push((index, item)),
                Err(langstar_sdk::LangstarError::ApiError { status: 404, .. }) => {
                    // 404 means no more items at this index - end of queue
                    break;
                }
                Err(e) => {
                    // Propagate other errors (auth failures, network issues, etc.)
                    return Err(e.into());
                }
            }
        }

        if args.json {
            let json_items: Vec<_> = items.iter().map(|(_, item)| item).collect();
            println!("{}", serde_json::to_string_pretty(&json_items)?);
        } else {
            let rows: Vec<QueueItemRow> = items
                .iter()
                .map(|(index, item)| QueueItemRow {
                    index: *index,
                    run_id: item.run.id.to_string().chars().take(8).collect(),
                    name: item.run.name.chars().take(30).collect(),
                    status: item.run.status.clone(),
                    added: item
                        .added_at
                        .map(|t| t.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "-".to_string()),
                })
                .collect();
            formatter.print_table(&rows)?;
            println!("\nFound {} items in queue", items.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_row_truncation() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Queue",
            "tenantId": "87654321-4321-4321-4321-210987654321",
            "queueType": "single",
            "description": "A very long description that should be truncated for display",
            "createdAt": "2024-01-01T12:00:00Z",
            "updatedAt": "2024-01-01T12:00:00Z"
        }"#;

        let queue: AnnotationQueue = serde_json::from_str(json).unwrap();
        let row = QueueRow::from(&queue);

        assert_eq!(row.id, "12345678");
        assert!(row.description.ends_with("..."));
    }
}
