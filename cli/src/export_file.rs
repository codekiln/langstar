//! The export file a prompt or an assistant moves through, and the record of
//! where langstar has imported it.
//!
//! Export writes the file, a reviewer reads it in Git, and import reads it in
//! the destination environment later. The file holds no export time and its
//! keys are sorted at every level, as are `commit.tags`, `repo.tags` and
//! `requires.secrets`, so exporting an unchanged resource leaves Git with
//! nothing to commit.
//!
//! Import writes the IDs it created to a record beside the export file,
//! `<file stem>.destinations.json`, keyed by destination workspace ID for a
//! prompt and by deployment ID for an assistant. Import replaces that record by
//! writing a temporary file in the same directory and renaming it over the old
//! one, so a langstar that stops in the middle of a write leaves the earlier
//! record whole, its pending entries included.

// Groups 5 through 10 of the export-import change call these from the prompt
// and assistant commands. Until those land, the unit tests are the only callers.
#![allow(dead_code)]

use crate::error::{CliError, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The only export file format langstar reads and writes.
pub const FORMAT_VERSION: u32 = 1;

/// Which resource an export file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Prompt,
    Assistant,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Prompt => "prompt",
            Kind::Assistant => "assistant",
        }
    }
}

/// One prompt commit, with everything import needs to recreate it elsewhere.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptExport {
    pub kind: Kind,
    pub format_version: u32,
    pub source: PromptSource,
    pub repo: PromptRepo,
    pub commit: PromptCommit,
    pub requires: PromptRequires,
}

/// Where the prompt came from. Compare never reads these, so a copy in another
/// workspace is no difference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptSource {
    pub workspace_id: String,
    pub handle: String,
    pub commit_hash: String,
}

/// The prompt's repo details, which import sets with `--update`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptRepo {
    pub handle: String,
    pub description: Option<String>,
    pub readme: Option<String>,
    pub tags: BTreeSet<String>,
    pub is_public: bool,
}

/// The commit itself. `manifest` keeps the JSON LangSmith returned, because the
/// typed view cannot hold a model binding, placeholders, several message types
/// or partial variables.
///
/// LangSmith fills `model_config` and `model_provider` in itself when import
/// pushes a commit, so the file records what the source held and the comparison
/// skips them. A commit saved without a model has neither.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptCommit {
    pub description: Option<String>,
    pub manifest: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_config: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,
    pub tags: BTreeSet<String>,
}

/// What the destination workspace needs before the prompt can run there.
///
/// `model_class` is null for a prompt saved without a model, and verify then
/// looks for no saved model configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptRequires {
    pub secrets: BTreeSet<String>,
    pub model_class: Option<String>,
}

/// One assistant, with everything import needs to recreate it on another
/// deployment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantExport {
    pub kind: Kind,
    pub format_version: u32,
    pub source: AssistantSource,
    pub assistant: AssistantBody,
    pub requires: AssistantRequires,
}

/// Where the assistant came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantSource {
    pub deployment_id: String,
    pub assistant_id: String,
    pub version: i32,
}

/// The assistant itself. `config` and `context` hold one settings value, which
/// the Agent Server mirrors from whichever field it was given, so export writes
/// both as the server returned them and the comparison reads them once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantBody {
    pub name: String,
    pub description: Option<String>,
    pub graph_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    pub metadata: Value,
}

/// The prompts the assistant refers to, each as the JSON Pointer the user named
/// with `--prompt-ref` and the prompt name that pointer held at export, such as
/// `support-system:prod`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantRequires {
    pub prompts: BTreeMap<String, String>,
}

/// Read a prompt export file.
pub fn read_prompt_export(path: &Path) -> Result<PromptExport> {
    parse_prompt_export(&read_to_string(path)?)
}

/// Read an assistant export file.
pub fn read_assistant_export(path: &Path) -> Result<AssistantExport> {
    parse_assistant_export(&read_to_string(path)?)
}

