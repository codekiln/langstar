//! The report that `compare`, `verify` and `import` print, for prompts and for
//! assistants alike, and the exit status each result gives.
//!
//! The report prints to stdout, in text or as one JSON object, and every
//! progress message goes to stderr, so `langstar -f json prompt compare
//! my-prompt.json` leaves stdout holding the report alone.
//!
//! Before it prints either side of a difference, the report puts `<redacted>`
//! in place of every string the credential check flags, by either rule, whether
//! the difference sits at the string or at an object that holds it, and whether
//! the string came from the destination or from the file. A LangChain secret
//! reference prints unchanged under any key, because it holds a secret's name
//! and never its value.

// Groups 5 through 10 of the export-import change call these from the prompt
// and assistant commands. Until those land, the unit tests are the only callers.
#![allow(dead_code)]

use crate::compare::Difference;
use crate::credential_check;
use crate::error::Result;
use crate::output::OutputFormat;
use serde::Serialize;
use serde_json::{Value, json};

/// The resource matches the file and every check passed.
pub const EXIT_MATCH: i32 = 0;

/// Any error langstar reports. `main` already exits with this.
pub const EXIT_ERROR: i32 = 1;

/// Langstar found a difference or a failed check.
pub const EXIT_DIFFERENCE: i32 = 3;

/// Whether the run found anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The resource matches the file and every check passed.
    Passed,
    /// Langstar found a difference or a failed check.
    Failed,
}

/// How one of verify's checks of the destination came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The destination has what the file needs.
    Passed,
    /// The destination lacks it, which fails the run.
    Failed,
    /// Something to tell the user that leaves the status alone, such as a workspace
    /// with no saved model configuration for the prompt's model class, which
    /// the prompt runs without.
    Note,
}

/// What import did, or what `--dry-run` says it would do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Create,
    Update,
    Unchanged,
}

/// One check of the destination: that the workspace holds a secret, that the
/// deployment serves a graph, that the workspace holds a prompt.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    /// What was checked, such as `secrets` or `graph`.
    pub name: String,
    pub outcome: Outcome,
    /// What to tell the user, such as the name of a missing secret.
    pub detail: String,
}

impl Check {
    pub fn passed(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: Outcome::Passed,
            detail: detail.into(),
        }
    }

    pub fn failed(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: Outcome::Failed,
            detail: detail.into(),
        }
    }

    pub fn note(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: Outcome::Note,
            detail: detail.into(),
        }
    }
}

/// What a compare, verify or import run found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResultReport {
    pub differences: Vec<Difference>,
    pub checks: Vec<Check>,
    /// What import did. Compare and verify leave this out.
    pub action: Option<Action>,
    /// The destination's commit hash or assistant ID, which import reports and
    /// a dry run of a create leaves null.
    pub destination_id: Option<String>,
}

impl ResultReport {
    /// A report of the differences a comparison found.
    pub fn of(differences: Vec<Difference>) -> Self {
        Self {
            differences,
            ..Self::default()
        }
    }

    pub fn with_checks(mut self, checks: Vec<Check>) -> Self {
        self.checks = checks;
        self
    }

    pub fn with_action(mut self, action: Action, destination_id: Option<String>) -> Self {
        self.action = Some(action);
        self.destination_id = destination_id;
        self
    }

    /// Whether the run found a difference or a failed check.
    pub fn status(&self) -> Status {
        let failed_check = self
            .checks
            .iter()
            .any(|check| check.outcome == Outcome::Failed);
        if self.differences.is_empty() && !failed_check {
            Status::Passed
        } else {
            Status::Failed
        }
    }

    /// The status langstar exits with: 0 for a match, 3 for a difference or a
    /// failed check. Any other error exits with 1, and clap keeps 2 for a
    /// mistyped flag.
    pub fn exit_code(&self) -> i32 {
        match self.status() {
            Status::Passed => EXIT_MATCH,
            Status::Failed => EXIT_DIFFERENCE,
        }
    }

    /// The JSON object `-f json` prints, with every flagged string redacted.
    pub fn to_json(&self) -> Value {
        let mut report = json!({
            "status": self.status(),
            "differences": self
                .differences
                .iter()
                .map(redacted_difference)
                .collect::<Vec<_>>(),
            "checks": self.checks,
        });
        if let Some(action) = self.action {
            let members = report.as_object_mut().expect("the report is an object");
            members.insert("action".to_string(), json!(action));
            members.insert("destination_id".to_string(), json!(self.destination_id));
        }
        report
    }

