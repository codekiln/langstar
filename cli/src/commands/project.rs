//! CLI commands for managing LangSmith projects.
//!
//! This module provides the `langstar project` command group for creating,
//! listing, updating, and managing projects (tracing sessions).

use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::{Args, Subcommand};
use langstar_sdk::{LangchainClient, ListProjectsParams, Project, ProjectCreate, ProjectUpdate};
use serde::Serialize;
use tabled::Tabled;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Commands
// ═══════════════════════════════════════════════════════════════════════════

/// Commands for managing LangSmith projects
#[derive(Debug, Subcommand)]
pub enum ProjectCommands {
    /// List projects
    List(ListArgs),
    /// Get details of a specific project
    Get(GetArgs),
    /// Create a new project
    Create(CreateArgs),
    /// Update a project
    Update(UpdateArgs),
    /// Delete a project
    Delete(DeleteArgs),
}

/// Arguments for the `project list` command
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by exact name match
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by name substring
    #[arg(long)]
    pub name_contains: Option<String>,

    /// Maximum number of projects to return
    #[arg(short, long, default_value = "100")]
    pub limit: i64,

    /// Include computed statistics (run counts, latencies, etc.)
    #[arg(long)]
    pub include_stats: bool,

    /// Output format: json or text
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,
}

/// Arguments for the `project get` command
#[derive(Debug, Args)]
pub struct GetArgs {
    /// Project ID or name
    pub id_or_name: String,

    /// Output format: json or text
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,
}

/// Arguments for the `project create` command
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of the project (required)
    pub name: String,

    /// Description of the project
    #[arg(long)]
    pub description: Option<String>,

    /// Metadata as JSON string
    #[arg(long)]
    pub metadata: Option<String>,

    /// Output format: json or text
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,
}

/// Arguments for the `project update` command
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Project ID or name
    pub id_or_name: String,

    /// New name for the project
    #[arg(long)]
    pub name: Option<String>,

    /// New description for the project
    #[arg(long)]
    pub description: Option<String>,

    /// Output format: json or text
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,
}

/// Arguments for the `project delete` command
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Project ID or name
    pub id_or_name: String,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Table Display
// ═══════════════════════════════════════════════════════════════════════════

/// Simplified project info for table display
#[derive(Debug, Tabled, Serialize)]
struct ProjectRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Runs")]
    run_count: String,
    #[tabled(rename = "Last Run")]
    last_run: String,
}

impl From<&Project> for ProjectRow {
    fn from(project: &Project) -> Self {
        let name = project.name.as_deref().unwrap_or("<unnamed>").to_string();

        let description = project
            .description
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(40)
            .collect::<String>();
        let description = if project.description.as_ref().is_some_and(|d| d.len() > 40) {
            format!("{}...", description)
        } else {
            description
        };

        let run_count = project
            .run_count
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());