/// Read a prompt export file's text.
pub fn parse_prompt_export(text: &str) -> Result<PromptExport> {
    check_header(text, Kind::Prompt, "export file")?;
    parse_body(text, Kind::Prompt)
}

/// Read an assistant export file's text.
pub fn parse_assistant_export(text: &str) -> Result<AssistantExport> {
    check_header(text, Kind::Assistant, "export file")?;
    parse_body(text, Kind::Assistant)
}

/// Read the whole file as `T`. serde_json's message for a field of the wrong
/// type quotes the value, which may be a credential, so the error names only
/// the line and column.
fn parse_body<T: DeserializeOwned>(text: &str, kind: Kind) -> Result<T> {
    serde_json::from_str(text).map_err(|error| {
        CliError::Config(format!(
            "This {} export file does not match the format at line {}, column {}.",
            kind.name(),
            error.line(),
            error.column()
        ))
    })
}

/// Serialize as JSON with the keys sorted at every level, and a trailing
/// newline, so two exports of one commit write the same file.
pub fn to_sorted_json<T: Serialize>(value: &T) -> Result<String> {
    let sorted = sort_keys(serde_json::to_value(value)?);
    Ok(format!("{}\n", serde_json::to_string_pretty(&sorted)?))
}

/// Where langstar keeps the record of what it imported from `export_path`.
pub fn record_path_for(export_path: &Path) -> PathBuf {
    export_path.with_extension("destinations.json")
}

/// What import created in each destination workspace, keyed by workspace ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptDestinations {
    pub kind: Kind,
    pub format_version: u32,
    pub destinations: BTreeMap<String, PromptDestination>,
}

/// The prompt one import created or updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptDestination {
    pub handle: String,
    pub commit_hash: String,
}

/// What import created on each destination deployment, keyed by deployment ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssistantDestinations {
    pub kind: Kind,
    pub format_version: u32,
    pub destinations: BTreeMap<String, AssistantDestination>,
}

/// The assistant one import created.
///
/// Import chooses the ID itself and writes it as `pending` before it sends the
/// create request, so an import that stops in between leaves the next one the
/// same ID to send again.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssistantDestination {
    pub assistant_id: String,
    pub state: DestinationState,
}

/// How far import got with one destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DestinationState {
    /// Import wrote the ID and has not seen the create succeed.
    Pending,
    /// The destination holds the assistant with this ID.
    Created,
}

impl Default for PromptDestinations {
    fn default() -> Self {
        Self {
            kind: Kind::Prompt,
            format_version: FORMAT_VERSION,
            destinations: BTreeMap::new(),
        }
    }
}

impl Default for AssistantDestinations {
    fn default() -> Self {
        Self {
            kind: Kind::Assistant,
            format_version: FORMAT_VERSION,
            destinations: BTreeMap::new(),
        }
    }
}

/// A record of what import created, for prompts or for assistants.
pub trait Record: DeserializeOwned + Default {
    /// The kind of resource whose destinations the record holds.
    const KIND: Kind;
}

impl Record for PromptDestinations {
    const KIND: Kind = Kind::Prompt;
}

impl Record for AssistantDestinations {
    const KIND: Kind = Kind::Assistant;
}

/// Read the record at `path`, or an empty one when no file is there yet.
///
/// Only a missing file reads as empty. A record langstar cannot read, such as
/// one in a directory the user may not list, is an error, so import never
/// starts from an empty record and loses the IDs the old one held.
///
/// A record of another format version or another kind, or one with a field
/// langstar does not know, is refused too: import rewrites the whole record,
/// and would drop what it could not read.
pub fn read_record<T: Record>(path: &Path) -> Result<T> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            check_header(&text, T::KIND, "import record")?;
            serde_json::from_str(&text).map_err(|error| {
                // A record holds only names and IDs, so serde_json's message,
                // which names an unknown field, is safe to print.
                CliError::Config(format!(
                    "The import record {} does not match the format: {}",
                    path.display(),
                    error
                ))
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(source) => Err(CliError::Config(format!(
            "Could not read {}: {}",
            path.display(),
            source
        ))),
    }
}

