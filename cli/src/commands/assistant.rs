use crate::compare::{Difference, compare_export};
use crate::config::Config;
use crate::credential_check::{self, Flagged};
use crate::deployment_utils::resolve_deployment;
use crate::error::{CliError, Result};
use crate::export_file::{
    AssistantBody, AssistantDestination, AssistantDestinations, AssistantExport, AssistantRequires,
    AssistantSource, DestinationState, FORMAT_VERSION, Kind, read_assistant_export, read_record,
    record_path_for, to_sorted_json, write_record,
};
use crate::output::{ColumnMetadata, OutputFormat, OutputFormatter};
use crate::report::{Action, Check, EXIT_MATCH, Outcome, ResultReport};
use clap::Subcommand;
use langstar_sdk::{
    Assistant, CreateAssistantRequest, IfExists, LangchainClient, LangstarError,
    UpdateAssistantRequest,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use tabled::Tabled;
use uuid::Uuid;

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

    /// Export an assistant to a JSON file that holds what you need to recreate
    /// it on another deployment
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

    /// Create an assistant on a deployment from an export file, then read it
    /// back and compare it with the file
    ///
    /// Before it creates anything, import checks that the deployment serves the
    /// file's graph and that the workspace holds each prompt under
    /// requires.prompts. It writes the new assistant's ID to
    /// <file stem>.destinations.json, so importing the same file onto the same
    /// deployment again compares that assistant with the file and creates no
    /// other. Exits with 0 when the assistant matches the file, 3 when a check
    /// fails or the assistant differs, and 1 on any other error.
    Import {
        /// Deployment name or ID to create the assistant on (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// The file 'langstar assistant export' wrote
        file: PathBuf,

        /// Read and write the record of created assistant IDs here, instead of
        /// <file stem>.destinations.json beside the export file
        #[arg(long, value_name = "PATH")]
        record: Option<PathBuf>,
    },

    /// Compare an assistant on a deployment with its export file
    ///
    /// Compares the file with the assistant that import recorded for the
    /// deployment in <file stem>.destinations.json, or with the one
    /// --assistant-id names, and lists each field that differs by its JSON
    /// Pointer path with both values. Exits with 0 when they match, 3 when they
    /// differ, and 1 on any other error.
    Compare {
        /// Deployment name or ID that holds the assistant (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// The file 'langstar assistant export' wrote
        file: PathBuf,

        /// Compare the file with this assistant instead of the one the record names
        #[arg(long, value_name = "ID")]
        assistant_id: Option<String>,

        /// Read the record of created assistant IDs here, instead of
        /// <file stem>.destinations.json beside the export file
        #[arg(long, value_name = "PATH")]
        record: Option<PathBuf>,
    },

    /// Compare an assistant with its export file, and check that the
    /// deployment serves its graph and the workspace holds its prompts
    ///
    /// Does what compare does, and also checks that the deployment serves the
    /// file's graph and that the workspace holds each prompt under
    /// requires.prompts. Verify looks for the prompts, and for the deployment,
    /// in the workspace that --workspace-id, LANGSMITH_WORKSPACE_ID or the
    /// config file names. Exits
    /// with 0 when the assistant matches and every check passes, 3 when a
    /// check fails or the assistant differs, and 1 on any other error.
    Verify {
        /// Deployment name or ID that holds the assistant (from 'langstar graph list')
        #[arg(long, required = true)]
        deployment: String,

        /// The file 'langstar assistant export' wrote
        file: PathBuf,

        /// Compare the file with this assistant instead of the one the record names
        #[arg(long, value_name = "ID")]
        assistant_id: Option<String>,

        /// Read the record of created assistant IDs here, instead of
        /// <file stem>.destinations.json beside the export file
        #[arg(long, value_name = "PATH")]
        record: Option<PathBuf>,
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
            AssistantCommands::Import { deployment, .. } => deployment,
            AssistantCommands::Compare { deployment, .. } => deployment,
            AssistantCommands::Verify { deployment, .. } => deployment,
            AssistantCommands::Delete { deployment, .. } => deployment,
        };

        // Refuse before any network lookup when there is nobody to ask.
        if let AssistantCommands::Delete { force: false, .. } = self {
            crate::confirm::require_terminal("--force")?;
        }

        // Refuse before the deployment lookup, whose responses the SDK would
        // write to its debug file as well. Import, compare and verify read an
        // assistant from the destination, which may hold a credential, so they
        // refuse too.
        if let AssistantCommands::Export { .. }
        | AssistantCommands::Import { .. }
        | AssistantCommands::Compare { .. }
        | AssistantCommands::Verify { .. } = self
        {
            refuse_debug_http(std::env::var_os(DEBUG_HTTP).is_some())?;
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

            AssistantCommands::Import {
                deployment: _,
                file,
                record,
            } => {
                let record = record.clone().unwrap_or_else(|| record_path_for(file));
                let report = import_assistant(
                    &client,
                    &deployment.id,
                    file,
                    &record,
                    &mut std::io::stderr(),
                )
                .await?;
                print_report_and_exit(&report, format)
            }

            AssistantCommands::Compare {
                deployment: _,
                file,
                assistant_id,
                record,
            }
            | AssistantCommands::Verify {
                deployment: _,
                file,
                assistant_id,
                record,
            } => {
                let record = record.clone().unwrap_or_else(|| record_path_for(file));
                let target = AssistantToCompare {
                    deployment_name,
                    deployment_id: &deployment.id,
                    assistant_id: assistant_id.as_deref(),
                    record: &record,
                };
                let verify = matches!(self, AssistantCommands::Verify { .. });
                let report =
                    compare_assistant(&client, file, &target, verify, &mut std::io::stderr())
                        .await?;
                print_report_and_exit(&report, format)
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

    let assistant_id = assistant.assistant_id.clone();
    let body = body_of(assistant);
    let body_value = serde_json::to_value(&body)?;

    let allowed = check_credentials(&body_value, options.not_a_secret)?;
    let prompts = prompt_references(&body_value, options.prompt_refs)?;

    let export = AssistantExport {
        kind: Kind::Assistant,
        format_version: FORMAT_VERSION,
        source: AssistantSource {
            deployment_id: deployment_id.to_string(),
            assistant_id,
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

/// The variable that makes the SDK write each response body to a file and
/// print the start of it to stderr.
const DEBUG_HTTP: &str = "LANGSTAR_DEBUG_HTTP";

/// Refuse to export while HTTP debugging is on. The SDK would
/// write the whole response, credentials included, to a file and to stderr
/// before the credential check could refuse it.
fn refuse_debug_http(debug_http: bool) -> Result<()> {
    if debug_http {
        return Err(refusal(format!(
            "{DEBUG_HTTP} is set, and it makes langstar write the assistant as the server \
             returned it to a debug file and to stderr, credentials included. Unset it and \
             run the command again."
        )));
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

/// The assistant as the export file holds it, without its ID, version or
/// timestamps, so `compare_export` can compare an assistant read from a
/// deployment with the file field by field.
fn body_of(assistant: Assistant) -> AssistantBody {
    AssistantBody {
        name: assistant.name,
        description: assistant.description,
        graph_id: assistant.graph_id,
        config: assistant.config,
        context: assistant.context,
        metadata: assistant.metadata.unwrap_or_else(|| json!({})),
    }
}

/// Create the file's assistant on the deployment the client points at, unless
/// the destination record names one an earlier import created there, then read
/// the assistant back and compare it with the file.
///
/// An error from reading the file, checking its prompt pointers or running the
/// destination checks returns before import writes the record. An error from
/// the create, the record write or the read-back can leave a pending or
/// created ID in the record, so the next import finds that assistant. `main`
/// exits with 1 on any error. Import puts a failed check or a difference in
/// the report it returns, and the command exits with status 3 after printing it.
async fn import_assistant(
    client: &LangchainClient,
    deployment_id: &str,
    file: &Path,
    record_path: &Path,
    stderr: &mut dyn Write,
) -> Result<ResultReport> {
    let export = read_assistant_export(file)?;
    let file_assistant = serde_json::to_value(&export.assistant)?;
    check_prompt_pointers(&file_assistant, &export.requires.prompts)?;
    let (config, context) = settings_to_send(&export.assistant)?;

    let mut record: AssistantDestinations = read_record(record_path)?;
    let recorded = record.destinations.get(deployment_id).cloned();

    if let Some(AssistantDestination {
        assistant_id,
        state: DestinationState::Created,
    }) = &recorded
    {
        writeln!(
            stderr,
            "{} records assistant '{assistant_id}' on this deployment. Comparing it with the file...",
            record_path.display()
        )?;
        let differences = read_back(
            client,
            &file_assistant,
            assistant_id,
            IdFrom::Record(record_path),
        )
        .await?;
        return Ok(ResultReport::of(differences)
            .with_action(Action::Unchanged, Some(assistant_id.clone())));
    }

    let checks = destination_checks(client, &export).await?;
    if checks.iter().any(|check| check.outcome == Outcome::Failed) {
        return Ok(ResultReport::default().with_checks(checks));
    }

    // A pending entry means an earlier import may have created the assistant
    // and stopped before it marked the ID created. Sending the same ID with
    // `if_exists: do_nothing` then returns that assistant instead of a second.
    let assistant_id = match recorded {
        Some(entry) => entry.assistant_id,
        None => Uuid::new_v4().to_string(),
    };
    record.destinations.insert(
        deployment_id.to_string(),
        AssistantDestination {
            assistant_id: assistant_id.clone(),
            state: DestinationState::Pending,
        },
    );
    write_record(record_path, &record)?;

    writeln!(
        stderr,
        "Creating assistant '{}' with ID {assistant_id}...",
        shown_at("/assistant/name", &export.assistant.name)
    )?;
    client
        .assistants()
        .create(&CreateAssistantRequest {
            assistant_id: Some(assistant_id.clone()),
            graph_id: export.assistant.graph_id.clone(),
            name: export.assistant.name.clone(),
            description: export.assistant.description.clone(),
            config,
            context,
            metadata: Some(export.assistant.metadata.clone()),
            if_exists: Some(IfExists::DoNothing),
        })
        .await
        .map_err(redacted)?;

    record.destinations.insert(
        deployment_id.to_string(),
        AssistantDestination {
            assistant_id: assistant_id.clone(),
            state: DestinationState::Created,
        },
    );
    write_record(record_path, &record)?;
    writeln!(
        stderr,
        "Wrote assistant ID {assistant_id} to {}",
        record_path.display()
    )?;

    let differences = read_back(
        client,
        &file_assistant,
        &assistant_id,
        IdFrom::Record(record_path),
    )
    .await?;
    Ok(ResultReport::of(differences)
        .with_checks(checks)
        .with_action(Action::Create, Some(assistant_id)))
}

/// Where a command found the ID of the assistant it reads, which the error
/// names when the deployment has no assistant with that ID.
enum IdFrom<'a> {
    /// The destination record at this path.
    Record(&'a Path),
    /// The `--assistant-id` flag.
    Flag,
}

/// Read the assistant with `assistant_id` from the deployment and compare it
/// with the file's assistant object.
async fn read_back(
    client: &LangchainClient,
    file_assistant: &Value,
    assistant_id: &str,
    id_from: IdFrom<'_>,
) -> Result<Vec<Difference>> {
    let assistant = match client.assistants().get(assistant_id).await {
        Ok(assistant) => assistant,
        Err(LangstarError::ApiError { status: 404, .. }) => {
            return Err(refusal(match id_from {
                IdFrom::Record(record_path) => format!(
                    "{} records assistant '{assistant_id}' on this deployment, and the \
                     deployment has no assistant with that ID. Remove the deployment's entry \
                     from the record to create the assistant again.",
                    record_path.display()
                ),
                IdFrom::Flag => format!(
                    "The deployment has no assistant with the ID '{assistant_id}' that you \
                     passed with --assistant-id."
                ),
            }));
        }
        Err(error) => return Err(redacted(error)),
    };
    let destination = serde_json::to_value(body_of(assistant))?;
    Ok(compare_export(
        &json!({ "assistant": file_assistant }),
        &json!({ "assistant": destination }),
    ))
}

/// Print the report and exit with the status it gives.
fn print_report_and_exit(report: &ResultReport, format: OutputFormat) -> Result<()> {
    report.print(format)?;
    match report.exit_code() {
        EXIT_MATCH => Ok(()),
        code => Err(CliError::Exit(code)),
    }
}

/// The assistant that `assistant compare` and `assistant verify` read.
struct AssistantToCompare<'a> {
    /// The deployment name or ID the user gave, which an error names.
    deployment_name: &'a str,
    /// The deployment's ID, which keys its entry in the destination record.
    deployment_id: &'a str,
    /// The `--assistant-id` flag, which takes the place of the record's entry.
    assistant_id: Option<&'a str>,
    /// The destination record to read the assistant ID from.
    record: &'a Path,
}

/// Compare the file's assistant with the one `target` names on the deployment
/// the client points at. With `verify`, also run the graph check and the prompt
/// check that import runs before it writes.
///
/// When reading the file, checking its prompt pointers, finding the assistant
/// ID or reading the assistant fails, this function returns the error, and
/// `main` exits with status 1.
/// A difference or a failed check goes in the report, and the command exits
/// with status 3 after printing it. Neither command writes anything.
async fn compare_assistant(
    client: &LangchainClient,
    file: &Path,
    target: &AssistantToCompare<'_>,
    verify: bool,
    stderr: &mut dyn Write,
) -> Result<ResultReport> {
    let export = read_assistant_export(file)?;
    let file_assistant = serde_json::to_value(&export.assistant)?;
    check_prompt_pointers(&file_assistant, &export.requires.prompts)?;

    let (assistant_id, id_from) = match target.assistant_id {
        Some(assistant_id) => (assistant_id.to_string(), IdFrom::Flag),
        None => (
            recorded_assistant_id(target)?,
            IdFrom::Record(target.record),
        ),
    };

    let checks = if verify {
        destination_checks(client, &export).await?
    } else {
        Vec::new()
    };

    writeln!(
        stderr,
        "Comparing assistant '{assistant_id}' with {}...",
        file.display()
    )?;
    let differences = read_back(client, &file_assistant, &assistant_id, id_from).await?;
    Ok(ResultReport::of(differences).with_checks(checks))
}

/// The assistant ID that the destination record holds for the target's
/// deployment, which import marked created.
fn recorded_assistant_id(target: &AssistantToCompare<'_>) -> Result<String> {
    let record: AssistantDestinations = read_record(target.record)?;
    match record.destinations.get(target.deployment_id) {
        Some(AssistantDestination {
            assistant_id,
            state: DestinationState::Created,
        }) => Ok(assistant_id.clone()),
        Some(AssistantDestination {
            assistant_id,
            state: DestinationState::Pending,
        }) => Err(refusal(format!(
            "{} holds assistant '{assistant_id}' for deployment '{}' as pending: an import \
             stopped before it finished creating the assistant. Import the file again to \
             finish it, or name the assistant to compare with --assistant-id <id>.",
            target.record.display(),
            target.deployment_name
        ))),
        None => Err(refusal(format!(
            "{} has no assistant for deployment '{}' ({}). Import the file onto that \
             deployment first, pass --record with the record that import wrote, or name the \
             assistant to compare with --assistant-id <id>.",
            target.record.display(),
            target.deployment_name,
            target.deployment_id
        ))),
    }
}

/// Check that each pointer under `requires.prompts` still holds, in the file's
/// assistant object, the prompt name recorded with it, so that a configuration
/// someone edited in Git without updating `requires.prompts` is refused.
///
/// The error names each pointer and both values, with `<redacted>` in place of
/// each value the credential check flags.
fn check_prompt_pointers(assistant: &Value, prompts: &BTreeMap<String, String>) -> Result<()> {
    let mut problems = Vec::new();
    for (pointer, recorded) in prompts {
        let path = format!("/assistant{pointer}");
        let recorded_shown = credential_check::redact_at(&path, &json!(recorded));
        match assistant.pointer(pointer) {
            Some(Value::String(found)) if found == recorded => {}
            Some(found) => problems.push(format!(
                "  {pointer}: requires.prompts records {recorded_shown}, and the assistant holds {}",
                credential_check::redact_at(&path, found)
            )),
            None => problems.push(format!(
                "  {pointer}: requires.prompts records {recorded_shown}, and the assistant holds \
                 nothing there"
            )),
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    Err(refusal(format!(
        "The export file's requires.prompts no longer matches its assistant:\n{}\nExport the \
         assistant again, or correct requires.prompts.",
        problems.join("\n")
    )))
}

/// The `config` and `context` to send for the file's assistant.
///
/// The Agent Server refuses a request that sets both `config.configurable` and
/// `context`, and fills either from the other, so import sends the settings in
/// one of them: in `config.configurable` when the file holds them there, and in
/// `context` otherwise. A file whose two fields differ is refused, because
/// import cannot tell which one the user meant.
fn settings_to_send(assistant: &AssistantBody) -> Result<(Option<Value>, Option<Value>)> {
    let present = |value: Option<&Value>| value.filter(|value| !value.is_null()).cloned();
    let configurable = present(
        assistant
            .config
            .as_ref()
            .and_then(|config| config.get("configurable")),
    );
    let context = present(assistant.context.as_ref());

    match (configurable, context) {
        (Some(configurable), Some(context)) if configurable != context => Err(refusal(
            "The export file's /assistant/config/configurable and /assistant/context hold \
             different settings. The Agent Server keeps one copy of an assistant's settings in \
             both fields, so import cannot tell which one to send. Make the two match, or \
             remove one, and import again."
                .to_string(),
        )),
        (Some(_), _) => Ok((assistant.config.clone(), None)),
        (None, context) => {
            let config = assistant.config.clone().map(|mut config| {
                if let Some(members) = config.as_object_mut() {
                    members.remove("configurable");
                }
                config
            });
            Ok((config, context))
        }
    }
}

/// Check that the deployment serves the file's graph and that the workspace
/// holds each prompt under `requires.prompts`. Import runs these checks before
/// it writes, and `assistant verify` runs them alongside its comparison.
async fn destination_checks(
    client: &LangchainClient,
    export: &AssistantExport,
) -> Result<Vec<Check>> {
    let mut checks = vec![graph_check(client, &export.assistant.graph_id).await?];
    for (pointer, name) in &export.requires.prompts {
        checks.push(prompt_check(client, pointer, name).await?);
    }
    Ok(checks)
}

/// Whether the deployment serves `graph_id`.
///
/// The Agent Server has no call that lists a deployment's graphs, so this
/// searches its assistants for the graph ID: every graph a deployment serves
/// has a default assistant. The live server answers a graph ID it does not
/// serve with 404 and the graph IDs it does serve, as
/// `reference/research/807-openapi-validation.md` records. Its OpenAPI spec
/// allows a 422 naming `graph_id` instead, so that counts as missing too.
///
/// When the credential check flags the graph ID, `graph_check` writes
/// `<redacted>` in its place in the check's detail, as `prompt_check` does for
/// a prompt name.
async fn graph_check(client: &LangchainClient, graph_id: &str) -> Result<Check> {
    let shown = shown_at("/assistant/graph_id", graph_id);
    let missing = |detail: &str| {
        Check::failed(
            "graph",
            format!("The deployment does not serve graph '{shown}'. {detail}")
                .trim_end()
                .to_string(),
        )
    };
    match client
        .assistants()
        .search_by_graph_id(graph_id, Some(1))
        .await
    {
        Ok(found) if !found.is_empty() => Ok(Check::passed(
            "graph",
            format!("The deployment serves graph '{shown}'."),
        )),
        Ok(_) => Ok(missing("")),
        Err(LangstarError::ApiError {
            status: 404,
            message,
        }) => Ok(missing(&redacted_body(&message))),
        Err(LangstarError::ApiError {
            status: 422,
            message,
        }) if message.contains("graph_id") => Ok(missing("")),
        Err(error) => Err(redacted(error)),
    }
}

/// Whether the workspace holds the prompt `name`, read as
/// `[<owner>/]<handle>[:<tag>]`.
///
/// `get_commit_with_model` encodes the tag as one path segment, so a tag such
/// as `prod#candidate` is looked up whole instead of being cut at the `#`.
///
/// The check's detail prints the name with `<redacted>` in place of a string
/// the credential check flags at `pointer`, since a file can hold an API key
/// there and still match its `requires.prompts`.
async fn prompt_check(client: &LangchainClient, pointer: &str, name: &str) -> Result<Check> {
    let (owner, handle, tag) = split_prompt_name(name);
    let shown = shown_at(&format!("/assistant{pointer}"), name);
    match client
        .prompts()
        .get_commit_with_model(owner, handle, tag)
        .await
    {
        Ok(_) => Ok(Check::passed(
            "prompt",
            format!("The workspace holds {shown}."),
        )),
        Err(LangstarError::ApiError { status: 404, .. }) => Ok(Check::failed(
            "prompt",
            format!("The workspace has no prompt {shown}."),
        )),
        Err(error) => Err(redacted(error)),
    }
}

/// A string from the export file, with `<redacted>` in its place when the
/// credential check flags it at `pointer`.
///
/// Export refuses to write a credential, but someone may edit the file in Git
/// before import reads it, so import checks each string again before printing it.
fn shown_at(pointer: &str, value: &str) -> String {
    match credential_check::redact_at(pointer, &json!(value)) {
        Value::String(shown) => shown,
        other => other.to_string(),
    }
}

/// An error from the Agent Server or LangSmith, with `<redacted>` in place of
/// each string the credential check flags in its body, and without the
/// request URL when the request never got an answer.
///
/// A server that refuses a request can quote the values it refused, as a 422
/// does, and import sends the file's settings, which nobody has checked for a
/// credential since export if someone edited the file in Git.
///
/// Langstar drops the URL from a request that got no answer because a prompt
/// lookup's URL carries the prompt name from the file.
fn redacted(error: LangstarError) -> CliError {
    match error {
        LangstarError::ApiError { status, message } => CliError::Sdk(LangstarError::ApiError {
            status,
            message: redacted_body(&message),
        }),
        LangstarError::HttpError(error) => {
            CliError::Sdk(LangstarError::HttpError(error.without_url()))
        }
        other => other.into(),
    }
}

/// A response body with `<redacted>` in place of each string the credential
/// check flags, whether the body is JSON or plain text.
fn redacted_body(body: &str) -> String {
    let value = serde_json::from_str::<Value>(body).unwrap_or_else(|_| json!(body));
    match credential_check::redact_at("", &value) {
        Value::String(text) => text,
        other => other.to_string(),
    }
}

/// The owner, handle and tag in a prompt name. With no owner, the owner is
/// `-`, which LangSmith reads as the current workspace; with no tag, the tag is
/// `latest`.
fn split_prompt_name(name: &str) -> (&str, &str, &str) {
    let (rest, tag) = name.split_once(':').unwrap_or((name, "latest"));
    let (owner, handle) = rest.split_once('/').unwrap_or(("-", rest));
    (owner, handle, tag)
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

    #[test]
    fn refuses_to_read_the_assistant_while_http_debugging_is_on() {
        let message = refuse_debug_http(true)
            .expect_err("export should refuse while LANGSTAR_DEBUG_HTTP is set")
            .to_string();
        assert!(message.contains("LANGSTAR_DEBUG_HTTP"), "{message}");
        refuse_debug_http(false).expect("export goes ahead without the variable");
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

    // assistant import

    const DESTINATION_ID: &str = "destination-0001";
    const SYSTEM_PROMPT: &str = "/config/configurable/system_prompt";

    /// `served_assistant()` as an export file holds it, without the fields the
    /// server sets.
    fn file_assistant() -> Value {
        let mut assistant = served_assistant();
        let members = assistant.as_object_mut().unwrap();
        for server_set in ["assistant_id", "version", "created_at", "updated_at"] {
            members.remove(server_set);
        }
        assistant
    }

    /// Write an export file holding `assistant` and `prompts` to `directory`.
    fn write_export(directory: &Path, assistant: Value, prompts: Value) -> PathBuf {
        let path = directory.join("support-bot.json");
        let file = json!({
            "kind": "assistant",
            "format_version": 1,
            "source": {"deployment_id": DEPLOYMENT_ID, "assistant_id": ASSISTANT_ID, "version": 3},
            "assistant": assistant,
            "requires": {"prompts": prompts},
        });
        std::fs::write(&path, file.to_string()).unwrap();
        path
    }

    /// An Agent Server that serves `test_graph`, accepts `creates` creates, and
    /// answers every assistant read with `read_back`.
    struct Destination {
        server: mockito::ServerGuard,
        client: LangchainClient,
        create: mockito::Mock,
    }

    async fn destination(read_back: &Value, creates: usize) -> Destination {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(
                json!({"graph_id": "test_graph"}),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!([served_assistant()]).to_string())
            .create_async()
            .await;
        let create = server
            .mock("POST", "/assistants")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(read_back.to_string())
            .expect(creates)
            .create_async()
            .await;
        server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/assistants/[0-9a-f-]+$".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(read_back.to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();
        Destination {
            server,
            client,
            create,
        }
    }

    async fn import(client: &LangchainClient, file: &Path, record: &Path) -> Result<ResultReport> {
        import_assistant(client, DESTINATION_ID, file, record, &mut Vec::new()).await
    }

    fn recorded(record: &Path) -> AssistantDestination {
        let record: AssistantDestinations = read_record(record).unwrap();
        record.destinations[DESTINATION_ID].clone()
    }

    #[tokio::test]
    async fn creates_the_assistant_once_and_finds_it_on_the_next_import() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let destination = destination(&served_assistant(), 1).await;

        let first = import(&destination.client, &file, &record).await.unwrap();
        assert_eq!(first.exit_code(), EXIT_MATCH, "{}", first.to_text());
        assert_eq!(first.action, Some(Action::Create));
        let entry = recorded(&record);
        assert_eq!(entry.state, DestinationState::Created);
        assert_eq!(
            first.destination_id.as_deref(),
            Some(entry.assistant_id.as_str())
        );

        let second = import(&destination.client, &file, &record).await.unwrap();
        assert_eq!(second.exit_code(), EXIT_MATCH, "{}", second.to_text());
        assert_eq!(second.action, Some(Action::Unchanged));
        assert_eq!(recorded(&record), entry);
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn keeps_the_id_when_the_read_back_differs_and_creates_nothing_next_time() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let mut dropped = served_assistant();
        dropped["config"]["configurable"]
            .as_object_mut()
            .unwrap()
            .remove("temperature");
        dropped["context"]
            .as_object_mut()
            .unwrap()
            .remove("temperature");
        let destination = destination(&dropped, 1).await;

        let first = import(&destination.client, &file, &record).await.unwrap();
        assert_eq!(first.exit_code(), 3);
        assert_eq!(
            first.differences[0].path,
            "/assistant/config/configurable/temperature"
        );
        assert_eq!(recorded(&record).state, DestinationState::Created);

        let second = import(&destination.client, &file, &record).await.unwrap();
        assert_eq!(second.exit_code(), 3);
        assert_eq!(second.action, Some(Action::Unchanged));
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn sends_the_pending_id_again() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let pending = "0b6f2c1e-8d3a-4f5b-9c7e-1a2b3c4d5e6f";
        let mut destinations = AssistantDestinations::default();
        destinations.destinations.insert(
            DESTINATION_ID.to_string(),
            AssistantDestination {
                assistant_id: pending.to_string(),
                state: DestinationState::Pending,
            },
        );
        write_record(&record, &destinations).unwrap();
        let mut destination = destination(&served_assistant(), 0).await;
        let same_id = destination
            .server
            .mock("POST", "/assistants")
            .match_body(mockito::Matcher::PartialJson(
                json!({"assistant_id": pending, "if_exists": "do_nothing"}),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(served_assistant().to_string())
            .expect(1)
            .create_async()
            .await;

        let report = import(&destination.client, &file, &record).await.unwrap();

        assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
        assert_eq!(
            recorded(&record),
            AssistantDestination {
                assistant_id: pending.to_string(),
                state: DestinationState::Created,
            }
        );
        same_id.assert_async().await;
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn creates_nothing_when_the_deployment_lacks_the_graph() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        assistant["graph_id"] = json!("test_graph_alt");
        let file = write_export(directory.path(), assistant, json!({}));
        let record = record_path_for(&file);
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(
                json!({"graph_id": "test_graph_alt"}),
            ))
            .with_status(404)
            .with_body(
                json!({"detail": "Graph 'test_graph_alt' not found. Expected one of: ['test_graph']"})
                    .to_string(),
            )
            .create_async()
            .await;

        let report = import(&destination.client, &file, &record).await.unwrap();

        assert_eq!(report.exit_code(), 3);
        assert_eq!(report.action, None);
        assert_eq!(report.checks[0].outcome, Outcome::Failed);
        assert!(report.checks[0].detail.contains("test_graph_alt"));
        assert!(!record.exists(), "import writes no record");
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn reads_a_422_naming_graph_id_as_a_missing_graph() {
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(json!({"graph_id": "other"})))
            .with_status(422)
            .with_body(
                json!({"detail": "body.graph_id: value is not one of 'test_graph'"}).to_string(),
            )
            .create_async()
            .await;
        destination
            .server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(json!({"graph_id": "broken"})))
            .with_status(422)
            .with_body(json!({"detail": "body.limit: must be positive"}).to_string())
            .create_async()
            .await;

        let check = graph_check(&destination.client, "other").await.unwrap();
        assert_eq!(check.outcome, Outcome::Failed);
        assert!(graph_check(&destination.client, "broken").await.is_err());
    }

    #[tokio::test]
    async fn creates_nothing_when_the_workspace_lacks_a_prompt() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(
            directory.path(),
            file_assistant(),
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let record = record_path_for(&file);
        let mut destination = destination(&served_assistant(), 0).await;
        let commit = destination
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(
                    r"^/api/v1/commits/-/support-system/prod(\?.*)?$".to_string(),
                ),
            )
            .with_status(404)
            .with_body(json!({"detail": "Not found"}).to_string())
            .expect(1)
            .create_async()
            .await;

        let report = import(&destination.client, &file, &record).await.unwrap();

        assert_eq!(report.exit_code(), 3);
        let prompt = &report.checks[1];
        assert_eq!(prompt.outcome, Outcome::Failed);
        assert!(prompt.detail.contains("support-system:prod"));
        assert!(!record.exists(), "import writes no record");
        commit.assert_async().await;
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn passes_the_prompt_check_for_a_prompt_the_workspace_holds() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(
            directory.path(),
            file_assistant(),
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let record = record_path_for(&file);
        let mut destination = destination(&served_assistant(), 1).await;
        destination
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(
                    r"^/api/v1/commits/-/support-system/prod(\?.*)?$".to_string(),
                ),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"commit_hash": "abc123", "manifest": {}}).to_string())
            .create_async()
            .await;

        let report = import(&destination.client, &file, &record).await.unwrap();

        assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
        assert_eq!(report.checks[1].outcome, Outcome::Passed);
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn looks_up_a_tag_with_a_reserved_character_whole() {
        let mut server = mockito::Server::new_async().await;
        let whole = server
            .mock(
                "GET",
                mockito::Matcher::Regex(
                    r"^/api/v1/commits/-/support-system/prod%23candidate(\?.*)?$".to_string(),
                ),
            )
            .with_status(404)
            .with_body(json!({"detail": "Not found"}).to_string())
            .expect(1)
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();

        let check = prompt_check(&client, SYSTEM_PROMPT, "support-system:prod#candidate")
            .await
            .unwrap();

        assert_eq!(check.outcome, Outcome::Failed);
        whole.assert_async().await;
    }

    #[tokio::test]
    async fn redacts_a_credential_the_server_quotes_when_it_refuses_the_create() {
        let directory = tempfile::tempdir().unwrap();
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        let mut assistant = file_assistant();
        set(&mut assistant, "/config/configurable/note", json!(token));
        set(&mut assistant, "/context/note", json!(token));
        let file = write_export(directory.path(), assistant, json!({}));
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock("POST", "/assistants")
            .with_status(422)
            .with_body(
                json!({"detail": [{"loc": ["body", "config"], "input": {"note": token}}]})
                    .to_string(),
            )
            .create_async()
            .await;

        let error = import(&destination.client, &file, &record_path_for(&file))
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("422"), "{error}");
        assert!(error.contains("<redacted>"), "{error}");
        assert!(!error.contains(&token), "{error}");
    }

    #[tokio::test]
    async fn redacts_a_credential_shaped_prompt_name_in_the_check() {
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        let mut server = mockito::Server::new_async().await;
        server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/api/v1/commits/".to_string()),
            )
            .with_status(404)
            .with_body(json!({"detail": "Not found"}).to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();

        let check = prompt_check(&client, SYSTEM_PROMPT, &token).await.unwrap();

        assert_eq!(check.outcome, Outcome::Failed);
        assert!(check.detail.contains("<redacted>"), "{}", check.detail);
        assert!(!check.detail.contains(&token), "{}", check.detail);
    }

    #[test]
    fn reads_the_owner_handle_and_tag_of_a_prompt_name() {
        assert_eq!(
            split_prompt_name("support-system"),
            ("-", "support-system", "latest")
        );
        assert_eq!(
            split_prompt_name("my-org/support-system:prod"),
            ("my-org", "support-system", "prod")
        );
        assert_eq!(
            split_prompt_name("-/support-system:prod"),
            ("-", "support-system", "prod")
        );
        assert_eq!(
            split_prompt_name("my-org/support-system:prod:candidate"),
            ("my-org", "support-system", "prod:candidate")
        );
    }

    #[tokio::test]
    async fn redacts_a_credential_shaped_name_and_graph_id() {
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        assistant["name"] = json!(token);
        let file = write_export(directory.path(), assistant, json!({}));
        let destination = destination(&served_assistant(), 1).await;
        let mut stderr = Vec::new();

        let _ = import_assistant(
            &destination.client,
            DESTINATION_ID,
            &file,
            &record_path_for(&file),
            &mut stderr,
        )
        .await;
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.contains("<redacted>"), "{stderr}");
        assert!(!stderr.contains(&token), "{stderr}");

        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/assistants/search")
            .with_status(404)
            .with_body(json!({"detail": format!("Graph '{token}' not found")}).to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();
        let check = graph_check(&client, &token).await.unwrap();
        assert_eq!(check.outcome, Outcome::Failed);
        assert!(check.detail.contains("<redacted>"), "{}", check.detail);
        assert!(!check.detail.contains(&token), "{}", check.detail);
    }

    #[tokio::test]
    async fn leaves_the_prompt_name_out_of_a_connection_failure() {
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        // Nothing listens on port 9 of localhost, so the request fails
        // before any server answers.
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            "http://127.0.0.1:9".to_string(),
            "http://127.0.0.1:9".to_string(),
            "http://127.0.0.1:9".to_string(),
        )
        .unwrap();

        let error = prompt_check(&client, SYSTEM_PROMPT, &token)
            .await
            .unwrap_err()
            .to_string();

        assert!(!error.contains(&token), "{error}");
    }

    #[tokio::test]
    async fn refuses_a_file_whose_settings_fields_differ() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        assistant["context"]["system_prompt"] = json!("support-system:staging");
        let file = write_export(directory.path(), assistant, json!({}));
        let record = record_path_for(&file);
        std::fs::write(
            &record,
            "{\"kind\":\"assistant\",\"format_version\":1,\"destinations\":{}}",
        )
        .unwrap();
        let before = std::fs::read_to_string(&record).unwrap();
        let destination = destination(&served_assistant(), 0).await;

        let error = import(&destination.client, &file, &record)
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("/assistant/config/configurable"), "{error}");
        assert!(error.contains("/assistant/context"), "{error}");
        assert!(!error.contains("support-system"), "{error}");
        assert_eq!(std::fs::read_to_string(&record).unwrap(), before);
        destination.create.assert_async().await;
    }

    #[test]
    fn sends_the_settings_in_one_field() {
        let body =
            |assistant: Value| -> AssistantBody { serde_json::from_value(assistant).unwrap() };
        let settings = json!({"system_prompt": "support-system:prod"});

        let both = body(file_assistant());
        let (config, context) = settings_to_send(&both).unwrap();
        assert_eq!(config, both.config);
        assert_eq!(context, None);

        let mut context_only = file_assistant();
        context_only["config"] = json!({"recursion_limit": 10});
        context_only["context"] = settings.clone();
        let (config, context) = settings_to_send(&body(context_only)).unwrap();
        assert_eq!(config, Some(json!({"recursion_limit": 10})));
        assert_eq!(context, Some(settings.clone()));

        let mut config_only = file_assistant();
        config_only["config"] = json!({"configurable": settings});
        config_only.as_object_mut().unwrap().remove("context");
        let (config, context) = settings_to_send(&body(config_only)).unwrap();
        assert_eq!(config, Some(json!({"configurable": settings})));
        assert_eq!(context, None);
    }

    #[tokio::test]
    async fn finds_no_difference_for_a_file_with_settings_in_one_field() {
        let settings = json!({"system_prompt": "support-system:prod", "temperature": 0.2});
        let mut config_only = file_assistant();
        config_only.as_object_mut().unwrap().remove("context");
        let mut context_only = file_assistant();
        context_only["config"] = json!({"recursion_limit": 10});
        context_only["context"] = settings;

        for assistant in [config_only, context_only] {
            let directory = tempfile::tempdir().unwrap();
            let file = write_export(directory.path(), assistant, json!({}));
            let destination = destination(&served_assistant(), 1).await;

            let report = import(&destination.client, &file, &record_path_for(&file))
                .await
                .unwrap();

            assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
            destination.create.assert_async().await;
        }
    }

    #[tokio::test]
    async fn refuses_a_prompt_export() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("my-prompt.json");
        std::fs::write(
            &file,
            json!({"kind": "prompt", "format_version": 1}).to_string(),
        )
        .unwrap();
        let destination = destination(&served_assistant(), 0).await;

        let error = import(&destination.client, &file, &record_path_for(&file))
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("kind is prompt"), "{error}");
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn refuses_a_prompt_reference_edited_after_export() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        set(&mut assistant, SYSTEM_PROMPT, json!(token));
        set(&mut assistant, "/context/system_prompt", json!(token));
        let file = write_export(
            directory.path(),
            assistant,
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let destination = destination(&served_assistant(), 0).await;

        let error = import(&destination.client, &file, &record_path_for(&file))
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains(SYSTEM_PROMPT), "{error}");
        assert!(error.contains("support-system:prod"), "{error}");
        assert!(error.contains("<redacted>"), "{error}");
        assert!(!error.contains(&token), "{error}");
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn names_a_recorded_assistant_the_deployment_no_longer_has() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let gone = "9e8d7c6b-5a4f-4e3d-8c2b-1a0f9e8d7c6b";
        let mut destinations = AssistantDestinations::default();
        destinations.destinations.insert(
            DESTINATION_ID.to_string(),
            AssistantDestination {
                assistant_id: gone.to_string(),
                state: DestinationState::Created,
            },
        );
        write_record(&record, &destinations).unwrap();
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", format!("/assistants/{gone}").as_str())
            .with_status(404)
            .with_body(json!({"detail": "Assistant not found"}).to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();

        let error = import(&client, &file, &record)
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains(gone), "{error}");
    }

    // Tasks 9.1, "Add langstar assistant compare", and 9.2, "Add langstar
    // assistant verify", of the export-import-prompts-and-assistants change

    const RECORDED_ID: &str = "7a6b5c4d-3e2f-4a1b-9c8d-7e6f5a4b3c2d";

    fn target<'a>(record: &'a Path, assistant_id: Option<&'a str>) -> AssistantToCompare<'a> {
        AssistantToCompare {
            deployment_name: "my-agent-prod",
            deployment_id: DESTINATION_ID,
            assistant_id,
            record,
        }
    }

    async fn compare(
        client: &LangchainClient,
        file: &Path,
        target: &AssistantToCompare<'_>,
        verify: bool,
    ) -> Result<ResultReport> {
        compare_assistant(client, file, target, verify, &mut Vec::new()).await
    }

    /// Write a record at `path` that marks `assistant_id` created on the
    /// destination deployment.
    fn write_created(path: &Path, assistant_id: &str) {
        let mut destinations = AssistantDestinations::default();
        destinations.destinations.insert(
            DESTINATION_ID.to_string(),
            AssistantDestination {
                assistant_id: assistant_id.to_string(),
                state: DestinationState::Created,
            },
        );
        write_record(path, &destinations).unwrap();
    }

    /// A server that answers a read of `RECORDED_ID` alone with `assistant`.
    async fn serve_recorded(
        assistant: &Value,
    ) -> (mockito::ServerGuard, mockito::Mock, LangchainClient) {
        let mut server = mockito::Server::new_async().await;
        let read = server
            .mock("GET", format!("/assistants/{RECORDED_ID}").as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(assistant.to_string())
            .expect(1)
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();
        (server, read, client)
    }

    #[tokio::test]
    async fn compare_reads_the_assistant_id_from_the_record_beside_the_file() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        write_created(&record, RECORDED_ID);
        let mut changed = served_assistant();
        changed["metadata"]["team"] = json!("billing");
        let (_server, read, client) = serve_recorded(&changed).await;

        let report = compare(&client, &file, &target(&record, None), false)
            .await
            .unwrap();

        read.assert_async().await;
        assert_eq!(report.exit_code(), 3, "{}", report.to_text());
        assert_eq!(report.differences.len(), 1, "{}", report.to_text());
        assert_eq!(report.differences[0].path, "/assistant/metadata/team");
        assert_eq!(report.differences[0].file, Some(json!("support")));
        assert_eq!(report.differences[0].destination, Some(json!("billing")));
        assert!(report.checks.is_empty());
        assert_eq!(report.action, None);
    }

    #[tokio::test]
    async fn compare_reads_the_assistant_id_from_the_record_that_record_names() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        std::fs::create_dir(directory.path().join("records")).unwrap();
        let record = directory.path().join("records").join("prod.json");
        write_created(&record, RECORDED_ID);
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        let report = compare(&client, &file, &target(&record, None), false)
            .await
            .unwrap();

        read.assert_async().await;
        assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
        assert!(!record_path_for(&file).exists(), "compare writes no record");
    }

    #[tokio::test]
    async fn compare_reads_the_assistant_that_assistant_id_names_over_the_record() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        write_created(&record, "1b2c3d4e-5f6a-4b7c-8d9e-0f1a2b3c4d5e");
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        let report = compare(&client, &file, &target(&record, Some(RECORDED_ID)), false)
            .await
            .unwrap();

        read.assert_async().await;
        assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
    }

    #[tokio::test]
    async fn compare_without_a_record_entry_names_the_deployment_and_assistant_id() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        let error = compare(&client, &file, &target(&record, None), false)
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("my-agent-prod"), "{error}");
        assert!(error.contains("--assistant-id"), "{error}");
        assert!(!record.exists(), "compare writes no record");
        read.expect(0).assert_async().await;
    }

    #[tokio::test]
    async fn compare_of_a_pending_record_entry_says_to_import_again() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let record = record_path_for(&file);
        let mut destinations = AssistantDestinations::default();
        destinations.destinations.insert(
            DESTINATION_ID.to_string(),
            AssistantDestination {
                assistant_id: RECORDED_ID.to_string(),
                state: DestinationState::Pending,
            },
        );
        write_record(&record, &destinations).unwrap();
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        let error = compare(&client, &file, &target(&record, None), false)
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("pending"), "{error}");
        assert!(error.contains(RECORDED_ID), "{error}");
        assert!(error.contains("--assistant-id"), "{error}");
        read.expect(0).assert_async().await;
    }

    #[tokio::test]
    async fn compare_names_an_assistant_id_the_deployment_lacks() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let gone = "9e8d7c6b-5a4f-4e3d-8c2b-1a0f9e8d7c6b";
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", format!("/assistants/{gone}").as_str())
            .with_status(404)
            .with_body(json!({"detail": "Assistant not found"}).to_string())
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap();

        let error = compare(
            &client,
            &file,
            &target(&record_path_for(&file), Some(gone)),
            false,
        )
        .await
        .unwrap_err()
        .to_string();

        assert!(error.contains(gone), "{error}");
        assert!(error.contains("--assistant-id"), "{error}");
    }

    #[tokio::test]
    async fn compare_refuses_a_prompt_reference_that_names_another_prompt() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        set(
            &mut assistant,
            SYSTEM_PROMPT,
            json!("support-system:staging"),
        );
        set(
            &mut assistant,
            "/context/system_prompt",
            json!("support-system:staging"),
        );
        let file = write_export(
            directory.path(),
            assistant,
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        for verify in [false, true] {
            let error = compare(
                &client,
                &file,
                &target(&record_path_for(&file), Some(RECORDED_ID)),
                verify,
            )
            .await
            .unwrap_err()
            .to_string();

            assert!(error.contains(SYSTEM_PROMPT), "{error}");
            assert!(error.contains("support-system:prod"), "{error}");
            assert!(error.contains("support-system:staging"), "{error}");
        }
        read.expect(0).assert_async().await;
    }

    #[tokio::test]
    async fn compare_redacts_an_api_key_at_a_prompt_reference() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        let token = format!("sk-{}", "a1B2c3D4e5".repeat(4));
        set(&mut assistant, SYSTEM_PROMPT, json!(token));
        set(&mut assistant, "/context/system_prompt", json!(token));
        let file = write_export(
            directory.path(),
            assistant,
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        let error = compare(
            &client,
            &file,
            &target(&record_path_for(&file), Some(RECORDED_ID)),
            false,
        )
        .await
        .unwrap_err()
        .to_string();

        assert!(error.contains(SYSTEM_PROMPT), "{error}");
        assert!(error.contains("support-system:prod"), "{error}");
        assert!(error.contains("<redacted>"), "{error}");
        assert!(!leaks(&error, &token), "{error}");
        read.expect(0).assert_async().await;
    }

    #[tokio::test]
    async fn compare_refuses_a_prompt_export() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("my-prompt.json");
        std::fs::write(
            &file,
            json!({"kind": "prompt", "format_version": 1}).to_string(),
        )
        .unwrap();
        let (_server, read, client) = serve_recorded(&served_assistant()).await;

        for verify in [false, true] {
            let error = compare(
                &client,
                &file,
                &target(&record_path_for(&file), Some(RECORDED_ID)),
                verify,
            )
            .await
            .unwrap_err()
            .to_string();
            assert!(error.contains("kind is prompt"), "{error}");
        }
        read.expect(0).assert_async().await;
    }

    #[tokio::test]
    async fn verify_passes_the_graph_check_and_compares() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(directory.path(), file_assistant(), json!({}));
        let destination = destination(&served_assistant(), 0).await;

        let report = compare(
            &destination.client,
            &file,
            &target(&record_path_for(&file), Some(RECORDED_ID)),
            true,
        )
        .await
        .unwrap();

        assert_eq!(report.exit_code(), EXIT_MATCH, "{}", report.to_text());
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].name, "graph");
        assert_eq!(report.checks[0].outcome, Outcome::Passed);
        assert_eq!(report.action, None);
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn verify_reports_a_graph_a_422_names_as_missing() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        assistant["graph_id"] = json!("other");
        let file = write_export(directory.path(), assistant, json!({}));
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(json!({"graph_id": "other"})))
            .with_status(422)
            .with_body(
                json!({"detail": "body.graph_id: value is not one of 'test_graph'"}).to_string(),
            )
            .create_async()
            .await;

        let report = compare(
            &destination.client,
            &file,
            &target(&record_path_for(&file), Some(RECORDED_ID)),
            true,
        )
        .await
        .unwrap();

        assert_eq!(report.exit_code(), 3, "{}", report.to_text());
        assert_eq!(report.checks[0].name, "graph");
        assert_eq!(report.checks[0].outcome, Outcome::Failed);
        assert!(
            report.checks[0].detail.contains("'other'"),
            "{:?}",
            report.checks
        );
        destination.create.assert_async().await;
    }

    #[tokio::test]
    async fn verify_exits_with_an_error_on_a_422_naming_another_field() {
        let directory = tempfile::tempdir().unwrap();
        let mut assistant = file_assistant();
        assistant["graph_id"] = json!("broken");
        let file = write_export(directory.path(), assistant, json!({}));
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::PartialJson(json!({"graph_id": "broken"})))
            .with_status(422)
            .with_body(json!({"detail": "body.limit: must be positive"}).to_string())
            .create_async()
            .await;

        let result = compare(
            &destination.client,
            &file,
            &target(&record_path_for(&file), Some(RECORDED_ID)),
            true,
        )
        .await;

        assert!(result.is_err(), "{result:?}");
    }

    #[tokio::test]
    async fn verify_reports_a_prompt_tag_the_workspace_lacks() {
        let directory = tempfile::tempdir().unwrap();
        let file = write_export(
            directory.path(),
            file_assistant(),
            json!({ SYSTEM_PROMPT: "support-system:prod" }),
        );
        let mut destination = destination(&served_assistant(), 0).await;
        destination
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(
                    r"^/api/v1/commits/-/support-system/prod(\?.*)?$".to_string(),
                ),
            )
            .with_status(404)
            .with_body(json!({"detail": "Commit not found"}).to_string())
            .create_async()
            .await;

        let report = compare(
            &destination.client,
            &file,
            &target(&record_path_for(&file), Some(RECORDED_ID)),
            true,
        )
        .await
        .unwrap();

        assert_eq!(report.exit_code(), 3, "{}", report.to_text());
        assert_eq!(report.checks[1].name, "prompt");
        assert_eq!(report.checks[1].outcome, Outcome::Failed);
        assert!(report.checks[1].detail.contains("support-system:prod"));
        assert!(report.differences.is_empty(), "{}", report.to_text());
    }

    #[tokio::test]
    async fn verify_requests_the_commit_each_prompt_name_reads_as() {
        for (name, path) in [
            ("support-system", "/api/v1/commits/-/support-system/latest"),
            (
                "my-org/support-system:prod",
                "/api/v1/commits/my-org/support-system/prod",
            ),
            (
                "-/support-system:prod",
                "/api/v1/commits/-/support-system/prod",
            ),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let mut assistant = file_assistant();
            set(&mut assistant, SYSTEM_PROMPT, json!(name));
            set(&mut assistant, "/context/system_prompt", json!(name));
            let file = write_export(
                directory.path(),
                assistant.clone(),
                json!({ SYSTEM_PROMPT: name }),
            );
            let mut served = served_assistant();
            served["config"] = assistant["config"].clone();
            served["context"] = assistant["context"].clone();
            let mut destination = destination(&served, 0).await;
            let commit = destination
                .server
                .mock("GET", mockito::Matcher::Regex(format!(r"^{path}(\?.*)?$")))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(json!({"commit_hash": "abc123", "manifest": {}}).to_string())
                .expect(1)
                .create_async()
                .await;

            let report = compare(
                &destination.client,
                &file,
                &target(&record_path_for(&file), Some(RECORDED_ID)),
                true,
            )
            .await
            .unwrap();

            commit.assert_async().await;
            assert_eq!(
                report.exit_code(),
                EXIT_MATCH,
                "{name}: {}",
                report.to_text()
            );
            assert_eq!(report.checks[1].outcome, Outcome::Passed, "{name}");
        }
    }
}