        let last_run = project
            .last_run_start_time
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());

        Self {
            id: project.id.to_string().chars().take(8).collect(),
            name,
            description,
            run_count,
            last_run,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Parse metadata JSON string into serde_json::Value
fn parse_metadata(s: &str) -> Result<serde_json::Value> {
    serde_json::from_str(s)
        .map_err(|e| crate::error::CliError::Config(format!("Invalid metadata JSON: {}", e)))
}

/// Resolve project ID or name to a UUID
async fn resolve_project_id(client: &LangchainClient, id_or_name: &str) -> Result<Uuid> {
    // Try to parse as UUID first
    if let Ok(uuid) = Uuid::parse_str(id_or_name) {
        return Ok(uuid);
    }

    // Otherwise, search by name
    let params = ListProjectsParams {
        name: Some(id_or_name.to_string()),
        limit: Some(1),
        ..Default::default()
    };

    let projects = client.list_projects(params).await?;

    if projects.is_empty() {
        return Err(crate::error::CliError::Config(format!(
            "Project not found: {}",
            id_or_name
        )));
    }

    Ok(projects[0].id)
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Implementation
// ═══════════════════════════════════════════════════════════════════════════

impl ProjectCommands {
    /// Execute the project command
    pub async fn execute(&self, config: &Config, global_format: OutputFormat) -> Result<()> {
        match self {
            ProjectCommands::List(args) => Self::execute_list(args, config, global_format).await,
            ProjectCommands::Get(args) => Self::execute_get(args, config, global_format).await,
            ProjectCommands::Create(args) => {
                Self::execute_create(args, config, global_format).await
            }
            ProjectCommands::Update(args) => {
                Self::execute_update(args, config, global_format).await
            }
            ProjectCommands::Delete(args) => Self::execute_delete(args, config).await,
        }
    }

    async fn execute_list(
        args: &ListArgs,
        config: &Config,
        global_format: OutputFormat,
    ) -> Result<()> {
        let format = if let Some(fmt) = &args.format {
            OutputFormat::from_str(fmt)?
        } else {
            global_format
        };

        let formatter = OutputFormatter::new(format);

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let params = ListProjectsParams {
            name: args.name.clone(),
            name_contains: args.name_contains.clone(),
            limit: Some(args.limit),
            include_stats: Some(args.include_stats),
            ..Default::default()
        };

        let projects = client.list_projects(params).await?;

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            }
            OutputFormat::Text => {
                for project in &projects {
                    let name = project.name.as_deref().unwrap_or("<unnamed>");
                    let runs = project
                        .run_count
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    println!("{}\t{}\t{}", project.id, name, runs);
                }
            }
            OutputFormat::Table => {
                let rows: Vec<ProjectRow> = projects.iter().map(ProjectRow::from).collect();
                formatter.print_table(&rows)?;
                println!("\nFound {} projects", projects.len());
            }
        }

        Ok(())
    }

    async fn execute_get(
        args: &GetArgs,
        config: &Config,
        global_format: OutputFormat,
    ) -> Result<()> {
        let format = if let Some(fmt) = &args.format {
            OutputFormat::from_str(fmt)?
        } else {
            global_format
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let project_id = resolve_project_id(&client, &args.id_or_name).await?;
        let project = client.get_project(project_id).await?;

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&project)?);
            }
            OutputFormat::Text => {
                let name = project.name.as_deref().unwrap_or("<unnamed>");
                let description = project.description.as_deref().unwrap_or("-");
                let runs = project
                    .run_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "-".to_string());
                println!("{}\t{}\t{}\t{}", project.id, name, description, runs);
            }
            OutputFormat::Table => {
                let name = project.name.as_deref().unwrap_or("<unnamed>");
                println!("Project: {}", name);
                println!("  ID: {}", project.id);
                if let Some(desc) = &project.description {
                    println!("  Description: {}", desc);
                }
                if let Some(runs) = project.run_count {
                    println!("  Runs: {}", runs);
                }
                if let Some(latency) = project.latency_p50 {
                    println!("  Latency P50: {:.2}ms", latency);
                }
                if let Some(latency) = project.latency_p99 {
                    println!("  Latency P99: {:.2}ms", latency);
                }
                if let Some(error_rate) = project.error_rate {
                    println!("  Error Rate: {:.2}%", error_rate * 100.0);
                }
                if let Some(last_run) = project.last_run_start_time {
                    println!("  Last Run: {}", last_run.format("%Y-%m-%dT%H:%M:%SZ"));
                }
            }
        }

        Ok(())
    }

    async fn execute_create(
        args: &CreateArgs,
        config: &Config,
        global_format: OutputFormat,
    ) -> Result<()> {
        let format = if let Some(fmt) = &args.format {
            OutputFormat::from_str(fmt)?
        } else {
            global_format
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let extra = args
            .metadata
            .as_ref()
            .map(|s| parse_metadata(s))
            .transpose()?;

        let request = ProjectCreate {
            name: Some(args.name.clone()),
            description: args.description.clone(),
            extra,
            ..Default::default()
        };

        let project = client.create_project(request).await?;

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&project)?);
            }
            OutputFormat::Text => {
                let name = project.name.as_deref().unwrap_or("<unnamed>");
                println!("{}\t{}", project.id, name);
            }
            OutputFormat::Table => {
                println!("Created project:");
                println!("  ID: {}", project.id);
                println!("  Name: {}", project.name.as_deref().unwrap_or("<unnamed>"));
                if let Some(desc) = &project.description {
                    println!("  Description: {}", desc);
                }
            }
        }

        Ok(())
    }

    async fn execute_update(
        args: &UpdateArgs,
        config: &Config,
        global_format: OutputFormat,
    ) -> Result<()> {
        let format = if let Some(fmt) = &args.format {
            OutputFormat::from_str(fmt)?
        } else {
            global_format
        };

        let formatter = OutputFormatter::new(format);

        if args.name.is_none() && args.description.is_none() {
            formatter.warning("No updates specified. Use --name or --description");
            return Ok(());
        }

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let project_id = resolve_project_id(&client, &args.id_or_name).await?;

        let request = ProjectUpdate {
            name: args.name.clone(),
            description: args.description.clone(),
            ..Default::default()
        };

        let project = client.update_project(project_id, request).await?;

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&project)?);
            }
            OutputFormat::Text => {
                let name = project.name.as_deref().unwrap_or("<unnamed>");
                println!("{}\t{}", project.id, name);
            }
            OutputFormat::Table => {
                println!("Project {} updated successfully", project.id);
                println!("  Name: {}", project.name.as_deref().unwrap_or("<unnamed>"));
                if let Some(desc) = &project.description {
                    println!("  Description: {}", desc);
                }
            }
        }

        Ok(())
    }

    async fn execute_delete(args: &DeleteArgs, config: &Config) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        let project_id = resolve_project_id(&client, &args.id_or_name).await?;

        if !args.yes {
            use std::io::{self, Write};
            eprintln!("Are you sure you want to delete project {}?", project_id);
            eprintln!("This action cannot be undone. Use --yes to skip this prompt.");
            print!("Type 'yes' to confirm: ");
            io::stdout().flush()?;

            let mut confirmation = String::new();
            io::stdin().read_line(&mut confirmation)?;

            if confirmation.trim().to_lowercase() != "yes" {
                println!("Deletion cancelled.");
                return Ok(());
            }
        }

        client.delete_project(project_id).await?;
        println!("Deleted project {}", project_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_row_truncation() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "name": "Test Project",
            "description": "A very long description that should be truncated for display purposes and shown with ellipsis",
            "run_count": 1500,
            "last_run_start_time": "2024-01-15T10:30:00Z"
        }"#;

        let project: Project = serde_json::from_str(json).unwrap();
        let row = ProjectRow::from(&project);

        assert_eq!(row.id, "12345678");
        assert_eq!(row.name, "Test Project");
        assert!(row.description.ends_with("..."));
        assert_eq!(row.run_count, "1500");
    }

    #[test]
    fn test_parse_metadata_valid() {
        let json = r#"{"environment": "production", "version": "1.0.0"}"#;
        let result = parse_metadata(json);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["environment"], "production");
        assert_eq!(value["version"], "1.0.0");
    }

    #[test]
    fn test_parse_metadata_invalid() {
        let invalid_json = r#"not valid json"#;
        let result = parse_metadata(invalid_json);
        assert!(result.is_err());
    }
}