/// Replace the record at `path` with `record`.
pub fn write_record<T: Serialize>(path: &Path, record: &T) -> Result<()> {
    let staged = stage_record(path, record)?;
    rename_record(&staged, path)
}

/// Write the new record to a temporary file beside `path` and return that file's
/// path. Until [`rename_record`] renames it, the record at `path` still holds
/// what the last finished write left there.
pub fn stage_record<T: Serialize>(path: &Path, record: &T) -> Result<PathBuf> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "destinations.json".to_string());
    let staged = directory.join(format!("{}.{}.tmp", name, std::process::id()));
    // Remove whatever is at the staged path, such as a file left by a langstar run
    // that stopped mid-write, or a symlink to another file. `create_new` then fails
    // if anything appears at the path before it creates the file, so the write
    // never goes through a symlink.
    match std::fs::remove_file(&staged) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)?;
    std::io::Write::write_all(&mut file, to_sorted_json(record)?.as_bytes())?;
    Ok(staged)
}

/// Rename a staged record over the record it replaces, in one step.
pub fn rename_record(staged: &Path, path: &Path) -> Result<()> {
    Ok(std::fs::rename(staged, path)?)
}

fn read_to_string(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| {
        CliError::Config(format!("Could not read {}: {}", path.display(), source))
    })
}

/// Reject a file of another format version or another kind before the command
/// reads anything else from it.
fn check_header(text: &str, expected: Kind, file: &str) -> Result<()> {
    let header: Value = serde_json::from_str(text)?;

    let format_version = header
        .get("format_version")
        .ok_or_else(|| CliError::Config(format!("This {file} has no format_version.")))?;
    if format_version.as_u64() != Some(FORMAT_VERSION as u64) {
        return Err(CliError::Config(format!(
            "This {}'s format_version is {}, and langstar reads format_version {}.",
            file,
            shown(format_version),
            FORMAT_VERSION
        )));
    }

    let kind = header
        .get("kind")
        .ok_or_else(|| CliError::Config(format!("This {file} has no kind.")))?;
    if kind.as_str() != Some(expected.name()) {
        return Err(CliError::Config(format!(
            "This {}'s kind is {}, and the {} commands read a file whose kind is {}.",
            file,
            shown(kind),
            expected.name(),
            expected.name()
        )));
    }

    Ok(())
}

/// A header value as an error message may print it: a number, `prompt` or
/// `assistant` as it is, and anything else by its JSON type alone, so a
/// credential pasted into the header never reaches the terminal.
fn shown(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::String(text)
            if [Kind::Prompt, Kind::Assistant]
                .iter()
                .any(|kind| kind.name() == text) =>
        {
            text.clone()
        }
        Value::String(_) => "a string".to_string(),
        Value::Null => "null".to_string(),
        Value::Bool(_) => "a boolean".to_string(),
        Value::Array(_) => "an array".to_string(),
        Value::Object(_) => "an object".to_string(),
    }
}

fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(members) => {
            let mut keys: Vec<String> = members.keys().cloned().collect();
            keys.sort();
            let mut members = members;
            let mut sorted = Map::with_capacity(keys.len());
            for key in keys {
                let member = members
                    .remove(&key)
                    .expect("the key came from this object's own keys");
                sorted.insert(key, sort_keys(member));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn prompt_export(repo_tags: &[&str], commit_tags: &[&str], secrets: &[&str]) -> PromptExport {
        PromptExport {
            kind: Kind::Prompt,
            format_version: FORMAT_VERSION,
            source: PromptSource {
                workspace_id: "<source-workspace-id>".to_string(),
                handle: "my-prompt".to_string(),
                commit_hash: "<hash>".to_string(),
            },
            repo: PromptRepo {
                handle: "my-prompt".to_string(),
                description: Some("Answers a support question.".to_string()),
                readme: Some("# my-prompt".to_string()),
                tags: repo_tags.iter().map(|tag| tag.to_string()).collect(),
                is_public: false,
            },
            commit: PromptCommit {
                description: Some("Raise the temperature.".to_string()),
                manifest: json!({ "lc": 1, "type": "constructor", "kwargs": {} }),
                model_config: None,
                model_provider: Some("ChatOpenAI".to_string()),
                tags: commit_tags.iter().map(|tag| tag.to_string()).collect(),
            },
            requires: PromptRequires {
                secrets: secrets.iter().map(|name| name.to_string()).collect(),
                model_class: Some("ChatOpenAI".to_string()),
            },
        }
    }

    // A file that changes only when the prompt does

    #[test]
    fn sorts_the_keys_at_every_level_whatever_order_they_arrived_in() {
        let mut outer_one = Map::new();
        outer_one.insert("zeta".to_string(), json!(1));
        let mut inner_one = Map::new();
        inner_one.insert("yankee".to_string(), json!(2));
        inner_one.insert("alpha".to_string(), json!(3));
        outer_one.insert("inner".to_string(), Value::Object(inner_one));

        let mut inner_two = Map::new();
        inner_two.insert("alpha".to_string(), json!(3));
        inner_two.insert("yankee".to_string(), json!(2));
        let mut outer_two = Map::new();
        outer_two.insert("inner".to_string(), Value::Object(inner_two));
        outer_two.insert("zeta".to_string(), json!(1));

        let one = to_sorted_json(&Value::Object(outer_one)).expect("a map serializes");
        let two = to_sorted_json(&Value::Object(outer_two)).expect("a map serializes");

        assert_eq!(one, two);
        assert!(
            one.find("\"inner\"") < one.find("\"zeta\""),
            "expected the outer keys sorted, got {one}"
        );
        assert!(
            one.find("\"alpha\"") < one.find("\"yankee\""),
            "expected the inner keys sorted, got {one}"
        );
    }

    #[test]
    fn writes_the_same_text_for_tags_and_secrets_given_in_any_order() {
        let one = prompt_export(
            &["support", "demo"],
            &["prod", "dev"],
            &["OPENAI_API_KEY", "ANTHROPIC_API_KEY"],
        );
        let two = prompt_export(
            &["demo", "support"],
            &["dev", "prod"],
            &["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
        );

        assert_eq!(
            to_sorted_json(&one).expect("an export serializes"),
            to_sorted_json(&two).expect("an export serializes")
        );
    }

    #[test]
    fn leaves_model_class_null_for_a_prompt_without_a_model() {
        let mut export = prompt_export(&[], &[], &[]);
        export.requires.model_class = None;
        export.commit.model_provider = None;

        let text = to_sorted_json(&export).expect("an export serializes");
        let written: Value = serde_json::from_str(&text).expect("the file is JSON");

        assert_eq!(written["requires"]["model_class"], Value::Null);
        assert!(written["commit"].get("model_provider").is_none());
    }

    // Reading an export file

    #[test]
    fn reads_a_prompt_export_back() {
        let export = prompt_export(&["support"], &["prod"], &["OPENAI_API_KEY"]);
        let text = to_sorted_json(&export).expect("an export serializes");
        assert_eq!(parse_prompt_export(&text).expect("the file reads"), export);
    }

    #[test]
    fn refuses_a_file_from_a_newer_format() {
        let text = r#"{"kind": "prompt", "format_version": 2}"#;
        let error = parse_prompt_export(text).expect_err("a newer format is refused");
        let message = error.to_string();
        assert!(message.contains("format_version"), "got {message}");
        assert!(message.contains('2'), "got {message}");
    }

    #[test]
    fn refuses_an_assistant_file_given_to_a_prompt_command() {
        let text = r#"{"kind": "assistant", "format_version": 1}"#;
        let error = parse_prompt_export(text).expect_err("the other kind is refused");
        let message = error.to_string();
        assert!(message.contains("kind"), "got {message}");
        assert!(message.contains("assistant"), "got {message}");
    }

    #[test]
    fn refuses_a_prompt_file_given_to_an_assistant_command() {
        let text = r#"{"kind": "prompt", "format_version": 1}"#;
        let error = parse_assistant_export(text).expect_err("the other kind is refused");
        let message = error.to_string();
        assert!(message.contains("kind"), "got {message}");
        assert!(message.contains("prompt"), "got {message}");
    }

    #[test]
    fn a_header_error_leaves_out_a_string_it_does_not_know() {
        let secret = "sk-abcdefghijklmnopqrstuvwxyz";
        for text in [
            format!(r#"{{"kind": "prompt", "format_version": "{secret}"}}"#),
            format!(r#"{{"kind": "{secret}", "format_version": 1}}"#),
        ] {
            let message = parse_prompt_export(&text)
                .expect_err("the header is refused")
                .to_string();
            assert!(!message.contains(secret), "got {message}");
            assert!(message.contains("a string"), "got {message}");
        }
    }

    #[test]
    fn a_field_of_the_wrong_type_is_refused_without_its_value() {
        let secret = "sk-abcdefghijklmnopqrstuvwxyz";
        let text = format!(
            r#"{{"kind": "assistant", "format_version": 1, "source": {{"deployment_id": "d", "assistant_id": "a", "version": "{secret}"}}}}"#
        );
        let message = parse_assistant_export(&text)
            .expect_err("a string version is refused")
            .to_string();
        assert!(!message.contains(secret), "got {message}");
        assert!(message.contains("line 1"), "got {message}");
    }

    // The destination record

    #[test]
    fn names_the_record_beside_the_export_file() {
        assert_eq!(
            record_path_for(Path::new("prompts/my-prompt.json")),
            PathBuf::from("prompts/my-prompt.destinations.json")
        );
    }

    #[test]
    fn reads_an_empty_record_when_no_file_is_there_yet() {
        let directory = TempDir::new().expect("a temporary directory");
        let record: PromptDestinations =
            read_record(&directory.path().join("my-prompt.destinations.json"))
                .expect("an absent record reads as empty");
        assert_eq!(record, PromptDestinations::default());
    }

    #[cfg(unix)]
    #[test]
    fn a_record_langstar_cannot_read_is_an_error_not_an_empty_record() {
        use std::os::unix::fs::PermissionsExt;
        let directory = TempDir::new().expect("a temporary directory");
        let locked = directory.path().join("locked");
        std::fs::create_dir(&locked).expect("the directory is created");
        let path = locked.join("my-prompt.destinations.json");
        std::fs::write(&path, "{}").expect("the record writes");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
            .expect("the directory is locked");
        // root lists any directory, and then the record is readable after all.
        let privileged = std::fs::read_dir(&locked).is_ok();
        let result: Result<PromptDestinations> = read_record(&path);
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))
            .expect("the directory is unlocked");
        if privileged {
            return;
        }
        let message = result
            .expect_err("an unreadable record is refused")
            .to_string();
        assert!(message.contains("Could not read"), "got {message}");
    }

    #[test]
    fn refuses_a_record_of_another_version_or_kind_or_with_an_unknown_field() {
        let directory = TempDir::new().expect("a temporary directory");
        let path = directory.path().join("my-prompt.destinations.json");
        for (text, expected) in [
            (
                r#"{"kind": "prompt", "format_version": 2, "destinations": {}}"#,
                "format_version",
            ),
            (
                r#"{"kind": "assistant", "format_version": 1, "destinations": {}}"#,
                "kind",
            ),
            (
                r#"{"kind": "prompt", "format_version": 1, "destinations": {}, "notes": "x"}"#,
                "notes",
            ),
            (
                r#"{"kind": "prompt", "format_version": 1, "destinations": {"w": {"handle": "h", "commit_hash": "c", "url": "u"}}}"#,
                "url",
            ),
        ] {
            std::fs::write(&path, text).expect("the record writes");
            let message = read_record::<PromptDestinations>(&path)
                .expect_err("the record is refused")
                .to_string();
            assert!(message.contains(expected), "got {message}");
        }
    }

    #[test]
    fn updating_one_destination_leaves_the_others_unchanged() {
        let directory = TempDir::new().expect("a temporary directory");
        let path = directory.path().join("my-prompt.destinations.json");

        let mut record = PromptDestinations::default();
        record.destinations.insert(
            "<first-workspace-id>".to_string(),
            PromptDestination {
                handle: "my-prompt".to_string(),
                commit_hash: "<first-hash>".to_string(),
            },
        );
        record.destinations.insert(
            "<second-workspace-id>".to_string(),
            PromptDestination {
                handle: "my-prompt".to_string(),
                commit_hash: "<second-hash>".to_string(),
            },
        );
        write_record(&path, &record).expect("the record writes");

        let mut read_back: PromptDestinations = read_record(&path).expect("the record reads");
        read_back
            .destinations
            .get_mut("<second-workspace-id>")
            .expect("the second destination is in the record")
            .commit_hash = "<newer-hash>".to_string();
        write_record(&path, &read_back).expect("the record writes");

        let after: PromptDestinations = read_record(&path).expect("the record reads");
        assert_eq!(
            after.destinations["<first-workspace-id>"].commit_hash,
            "<first-hash>"
        );
        assert_eq!(
            after.destinations["<second-workspace-id>"].commit_hash,
            "<newer-hash>"
        );
    }

    #[test]
    fn a_write_that_stops_before_the_rename_leaves_the_earlier_record_whole() {
        let directory = TempDir::new().expect("a temporary directory");
        let path = directory.path().join("support-bot.destinations.json");

        let mut earlier = AssistantDestinations::default();
        earlier.destinations.insert(
            "<deployment-id>".to_string(),
            AssistantDestination {
                assistant_id: "<assistant-id>".to_string(),
                state: DestinationState::Pending,
            },
        );
        write_record(&path, &earlier).expect("the record writes");

        // Write the replacement to its temporary file and stop there, the way a
        // langstar that dies mid-write does.
        let mut replacement = AssistantDestinations::default();
        replacement.destinations.insert(
            "<other-deployment-id>".to_string(),
            AssistantDestination {
                assistant_id: "<other-assistant-id>".to_string(),
                state: DestinationState::Created,
            },
        );
        let staged = stage_record(&path, &replacement).expect("the replacement stages");
        assert!(staged.exists(), "the staged file is beside the record");

        let after: AssistantDestinations = read_record(&path).expect("the record reads");
        assert_eq!(after, earlier);
        assert_eq!(
            after.destinations["<deployment-id>"].state,
            DestinationState::Pending
        );
    }

    #[cfg(unix)]
    #[test]
    fn staging_replaces_a_symlink_at_the_staged_path_without_writing_through_it() {
        let directory = TempDir::new().expect("a temporary directory");
        let path = directory.path().join("support-bot.destinations.json");
        let other = directory.path().join("other-file");
        std::fs::write(&other, "left alone").expect("the other file writes");
        let planted = directory.path().join(format!(
            "support-bot.destinations.json.{}.tmp",
            std::process::id()
        ));
        std::os::unix::fs::symlink(&other, &planted).expect("the symlink is created");

        let staged =
            stage_record(&path, &AssistantDestinations::default()).expect("the record stages");

        assert_eq!(staged, planted);
        assert!(
            !std::fs::symlink_metadata(&staged)
                .expect("the staged file exists")
                .file_type()
                .is_symlink(),
            "the staged file is a plain file, not the planted symlink"
        );
        assert_eq!(
            std::fs::read_to_string(&other).expect("the other file reads"),
            "left alone"
        );
    }
}