    /// The report a user reads, with every flagged string redacted.
    pub fn to_text(&self) -> String {
        let mut text = String::new();

        if self.differences.is_empty() {
            text.push_str("No differences.\n");
        } else {
            text.push_str("Differences\n\n");
            for difference in &self.differences {
                let redacted = redacted_difference(difference);
                text.push_str(&format!("  {}\n", difference.path));
                text.push_str(&format!("    file:        {}\n", side(&redacted.file)));
                text.push_str(&format!(
                    "    destination: {}\n",
                    side(&redacted.destination)
                ));
            }
        }

        if !self.checks.is_empty() {
            text.push_str("\nChecks\n\n");
            for check in &self.checks {
                let outcome = match check.outcome {
                    Outcome::Passed => "passed",
                    Outcome::Failed => "failed",
                    Outcome::Note => "note  ",
                };
                text.push_str(&format!(
                    "  {}  {}: {}\n",
                    outcome, check.name, check.detail
                ));
            }
        }

        if let Some(action) = self.action {
            let action = match action {
                Action::Create => "create",
                Action::Update => "update",
                Action::Unchanged => "unchanged",
            };
            text.push_str(&format!("\nAction: {}\n", action));
            if let Some(id) = &self.destination_id {
                text.push_str(&format!("Destination: {}\n", id));
            }
        }

        text
    }

    /// Print the report to stdout.
    pub fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&self.to_json())?),
            OutputFormat::Table | OutputFormat::Text => print!("{}", self.to_text()),
        }
        Ok(())
    }
}

/// Print a progress message to stderr, whatever the output format, so it never
/// lands in the JSON a script reads from stdout.
pub fn progress(message: &str) {
    eprintln!("{}", message);
}

fn redacted_difference(difference: &Difference) -> Difference {
    Difference {
        path: difference.path.clone(),
        file: difference
            .file
            .as_ref()
            .map(|value| credential_check::redact_at(&difference.path, value)),
        destination: difference
            .destination
            .as_ref()
            .map(|value| credential_check::redact_at(&difference.path, value)),
    }
}

