use crate::config::Config;
use crate::credential_check::{self, Flagged};
use crate::deployment_utils::resolve_deployment;
use crate::error::{CliError, Result};
use crate::export_file::{
    AssistantBody, AssistantExport, AssistantRequires, AssistantSource, FORMAT_VERSION, Kind,
    to_sorted_json,
};
use crate::output::{ColumnMetadata, OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{
    Assistant, CreateAssistantRequest, LangchainClient, LangstarError, UpdateAssistantRequest,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use tabled::Tabled;

/// Available columns for assistant list text output
const ASSISTANT_COLUMNS: &[&str] = &[
    "assistant_id",
    "name",
    "graph_id",
    "description",
    "created_at",
];

/// Commands for interacting with LangGraph Assistants
#[derive(Debug, Subcommand)]
pub enum AssistantCommands {
    /// List all assistants
    List {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Maximum number of assistants to return
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Number of assistants to skip
        #[arg(short, long, default_value = "0")]
        offset: u32,

        /// Select specific columns for text output (comma-separated)
        /// Available: assistant_id, name, graph_id, description, created_at
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Show available columns for text output
        #[arg(long)]
        show_columns: bool,
    },

    /// Search for assistants by name
    Search {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Get details of a specific assistant
    Get {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Assistant ID
        assistant_id: String,
    },

    /// Create a new assistant
    Create {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Graph ID to base the assistant on
        #[arg(short, long)]
        graph_id: String,

        /// Name for the assistant
        #[arg(short, long)]
        name: String,

        /// Description for the assistant
        #[arg(short, long)]
        description: Option<String>,

        /// Configuration JSON file path
        #[arg(long)]
        config_file: Option<String>,

        /// Configuration JSON (inline)
        #[arg(long, conflicts_with = "config_file")]
        config: Option<String>,
    },

    /// Update an existing assistant
    Update {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Assistant ID to update
        assistant_id: String,

        /// Updated name
        #[arg(short, long)]
        name: Option<String>,

        /// Updated description
        #[arg(short, long)]
        description: Option<String>,

        /// Configuration JSON file path
        #[arg(long)]
        config_file: Option<String>,

        /// Configuration JSON (inline)
        #[arg(long, conflicts_with = "config_file")]
        config: Option<String>,
    },

    /// Export an assistant to a JSON file that can recreate it on another
    /// deployment
    ///
    /// The file holds the assistant's name, description, graph ID,
    /// configuration, context and metadata, and where it came from. Before it
    /// writes, export checks every string in the assistant for a credential and,
    /// when it finds one, names each path and writes no file.
    Export {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Assistant ID
        assistant_id: String,

        /// Write the file here. Without it, stdout holds only the file.
        #[arg(long, value_name = "PATH")]
        output: Option<PathBuf>,

        /// JSON Pointer, inside the assistant, of a string that names a prompt,
        /// such as /config/configurable/system_prompt holding support-system:prod.
        /// Repeat for each prompt.
        #[arg(long = "prompt-ref", value_name = "JSON_POINTER")]
        prompt_refs: Vec<String>,

        /// JSON Pointer, inside the assistant, of a string the credential check
        /// flags and you have read and found to hold no credential. Export writes
        /// it and prints a warning. Repeat for each string.
        #[arg(long = "not-a-secret", value_name = "JSON_POINTER")]
        not_a_secret: Vec<String>,
    },

    /// Delete an assistant
    Delete {
        /// Deployment name or ID (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// Assistant ID to delete
        assistant_id: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        force: bool,
    },
}

/// Simplified assistant info for table display
#[derive(Debug, Tabled)]
struct AssistantRow {
    #[tabled(rename = "ID")]
    assistant_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Graph ID")]
    graph_id: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&Assistant> for AssistantRow {
    fn from(assistant: &Assistant) -> Self {
        Self {
            assistant_id: if assistant.assistant_id.len() > 16 {
                format!("{}...", &assistant.assistant_id[..13])
            } else {
                assistant.assistant_id.clone()
            },
            name: if assistant.name.len() > 30 {
                format!("{}...", &assistant.name[..27])
            } else {
                assistant.name.clone()
            },
            graph_id: if assistant.graph_id.len() > 16 {
                format!("{}...", &assistant.graph_id[..13])
            } else {
                assistant.graph_id.clone()
            },
            created_at: assistant
                .created_at
                .as_ref()
                .and_then(|s| s.split('T').next())
                .unwrap_or("N/A")
                .to_string(),
        }
    }
}

/// Implement ColumnMetadata for Assistant to support text output
impl ColumnMetadata for Assistant {
    fn available_columns() -> Vec<&'static str> {
        ASSISTANT_COLUMNS.to_vec()
    }

    fn render_tsv(&self, columns: &[String]) -> String {
        columns
            .iter()
            .map(|col| match col.as_str() {
                "assistant_id" => self.assistant_id.clone(),
                "name" => self.name.replace(['\t', '\n'], " "),
                "graph_id" => self.graph_id.clone(),
                "description" => self
                    .description
                    .clone()
                    .unwrap_or_default()
                    .replace(['\t', '\n'], " "),
                "created_at" => self
                    .created_at
                    .clone()
                    .unwrap_or_default()
                    .replace(['\t', '\n'], " "),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\t")
    }
}

impl AssistantCommands {
    /// Execute the assistant command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        // Extract deployment name from command
        let deployment_name = match self {
            AssistantCommands::List { deployment, .. } => deployment,
            AssistantCommands::Search { deployment, .. } => deployment,
            AssistantCommands::Get { deployment, .. } => deployment,
            AssistantCommands::Create { deployment, .. } => deployment,
            AssistantCommands::Update { deployment, .. } => deployment,
            AssistantCommands::Export { deployment, .. } => deployment,
            AssistantCommands::Delete { deployment, .. } => deployment,
        };

        // Refuse before any network lookup when there is nobody to ask.
        if let AssistantCommands::Delete { force: false, .. } = self {
            crate::confirm::require_terminal("--force")?;
        }

        // Resolve deployment to URL
        let deployment = resolve_deployment(config, deployment_name).await?;

        // Create client with custom deployment URL and the deployment's workspace
        let client = deployment.client(config)?;
        let formatter = OutputFormatter::new(format);

        match self {
            AssistantCommands::List {
                deployment: _,
                limit,
                offset,
                columns,
                show_columns,
            } => {
                // Handle --show-columns flag: display available columns and exit
                if *show_columns {
                    println!("Available columns for assistant list:");
                    for col in ASSISTANT_COLUMNS {
                        println!("  {}", col);
                    }
                    println!(
                        "\nUsage: langstar assistant list --deployment <name> -f text --columns assistant_id,name"
                    );
                    return Ok(());
                }

                // Validate --columns if provided
                if let Some(cols) = columns {
                    for col in cols {
                        if !ASSISTANT_COLUMNS.contains(&col.as_str()) {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid column '{}'. Available columns: {}",
                                col,
                                ASSISTANT_COLUMNS.join(", ")
                            )));
                        }
                    }
                }

                formatter.info(&format!(
                    "Fetching assistants (limit: {}, offset: {})...",
                    limit, offset
                ));

                let assistants = client
                    .assistants()
                    .list(Some(*limit), Some(*offset))
                    .await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&json!({ "assistants": assistants }))?;
                    }
                    OutputFormat::Text => {
                        // Use provided columns or default to all columns
                        formatter.print_text(&assistants, columns.as_deref())?;
                    }
                    OutputFormat::Table => {
                        let rows: Vec<AssistantRow> = assistants.iter().map(|a| a.into()).collect();
                        formatter.print_table(&rows)?;

                        if assistants.is_empty() {
                            eprintln!("\nℹ No assistants found");
                        } else {
                            eprintln!(
                                "\nℹ Found {} assistant{}",
                                assistants.len(),
                                if assistants.len() == 1 { "" } else { "s" }
                            );
                        }
                    }
                }

                Ok(())
            }

            AssistantCommands::Search {
                deployment: _,
                query,
                limit,
            } => {
                formatter.info(&format!("Searching for assistants matching '{}'...", query));

                let assistants = client.assistants().search(query, Some(*limit)).await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({ "assistants": assistants }))?;
                } else {
                    let rows: Vec<AssistantRow> = assistants.iter().map(|a| a.into()).collect();
                    formatter.print_table(&rows)?;

                    if assistants.is_empty() {
                        eprintln!("\nℹ No assistants found matching '{}'", query);
                    } else {
                        eprintln!(
                            "\nℹ Found {} matching assistant{}",
                            assistants.len(),
                            if assistants.len() == 1 { "" } else { "s" }
                        );
                    }
                }

                Ok(())
            }

            AssistantCommands::Get {
                deployment: _,
                assistant_id,
            } => {
                formatter.info(&format!("Fetching assistant '{}'...", assistant_id));

                let assistant = client.assistants().get(assistant_id).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    let row = AssistantRow::from(&assistant);
                    formatter.print_table(&[row])?;

                    // Show additional details in table mode
                    eprintln!("\nℹ Assistant Details:");
                    eprintln!("  ID: {}", assistant.assistant_id);
                    eprintln!("  Name: {}", assistant.name);
                    eprintln!("  Graph ID: {}", assistant.graph_id);
                    if let Some(description) = &assistant.description {
                        eprintln!("  Description: {}", description);
                    }
                    if let Some(created) = &assistant.created_at {
                        eprintln!("  Created: {}", created);
                    }
                    if let Some(updated) = &assistant.updated_at {
                        eprintln!("  Updated: {}", updated);
                    }
                    if let Some(config) = &assistant.config {
                        eprintln!("  Config: {}", serde_json::to_string_pretty(config)?);
                    }
                    if let Some(metadata) = &assistant.metadata {
                        eprintln!("  Metadata: {}", serde_json::to_string_pretty(metadata)?);
                    }
                }

                Ok(())
            }

            AssistantCommands::Create {
                deployment: _,
                graph_id,
                name,
                description,
                config_file,
                config,
            } => {
                formatter.info(&format!("Creating assistant '{}'...", name));

                // Parse config from file or inline JSON
                let config_value = if let Some(file_path) = config_file {
                    let content = std::fs::read_to_string(file_path)?;
                    Some(serde_json::from_str(&content)?)
                } else if let Some(json_str) = config {
                    Some(serde_json::from_str(json_str)?)
                } else {
                    None
                };

                let request = CreateAssistantRequest {
                    assistant_id: None,
                    graph_id: graph_id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    config: config_value,
                    context: None,
                    metadata: None,
                    if_exists: None,
                };

                let assistant = client.assistants().create(&request).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    formatter.print_table(&[AssistantRow::from(&assistant)])?;
                    eprintln!("\n✓ Successfully created assistant '{}'", assistant.name);
                    eprintln!("  ID: {}", assistant.assistant_id);
                }

                Ok(())
            }

            AssistantCommands::Update {
                deployment: _,
                assistant_id,
                name,
                description,
                config_file,
                config,
            } => {
                formatter.info(&format!("Updating assistant '{}'...", assistant_id));

                // Parse config from file or inline JSON
                let config_value = if let Some(file_path) = config_file {
                    let content = std::fs::read_to_string(file_path)?;
                    Some(serde_json::from_str(&content)?)
                } else if let Some(json_str) = config {
                    Some(serde_json::from_str(json_str)?)
                } else {
                    None
                };

                let request = UpdateAssistantRequest {
                    name: name.clone(),
                    graph_id: None,
                    description: description.clone(),
                    config: config_value,
                    context: None,
                    metadata: None,
                };

                let assistant = client.assistants().update(assistant_id, &request).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    formatter.print_table(&[AssistantRow::from(&assistant)])?;
                    eprintln!("\n✓ Successfully updated assistant '{}'", assistant.name);
                }

                Ok(())
            }

            AssistantCommands::Export {
                deployment: _,
                assistant_id,
                output,
                prompt_refs,
                not_a_secret,
            } => {
                let options = ExportOptions {
                    output: output.as_deref(),
                    prompt_refs,
                    not_a_secret,
                };
                export_assistant(
                    &client,
                    &deployment.id,
                    assistant_id,
                    &options,
                    &mut std::io::stdout(),
                    &mut std::io::stderr(),
                )
                .await
            }

            AssistantCommands::Delete {
                deployment: _,
                assistant_id,
                force,
            } => {
                if !force {
                    eprintln!(
                        "⚠ This will permanently delete assistant '{}'",
                        assistant_id
                    );
                    if !crate::confirm::confirm("Continue? [y/N]:", &["y", "yes"], "--force")? {
                        eprintln!("❌ Cancelled");
                        return Ok(());
                    }
                }

                formatter.info(&format!("Deleting assistant '{}'...", assistant_id));

                client.assistants().delete(assistant_id).await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({ "deleted": assistant_id }))?;
                } else {
                    eprintln!("✓ Successfully deleted assistant '{}'", assistant_id);
                }

                Ok(())
            }
        }
    }
}

