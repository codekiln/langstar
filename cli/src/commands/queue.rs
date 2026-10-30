//! CLI commands for managing LangSmith annotation queues.
//!
//! This module provides the `langstar queue` command group for creating,
//! listing, and managing annotation queues and their runs.

use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::{Args, Subcommand};
use langstar_sdk::{
    AnnotationQueue, AnnotationQueueItemStatus, AnnotationQueueListItem,
    CreateAnnotationQueueRequest, LangchainClient, ListAnnotationQueueItemsParams,
    ListAnnotationQueuesParams, QueueType, UpdateAnnotationQueueRequest,
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
    /// List the items in an annotation queue, one review section at a time
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

    /// Review section to list: needs-my-review, needs-others-review or archived (default: all)
    #[arg(long, value_parser = ["needs-my-review", "needs-others-review", "archived"])]
    pub status: Option<String>,

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
    #[tabled(rename = "Item ID")]
    id: String,
    #[tabled(rename = "Run ID")]
    run_id: String,
    #[tabled(rename = "Type")]
    item_type: String,
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
            eprintln!("This action cannot be undone. Use --force to skip this prompt.");
            if !crate::confirm::confirm("Type 'yes' to confirm:", &["yes"], "--force")? {
                println!("Deletion cancelled.");
                return Ok(());
            }
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

        let items = client
            .add_runs_to_annotation_queue(args.queue_id, run_ids)
            .await?;

        println!("Added {} runs to queue {}", items.len(), args.queue_id);

        Ok(())
    }

    async fn execute_remove_run(args: &RemoveRunArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let removed = client
            .delete_run_from_annotation_queue(args.queue_id, args.run_id)
            .await?;

        if !removed {
            return Err(crate::error::CliError::Other(anyhow::anyhow!(
                "Run {} is not in queue {}",
                args.run_id,
                args.queue_id
            )));
        }
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

        let items = collect_items(&client, args).await?;

        if args.json {
            println!("{}", items_json(&items)?);
        } else {
            let rows: Vec<QueueItemRow> = items
                .iter()
                .map(|(status, item)| QueueItemRow {
                    id: item.id.to_string().chars().take(8).collect(),
                    run_id: item
                        .run_id
                        .map(|id| id.to_string().chars().take(8).collect())
                        .unwrap_or_else(|| "-".to_string()),
                    item_type: item
                        .item_type
                        .map(|t| t.as_str().to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    status: status.as_str().to_string(),
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

/// An item and the review section it was listed from.
type SectionItem = (AnnotationQueueItemStatus, AnnotationQueueListItem);

/// Lists the items in the sections `--status` names (every section without it),
/// at most `--limit` in all.
async fn collect_items(client: &LangchainClient, args: &ItemsArgs) -> Result<Vec<SectionItem>> {
    let statuses = match args.status.as_deref() {
        Some("needs-my-review") => vec![AnnotationQueueItemStatus::NeedsMyReview],
        Some("needs-others-review") => vec![AnnotationQueueItemStatus::NeedsOthersReview],
        Some("archived") => vec![AnnotationQueueItemStatus::Archived],
        _ => AnnotationQueueItemStatus::ALL.to_vec(),
    };

    let mut items = Vec::new();
    for status in statuses {
        let remaining = args.limit as usize - items.len();
        if remaining == 0 {
            break;
        }
        let mut params = ListAnnotationQueueItemsParams::new(status);
        params.page_size = Some(args.limit.clamp(1, 100));
        let section = client
            .collect_annotation_queue_items(args.queue_id, params, Some(remaining))
            .await?;
        items.extend(section.into_iter().map(|item| (status, item)));
    }
    Ok(items)
}

/// The `--json` output: each item with a `status` field naming its section.
fn items_json(items: &[SectionItem]) -> Result<String> {
    let json_items = items
        .iter()
        .map(|(status, item)| {
            let mut value = serde_json::to_value(item)?;
            value["status"] = serde_json::json!(status.as_str());
            Ok(value)
        })
        .collect::<std::result::Result<Vec<_>, serde_json::Error>>()?;
    Ok(serde_json::to_string_pretty(&json_items)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_row_truncation() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Queue",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "queue_type": "single",
            "description": "A very long description that should be truncated for display",
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z"
        }"#;

        let queue: AnnotationQueue = serde_json::from_str(json).unwrap();
        let row = QueueRow::from(&queue);

        assert_eq!(row.id, "12345678");
        assert!(row.description.ends_with("..."));
    }

    const QUEUE: &str = "12345678-1234-1234-1234-123456789012";

    /// A mock LangSmith that holds one item in each review section and records
    /// which sections the CLI asked for.
    struct SectionServer {
        server: mockito::ServerGuard,
        mocks: Vec<(AnnotationQueueItemStatus, Uuid, mockito::Mock)>,
    }

    impl SectionServer {
        async fn new(listed: &[AnnotationQueueItemStatus]) -> Self {
            let mut server = mockito::Server::new_async().await;
            let mut mocks = Vec::new();
            for status in AnnotationQueueItemStatus::ALL {
                let item_id = Uuid::new_v4();
                let body = serde_json::json!({"items": [{
                    "id": item_id,
                    "queue_id": QUEUE,
                    "item_type": "RUN",
                    "run_id": Uuid::new_v4(),
                    "added_at": "2026-01-01T00:00:00Z"
                }]});
                let mock = server
                    .mock(
                        "GET",
                        format!("/api/v1/platform/annotation-queues/{QUEUE}/items").as_str(),
                    )
                    .match_query(mockito::Matcher::UrlEncoded(
                        "status".into(),
                        status.as_str().into(),
                    ))
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(body.to_string())
                    .expect(usize::from(listed.contains(&status)))
                    .create_async()
                    .await;
                mocks.push((status, item_id, mock));
            }
            Self { server, mocks }
        }

        fn client(&self) -> LangchainClient {
            LangchainClient::with_base_urls(
                langstar_sdk::AuthConfig::new(Some("test_key".to_string()), None, None),
                self.server.url(),
                self.server.url(),
                self.server.url(),
            )
            .unwrap()
        }

        fn item_id(&self, status: AnnotationQueueItemStatus) -> String {
            let (_, id, _) = self.mocks.iter().find(|(s, _, _)| *s == status).unwrap();
            id.to_string()
        }

        async fn assert_sections_listed(&self) {
            for (_, _, mock) in &self.mocks {
                mock.assert_async().await;
            }
        }
    }

    /// Runs what `queue items <queue> [--status <status>] --json` runs, listing
    /// and JSON output, against `server` and returns each printed item's (id, status).
    async fn items_output(server: &SectionServer, status: Option<&str>) -> Vec<(String, String)> {
        let args = ItemsArgs {
            queue_id: Uuid::parse_str(QUEUE).unwrap(),
            limit: 100,
            status: status.map(str::to_string),
            json: true,
        };
        let items = collect_items(&server.client(), &args).await.unwrap();
        let printed: Vec<serde_json::Value> =
            serde_json::from_str(&items_json(&items).unwrap()).unwrap();
        printed
            .iter()
            .map(|item| {
                (
                    item["id"].as_str().unwrap().to_string(),
                    item["status"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    #[tokio::test]
    async fn test_items_status_lists_only_the_named_section() {
        for (flag, status) in [
            ("needs-my-review", AnnotationQueueItemStatus::NeedsMyReview),
            (
                "needs-others-review",
                AnnotationQueueItemStatus::NeedsOthersReview,
            ),
            ("archived", AnnotationQueueItemStatus::Archived),
        ] {
            let server = SectionServer::new(&[status]).await;
            let output = items_output(&server, Some(flag)).await;
            assert_eq!(
                output,
                vec![(server.item_id(status), status.as_str().to_string())],
                "--status {flag}"
            );
            server.assert_sections_listed().await;
        }
    }

    #[tokio::test]
    async fn test_items_without_status_lists_every_section() {
        let server = SectionServer::new(&AnnotationQueueItemStatus::ALL).await;
        let output = items_output(&server, None).await;
        let expected: Vec<(String, String)> = AnnotationQueueItemStatus::ALL
            .iter()
            .map(|status| (server.item_id(*status), status.as_str().to_string()))
            .collect();
        assert_eq!(output, expected);
        server.assert_sections_listed().await;
    }
}