fn side(value: &Option<Value>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "(absent)".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential_check::REDACTED;
    use serde_json::json;

    fn difference(path: &str, destination: Value) -> Difference {
        Difference {
            path: path.to_string(),
            file: None,
            destination: Some(destination),
        }
    }

    fn report_of(difference: Difference) -> ResultReport {
        ResultReport::of(vec![difference])
    }

    /// Both the text and the `-f json` report, so one assertion covers each.
    fn printed(report: &ResultReport) -> String {
        format!(
            "{}\n{}",
            report.to_text(),
            serde_json::to_string(&report.to_json()).expect("the report serializes")
        )
    }

    fn assert_redacted(report: &ResultReport, secret: &str) {
        let printed = printed(report);
        assert!(
            printed.contains(REDACTED),
            "expected <redacted> in:\n{printed}"
        );
        assert!(
            !printed.contains(secret),
            "the report holds the flagged string:\n{printed}"
        );
        for run in secret.as_bytes().windows(8) {
            let run = std::str::from_utf8(run).expect("the test strings are ASCII");
            assert!(
                !printed.contains(run),
                "the report holds the run {run:?} of the flagged string"
            );
        }
    }

    // The status and the exit status

    #[test]
    fn a_report_with_nothing_in_it_passes_and_exits_with_zero() {
        let report = ResultReport::default();
        assert_eq!(report.status(), Status::Passed);
        assert_eq!(report.exit_code(), EXIT_MATCH);
        assert!(report.to_text().contains("No differences."));
    }

    #[test]
    fn a_difference_fails_and_exits_with_three() {
        let report = report_of(difference("/repo/readme", json!("# two")));
        assert_eq!(report.status(), Status::Failed);
        assert_eq!(report.exit_code(), EXIT_DIFFERENCE);
    }

    #[test]
    fn a_failed_check_fails_and_exits_with_three() {
        let report = ResultReport::default()
            .with_checks(vec![Check::failed("secrets", "OPENAI_API_KEY is missing")]);
        assert_eq!(report.status(), Status::Failed);
        assert_eq!(report.exit_code(), EXIT_DIFFERENCE);
    }

    #[test]
    fn a_note_leaves_the_exit_status_at_zero() {
        let report = ResultReport::default().with_checks(vec![Check::note(
            "model configuration",
            "no saved model configuration matches ChatOpenAI",
        )]);
        assert_eq!(report.status(), Status::Passed);
        assert_eq!(report.exit_code(), EXIT_MATCH);
    }

    // What the JSON report holds

    #[test]
    fn the_json_report_holds_the_status_the_differences_and_the_checks() {
        let report = report_of(difference("/repo/readme", json!("# two")))
            .with_checks(vec![Check::passed("secrets", "every secret is there")]);
        let printed = report.to_json();

        assert_eq!(printed["status"], json!("failed"));
        assert_eq!(printed["differences"][0]["path"], json!("/repo/readme"));
        assert_eq!(printed["differences"][0]["destination"], json!("# two"));
        assert!(printed["differences"][0].get("file").is_none());
        assert_eq!(printed["checks"][0]["name"], json!("secrets"));
        assert!(printed.get("action").is_none());
    }

    #[test]
    fn an_import_report_holds_its_action_and_the_destination_id() {
        let report =
            ResultReport::default().with_action(Action::Create, Some("<commit-hash>".to_string()));
        let printed = report.to_json();

        assert_eq!(printed["action"], json!("create"));
        assert_eq!(printed["destination_id"], json!("<commit-hash>"));
        assert!(report.to_text().contains("Action: create"));
    }

    #[test]
    fn a_dry_run_that_would_create_reports_a_null_destination_id() {
        let report = ResultReport::default().with_action(Action::Create, None);
        assert_eq!(report.to_json()["destination_id"], Value::Null);
    }

    // Redaction

    #[test]
    fn redacts_a_destination_string_under_a_credential_key_name() {
        let secret = "sk-proj-0123456789abcdefghij";
        let report = report_of(difference(
            "/commit/manifest/kwargs/last/kwargs/bound/kwargs/openai_api_key",
            json!(secret),
        ));
        assert_redacted(&report, secret);
    }

    #[test]
    fn redacts_a_string_inside_a_model_the_destination_added() {
        let secret = "sk-proj-0123456789abcdefghij";
        let report = report_of(difference(
            "/commit/manifest/kwargs/last",
            json!({
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": { "openai_api_key": secret }
            }),
        ));
        assert_redacted(&report, secret);
        assert!(
            printed(&report).contains("ChatOpenAI"),
            "the report still says where the difference is"
        );
    }

    #[test]
    fn redacts_an_assistant_setting_under_either_spelling_of_the_key() {
        let secret = "sk-proj-0123456789abcdefghij";
        for key in ["openai_api_key", "OPENAI_API_KEY"] {
            let report = report_of(difference(
                &format!("/assistant/config/configurable/{key}"),
                json!(secret),
            ));
            assert_redacted(&report, secret);
        }
    }

    #[test]
    fn redacts_a_bearer_token_in_a_model_header() {
        let secret = "Bearer 0123456789abcdefghij";
        let report = report_of(difference(
            "/commit/manifest/kwargs/last/kwargs/bound/kwargs/default_headers/Authorization",
            json!(secret),
        ));
        assert_redacted(&report, secret);
    }

    #[test]
    fn redacts_a_token_under_a_neutral_key_in_a_model() {
        let secret = "lsv2_pt_0123456789abcdef0123456789abcdef";
        let report = report_of(difference(
            "/commit/manifest/kwargs/last/kwargs/bound/kwargs/model_kwargs/extra",
            json!(secret),
        ));
        assert_redacted(&report, secret);
    }

    #[test]
    fn redacts_a_connection_string_in_an_assistant_context() {
        let secret = "postgresql://app:s3cret-pw@host/db";
        let report = report_of(difference(
            "/assistant/context/checkpoint_uri",
            json!(secret),
        ));
        assert_redacted(&report, secret);
    }

    #[test]
    fn redacts_the_password_in_an_object_the_destination_added_and_prints_the_user() {
        let secret = "s3cret-pw-for-the-app";
        let report = report_of(difference(
            "/assistant/config/configurable/db",
            json!({ "user": "app", "password": secret }),
        ));
        assert_redacted(&report, secret);

        let printed = printed(&report);
        assert!(
            printed.contains("\"user\":\"app\"") || printed.contains("\\\"user\\\":\\\"app\\\""),
            "expected user to print unchanged in:\n{printed}"
        );
    }

    // Secret references

    #[test]
    fn prints_a_secret_reference_in_a_model_unchanged() {
        let report = report_of(difference(
            "/commit/manifest/kwargs/last",
            json!({
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": {
                    "openai_api_key": { "id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret" }
                }
            }),
        ));
        let printed = printed(&report);

        assert!(printed.contains("OPENAI_API_KEY"), "got:\n{printed}");
        assert!(!printed.contains(REDACTED), "got:\n{printed}");
    }

    #[test]
    fn prints_an_assistant_setting_that_reads_a_secret_unchanged() {
        let report = report_of(difference(
            "/assistant/config/configurable/openai_api_key",
            json!({ "id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret" }),
        ));
        let printed = printed(&report);

        assert!(printed.contains("OPENAI_API_KEY"), "got:\n{printed}");
        assert!(!printed.contains(REDACTED), "got:\n{printed}");
    }
}