/// What `assistant export` was asked to do beyond reading the assistant.
struct ExportOptions<'a> {
    /// The file to write, or `None` for stdout.
    output: Option<&'a Path>,
    /// The `--prompt-ref` pointers.
    prompt_refs: &'a [String],
    /// The `--not-a-secret` pointers.
    not_a_secret: &'a [String],
}

/// Read one assistant from the deployment the client points at, check it, and
/// write its export file to `options.output` or to `stdout`.
///
/// Every refusal returns an error before anything reaches `stdout` or the file,
/// and `main` turns that error into exit status 1. Progress and warnings go to
/// `stderr`, so `stdout` holds only the file.
async fn export_assistant(
    client: &LangchainClient,
    deployment_id: &str,
    assistant_id: &str,
    options: &ExportOptions<'_>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    writeln!(stderr, "Reading assistant '{assistant_id}'...")?;
    let assistant = match client.assistants().get(assistant_id).await {
        Ok(assistant) => assistant,
        Err(LangstarError::ApiError { status: 404, .. }) => {
            return Err(refusal(format!(
                "The deployment has no assistant with ID '{assistant_id}'."
            )));
        }
        Err(error) => return Err(error.into()),
    };

    if is_default_assistant(&assistant) {
        return Err(refusal(format!(
            "Assistant '{assistant_id}' is the default assistant for graph '{}'. Every \
             deployment creates that assistant for each graph it serves, so langstar does \
             not export it. Create an assistant with 'langstar assistant create' and \
             export that one.",
            assistant.graph_id
        )));
    }

    let version = assistant
        .version
        .and_then(|version| i32::try_from(version).ok())
        .ok_or_else(|| {
            refusal(format!(
                "The deployment returned assistant '{assistant_id}' without a version, which \
                 the export file records."
            ))
        })?;

    let body = AssistantBody {
        name: assistant.name,
        description: assistant.description,
        graph_id: assistant.graph_id,
        config: assistant.config,
        context: assistant.context,
        metadata: assistant.metadata.unwrap_or_else(|| json!({})),
    };
    let body_value = serde_json::to_value(&body)?;

    let allowed = check_credentials(&body_value, options.not_a_secret)?;
    let prompts = prompt_references(&body_value, options.prompt_refs)?;

    let export = AssistantExport {
        kind: Kind::Assistant,
        format_version: FORMAT_VERSION,
        source: AssistantSource {
            deployment_id: deployment_id.to_string(),
            assistant_id: assistant.assistant_id,
            version,
        },
        assistant: body,
        requires: AssistantRequires { prompts },
    };
    let text = to_sorted_json(&export)?;

    for item in &allowed {
        writeln!(
            stderr,
            "Warning: writing the string at {} ({}) because --not-a-secret names it.",
            item.path, item.rule
        )?;
    }

    match options.output {
        Some(path) => {
            std::fs::write(path, text).map_err(|source| {
                refusal(format!("Could not write {}: {}", path.display(), source))
            })?;
            writeln!(
                stderr,
                "Wrote assistant '{}' to {}",
                export.source.assistant_id,
                path.display()
            )?;
        }
        None => {
            stdout.write_all(text.as_bytes())?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// An error that `main` prints as it stands, with no prefix.
fn refusal(message: String) -> CliError {
    CliError::Other(anyhow::anyhow!(message))
}

/// Whether this is the assistant the deployment created for one of its graphs,
/// which the Agent Server marks with `created_by: system` in its metadata.
fn is_default_assistant(assistant: &Assistant) -> bool {
    assistant
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("created_by"))
        .and_then(Value::as_str)
        == Some("system")
}

/// Run the credential check over the assistant object and return the flagged
/// strings that `--not-a-secret` lets through.
///
/// Refuses when a flagged string's path is missing from `not_a_secret`, or
/// when `not_a_secret` names a path where the check flags nothing. The error
/// names each path and rule and holds no part of any flagged string.
fn check_credentials(assistant: &Value, not_a_secret: &[String]) -> Result<Vec<Flagged>> {
    let flagged = credential_check::find_credentials(assistant);
    let named: BTreeSet<&str> = not_a_secret.iter().map(String::as_str).collect();
    let flagged_paths: BTreeSet<&str> = flagged.iter().map(|item| item.path.as_str()).collect();

    let (allowed, refused): (Vec<Flagged>, Vec<Flagged>) = flagged
        .iter()
        .cloned()
        .partition(|item| named.contains(item.path.as_str()));
    let unflagged: Vec<&str> = named
        .iter()
        .copied()
        .filter(|path| !flagged_paths.contains(path))
        .collect();

    let mut problems = Vec::new();
    if !unflagged.is_empty() {
        let mut message = String::from(
            "--not-a-secret names a path where the credential check flags no string:\n",
        );
        for path in &unflagged {
            message.push_str(&format!("  {path}\n"));
        }
        message.push_str("Remove each one from the command.");
        problems.push(message);
    }
    if !refused.is_empty() {
        problems.push(credential_check::refusal_message(&refused));
    }
    if problems.is_empty() {
        Ok(allowed)
    } else {
        Err(refusal(problems.join("\n")))
    }
}

/// Look up each `--prompt-ref` pointer in the assistant object and return the
/// prompt name it holds, keyed by the pointer.
fn prompt_references(assistant: &Value, pointers: &[String]) -> Result<BTreeMap<String, String>> {
    let mut prompts = BTreeMap::new();
    for pointer in pointers {
        let name = assistant
            .pointer(pointer)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                refusal(format!(
                    "--prompt-ref {pointer} names no string in the assistant. Give the JSON \
                     Pointer of the configuration or context value that holds the prompt \
                     name, such as /config/configurable/system_prompt."
                ))
            })?;
        prompts.insert(pointer.clone(), name.to_string());
    }
    Ok(prompts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_row_truncation() {
        let assistant = Assistant {
            assistant_id: "very-long-assistant-id-that-should-be-truncated".to_string(),
            graph_id: "graph-123".to_string(),
            name: "Test Assistant".to_string(),
            description: None,
            config: None,
            context: None,
            metadata: None,
            version: None,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: None,
        };

        let row = AssistantRow::from(&assistant);
        assert!(row.assistant_id.len() <= 16);
        assert_eq!(row.name, "Test Assistant");
    }

    // Task 8.1, "Add langstar assistant export", of the export-import-prompts-and-assistants change

    use langstar_sdk::AuthConfig;
    use tempfile::TempDir;

    const ASSISTANT_ID: &str = "3f1c2b8e-0d4a-4c6e-9b7a-2e5f8d9c1a00";
    const DEPLOYMENT_ID: &str = "deployment-0001";

    /// An assistant as the Agent Server returns it, with settings in both fields.
    fn served_assistant() -> Value {
        json!({
            "assistant_id": ASSISTANT_ID,
            "graph_id": "test_graph",
            "name": "support-bot",
            "description": "Answers support questions.",
            "config": {
                "configurable": {"system_prompt": "support-system:prod", "temperature": 0.2},
                "recursion_limit": 10
            },
            "context": {"system_prompt": "support-system:prod", "temperature": 0.2},
            "metadata": {"team": "support"},
            "version": 3,
            "created_at": "2026-11-01T12:00:00Z",
            "updated_at": "2026-11-02T12:00:00Z"
        })
    }

    /// Set the value at `pointer`, adding objects on the way as needed.
    fn set(value: &mut Value, pointer: &str, new: Value) {
        let mut current = value;
        for token in pointer.trim_start_matches('/').split('/') {
            current = current
                .as_object_mut()
                .expect("each step of the pointer is an object")
                .entry(token.to_string())
                .or_insert_with(|| json!({}));
        }
        *current = new;
    }

    async fn serve(assistant: &Value) -> (mockito::ServerGuard, LangchainClient) {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", format!("/assistants/{ASSISTANT_ID}").as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(assistant.to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();
        (server, client)
    }

    struct Run {
        result: Result<()>,
        stdout: String,
        stderr: String,
    }

    async fn export(
        client: &LangchainClient,
        output: Option<&Path>,
        prompt_refs: &[&str],
        not_a_secret: &[&str],
    ) -> Run {
        let prompt_refs: Vec<String> = prompt_refs.iter().map(|p| p.to_string()).collect();
        let not_a_secret: Vec<String> = not_a_secret.iter().map(|p| p.to_string()).collect();
        let options = ExportOptions {
            output,
            prompt_refs: &prompt_refs,
            not_a_secret: &not_a_secret,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = export_assistant(
            client,
            DEPLOYMENT_ID,
            ASSISTANT_ID,
            &options,
            &mut stdout,
            &mut stderr,
        )
        .await;
        Run {
            result,
            stdout: String::from_utf8(stdout).unwrap(),
            stderr: String::from_utf8(stderr).unwrap(),
        }
    }

    /// Whether `message` holds `secret` or any eight-character run of it.
    fn leaks(message: &str, secret: &str) -> bool {
        let chars: Vec<char> = secret.chars().collect();
        message.contains(secret)
            || chars
                .windows(8)
                .any(|run| message.contains(&run.iter().collect::<String>()))
    }

    /// Serve an assistant holding `secret` at `pointer`, export it to a file and
    /// to stdout, and assert that each run refuses, names the path and the rule
    /// without the string, writes no file and prints nothing to stdout.
    async fn assert_refused(pointer: &str, secret: &str, rule: credential_check::Rule) {
        let mut assistant = served_assistant();
        set(&mut assistant, pointer, json!(secret));
        let (_server, client) = serve(&assistant).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        for output in [Some(file.as_path()), None] {
            let run = export(&client, output, &[], &[]).await;
            let message = run
                .result
                .expect_err("export of an assistant holding a credential should fail")
                .to_string();
            assert!(
                message.contains(&format!("{pointer} ({rule})")),
                "expected {pointer} and the rule {rule} in: {message}"
            );
            assert!(
                !leaks(&message, secret),
                "the error should hold no part of the string: {message}"
            );
            assert!(
                !leaks(&run.stderr, secret),
                "stderr should hold no part of the string: {}",
                run.stderr
            );
            assert_eq!(run.stdout, "", "export should print nothing to stdout");
            assert!(!file.exists(), "export should write no file");
        }
    }

    #[tokio::test]
    async fn refuses_an_api_key_in_the_configuration() {
        assert_refused(
            "/config/configurable/OPENAI_API_KEY",
            "Qm4tVx8Lp2Ra7Nc1",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_an_api_key_in_the_context() {
        assert_refused(
            "/context/anthropic_api_key",
            "Hy6dKw3Zs9Fb2Te5",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_an_api_key_in_the_metadata() {
        assert_refused(
            "/metadata/openai_api_key",
            "Ju8cPn5Gx1Lv4Wq7",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_an_access_token_in_the_context() {
        assert_refused(
            "/context/github_access_token",
            "Rb2mXs7Kd4Yh9Ef3",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_a_database_password_in_the_configuration() {
        assert_refused(
            "/config/configurable/db/password",
            "Wn5qTz1Vc8Gp3Lk6",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_a_connection_string_with_a_password() {
        assert_refused(
            "/config/configurable/checkpoint_uri",
            "postgresql://app:Zq7vLm2Rx9Wd@db.example.com/agent",
            credential_check::Rule::UrlWithPassword,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_an_authorization_header() {
        assert_refused(
            "/config/configurable/tool_headers/Authorization",
            "Bearer Zq7vLm2Rx9WdKp4TnB8sYc3H",
            credential_check::Rule::KeyName,
        )
        .await;
    }

    #[tokio::test]
    async fn refuses_a_private_key_in_the_metadata() {
        assert_refused(
            "/metadata/deploy_note",
            "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAE\n-----END OPENSSH PRIVATE KEY-----",
            credential_check::Rule::PemPrivateKey,
        )
        .await;
    }

    #[tokio::test]
    async fn writes_a_secret_reference() {
        let reference = json!({"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]});
        let mut assistant = served_assistant();
        set(
            &mut assistant,
            "/config/configurable/openai_api_key",
            reference.clone(),
        );
        let (_server, client) = serve(&assistant).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        let run = export(&client, Some(&file), &[], &[]).await;

        run.result.expect("a secret reference holds no credential");
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(
            written.pointer("/assistant/config/configurable/openai_api_key"),
            Some(&reference)
        );
    }

    #[tokio::test]
    async fn writes_a_flagged_string_named_by_not_a_secret() {
        let pointer = "/config/configurable/token_budget_policy";
        let mut assistant = served_assistant();
        set(&mut assistant, pointer, json!("strict"));
        let (_server, client) = serve(&assistant).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        let run = export(&client, Some(&file), &[], &[pointer]).await;

        run.result.expect("--not-a-secret lets the string through");
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(
            written.pointer(&format!("/assistant{pointer}")),
            Some(&json!("strict"))
        );
        assert!(
            run.stderr.contains(pointer),
            "stderr should name the path: {}",
            run.stderr
        );
    }

    #[tokio::test]
    async fn refuses_a_flagged_string_without_not_a_secret() {
        let pointer = "/config/configurable/token_budget_policy";
        let mut assistant = served_assistant();
        set(&mut assistant, pointer, json!("strict"));
        let (_server, client) = serve(&assistant).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        let run = export(&client, Some(&file), &[], &[]).await;

        let message = run.result.expect_err("export should refuse").to_string();
        assert!(message.contains(pointer), "{message}");
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn refuses_not_a_secret_at_a_string_nothing_flags() {
        let (_server, client) = serve(&served_assistant()).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        let run = export(&client, Some(&file), &[], &["/description"]).await;

        let message = run.result.expect_err("export should refuse").to_string();
        assert!(message.contains("/description"), "{message}");
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn records_a_prompt_reference() {
        let (_server, client) = serve(&served_assistant()).await;

        let run = export(&client, None, &["/config/configurable/system_prompt"], &[]).await;

        run.result.expect("the pointer names a string");
        let written: Value = serde_json::from_str(&run.stdout).unwrap();
        assert_eq!(
            written["requires"]["prompts"],
            json!({"/config/configurable/system_prompt": "support-system:prod"})
        );
    }

    #[tokio::test]
    async fn refuses_a_prompt_reference_that_names_nothing() {
        let (_server, client) = serve(&served_assistant()).await;

        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        let pointer = "/config/configurable/missing_prompt";
        let run = export(&client, Some(&file), &[pointer], &[]).await;

        let message = run.result.expect_err("export should refuse").to_string();
        assert!(message.contains(pointer), "{message}");
        assert!(!file.exists());
        assert_eq!(run.stdout, "");
    }

    #[tokio::test]
    async fn names_an_assistant_the_deployment_lacks() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", format!("/assistants/{ASSISTANT_ID}").as_str())
            .with_status(404)
            .with_body(r#"{"detail": "Assistant not found"}"#)
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();

        let run = export(&client, None, &[], &[]).await;

        let message = run.result.expect_err("export should fail").to_string();
        assert!(message.contains(ASSISTANT_ID), "{message}");
        assert_eq!(run.stdout, "");
    }

    #[tokio::test]
    async fn refuses_a_graphs_default_assistant() {
        let mut assistant = served_assistant();
        set(&mut assistant, "/metadata/created_by", json!("system"));
        let (_server, client) = serve(&assistant).await;

        let run = export(&client, None, &[], &[]).await;

        let message = run.result.expect_err("export should refuse").to_string();
        assert!(message.contains("Every deployment creates"), "{message}");
        assert_eq!(run.stdout, "");
    }

    #[tokio::test]
    async fn prints_only_the_file_to_stdout() {
        let (_server, client) = serve(&served_assistant()).await;

        let run = export(&client, None, &[], &[]).await;

        run.result.expect("export should succeed");
        let written: Value = serde_json::from_str(&run.stdout).unwrap();
        assert_eq!(written["kind"], "assistant");
        assert_eq!(written["format_version"], 1);
        assert_eq!(
            written["source"],
            json!({"deployment_id": DEPLOYMENT_ID, "assistant_id": ASSISTANT_ID, "version": 3})
        );
        let mut expected = served_assistant();
        let expected = expected.as_object_mut().unwrap();
        for server_set in ["assistant_id", "version", "created_at", "updated_at"] {
            expected.remove(server_set);
        }
        assert_eq!(written["assistant"], Value::Object(expected.clone()));
        assert!(
            run.stderr.contains("Reading assistant"),
            "progress goes to stderr: {}",
            run.stderr
        );
    }
}
