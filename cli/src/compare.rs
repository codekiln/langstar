//! The comparison behind `compare`, `verify` and the read-back that `import`
//! runs after it writes.
//!
//! It reads the destination in the shape of the export file's `repo`, `commit`
//! and `assistant` objects and names each value that differs by its JSON
//! Pointer path. An array is one value: a changed `id` array inside a manifest,
//! `ChatOpenAI` replaced by `ChatAnthropic`, is reported at the array's own
//! path with both arrays.
//!
//! The comparison reads two of the export file's values differently from the
//! rest:
//!
//! - `commit.model_config` and `commit.model_provider`, which LangSmith fills
//!   in itself when import pushes a commit and a destination workspace can fill
//!   in differently from the source.
//! - An assistant's settings, which `config.configurable` and `context` hold
//!   between them. The Agent Server mirrors whichever field it was given into
//!   the other, so the comparison takes the file's settings from whichever
//!   field the file holds and the destination's from whichever the server
//!   returned. Every other key under `config`, such as `config.recursion_limit`,
//!   is compared as it stands.

// Groups 5 through 10 of the export-import change call these from the prompt
// and assistant commands. Until those land, the unit tests are the only callers.
#![allow(dead_code)]

use crate::json_pointer;
use serde::Serialize;
use serde_json::{Map, Value};

/// The pointer to the settings field the comparison names when the file holds
/// its settings under `config.configurable`.
const CONFIGURABLE: &str = "/assistant/config/configurable";

/// The pointer to the settings field the comparison names when `context` holds
/// them.
const CONTEXT: &str = "/assistant/context";

/// One value that differs between the export file and the destination.
///
/// A side that has no value there leaves its field out, so a key only the
/// destination holds reads as a `destination` with no `file`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Difference {
    /// The JSON Pointer path of the value, as RFC 6901 writes it.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Value>,
}

impl Difference {
    fn new(path: impl Into<String>, file: Option<&Value>, destination: Option<&Value>) -> Self {
        Self {
            path: path.into(),
            file: file.cloned(),
            destination: destination.cloned(),
        }
    }
}

/// Compare the file's `repo`, `commit` and `assistant` objects with the
/// destination's, sorted by path.
pub fn compare_export(file: &Value, destination: &Value) -> Vec<Difference> {
    let mut differences = Vec::new();

    compare_object(
        "/repo",
        file.get("repo"),
        destination.get("repo"),
        &mut differences,
    );
    compare_object(
        "/commit",
        without(file.get("commit"), &["model_config", "model_provider"]).as_ref(),
        without(
            destination.get("commit"),
            &["model_config", "model_provider"],
        )
        .as_ref(),
        &mut differences,
    );
    compare_assistant(
        file.get("assistant"),
        destination.get("assistant"),
        &mut differences,
    );

    differences.sort_by(|one, two| one.path.cmp(&two.path));
    differences
}

/// Compare two JSON values, naming each difference by its path below them.
pub fn compare_values(file: &Value, destination: &Value) -> Vec<Difference> {
    compare_values_at("", file, destination)
}

/// Compare two JSON values that sit at `base`, naming each difference by its
/// full path.
pub fn compare_values_at(base: &str, file: &Value, destination: &Value) -> Vec<Difference> {
    let mut differences = Vec::new();
    walk(base.to_string(), file, destination, &mut differences);
    differences.sort_by(|one, two| one.path.cmp(&two.path));
    differences
}

fn walk(path: String, file: &Value, destination: &Value, found: &mut Vec<Difference>) {
    let (Value::Object(file_members), Value::Object(destination_members)) = (file, destination)
    else {
        if file != destination {
            found.push(Difference::new(path, Some(file), Some(destination)));
        }
        return;
    };

    for (key, file_member) in file_members {
        let child = json_pointer::child(&path, key);
        match destination_members.get(key) {
            Some(destination_member) => walk(child, file_member, destination_member, found),
            None => found.push(Difference::new(child, Some(file_member), None)),
        }
    }
    for (key, destination_member) in destination_members {
        if !file_members.contains_key(key) {
            found.push(Difference::new(
                json_pointer::child(&path, key),
                None,
                Some(destination_member),
            ));
        }
    }
}

fn compare_object(
    base: &str,
    file: Option<&Value>,
    destination: Option<&Value>,
    found: &mut Vec<Difference>,
) {
    match (file, destination) {
        (None, None) => {}
        (Some(file), Some(destination)) => walk(base.to_string(), file, destination, found),
        (file, destination) => found.push(Difference::new(base, file, destination)),
    }
}

fn compare_assistant(
    file: Option<&Value>,
    destination: Option<&Value>,
    found: &mut Vec<Difference>,
) {
    let destination_settings = settings_of(destination);

    // A file whose two settings fields disagree was edited in one of them, so
    // each is compared on its own and the edit shows up as a difference.
    if let Some(fields) = conflicting_settings(file) {
        for (settings, field) in fields {
            match destination_settings {
                Some((destination_settings, _)) => {
                    walk(field.to_string(), settings, destination_settings, found)
                }
                None => found.push(Difference::new(field, Some(settings), None)),
            }
        }
        compare_object(
            "/assistant",
            without_settings(file).as_ref(),
            without_settings(destination).as_ref(),
            found,
        );
        return;
    }

    let file_settings = settings_of(file);
    match (file_settings, destination_settings) {
        (None, None) => {}
        (Some((settings, field)), Some((destination_settings, _))) => {
            walk(field.to_string(), settings, destination_settings, found)
        }
        (Some((settings, field)), None) => found.push(Difference::new(field, Some(settings), None)),
        (None, Some((settings, field))) => found.push(Difference::new(field, None, Some(settings))),
    }

    compare_object(
        "/assistant",
        without_settings(file).as_ref(),
        without_settings(destination).as_ref(),
        found,
    );
}

/// The settings an assistant carries, and the field they came from.
///
/// `config.configurable` and `context` hold one value, so the first field that
/// holds anything answers for both.
fn settings_of(assistant: Option<&Value>) -> Option<(&Value, &'static str)> {
    let assistant = assistant?;
    if let Some(configurable) = present(assistant.pointer("/config/configurable")) {
        return Some((configurable, CONFIGURABLE));
    }
    present(assistant.get("context")).map(|context| (context, CONTEXT))
}

/// Both settings fields, when an assistant holds both and they differ.
fn conflicting_settings(assistant: Option<&Value>) -> Option<[(&Value, &'static str); 2]> {
    let assistant = assistant?;
    let configurable = present(assistant.pointer("/config/configurable"))?;
    let context = present(assistant.get("context"))?;
    (configurable != context).then_some([(configurable, CONFIGURABLE), (context, CONTEXT)])
}

/// The assistant with `context` and `config.configurable` taken out, and with
/// `config` taken out too when the settings were all it held.
fn without_settings(assistant: Option<&Value>) -> Option<Value> {
    let assistant = without(assistant, &["context"])?;
    let Some(members) = assistant.as_object() else {
        return Some(assistant);
    };

    let config = without(members.get("config"), &["configurable"]);
    let mut members = members.clone();
    match config {
        Some(config) if config.as_object().is_some_and(Map::is_empty) => {
            members.remove("config");
        }
        Some(config) => {
            members.insert("config".to_string(), config);
        }
        None => {}
    }
    Some(Value::Object(members))
}

/// `value` with each of `keys` taken out, when `value` is an object.
fn without(value: Option<&Value>, keys: &[&str]) -> Option<Value> {
    let value = value?;
    let Some(members) = value.as_object() else {
        return Some(value.clone());
    };
    let mut members = members.clone();
    for key in keys {
        members.remove(*key);
    }
    Some(Value::Object(members))
}

/// A value the side actually carries. A field set to null carries nothing.
fn present(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| !value.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn paths(differences: &[Difference]) -> Vec<&str> {
        differences
            .iter()
            .map(|difference| difference.path.as_str())
            .collect()
    }

    // The JSON comparison

    #[test]
    fn reports_a_changed_value_deep_in_a_nested_object() {
        let differences = compare_values(
            &json!({ "model": { "kwargs": { "temperature": 0.0 } } }),
            &json!({ "model": { "kwargs": { "temperature": 0.7 } } }),
        );
        assert_eq!(
            differences,
            vec![Difference {
                path: "/model/kwargs/temperature".to_string(),
                file: Some(json!(0.0)),
                destination: Some(json!(0.7)),
            }]
        );
    }

    #[test]
    fn reports_a_changed_array_at_the_array_itself() {
        let differences = compare_values(&json!({ "stop": ["a", "b"] }), &json!({ "stop": ["a"] }));
        assert_eq!(
            differences,
            vec![Difference {
                path: "/stop".to_string(),
                file: Some(json!(["a", "b"])),
                destination: Some(json!(["a"])),
            }]
        );
    }

    #[test]
    fn reports_a_key_only_one_side_holds() {
        let differences =
            compare_values(&json!({ "readme": "# my-prompt" }), &json!({ "tags": [] }));
        assert_eq!(
            differences,
            vec![
                Difference {
                    path: "/readme".to_string(),
                    file: Some(json!("# my-prompt")),
                    destination: None,
                },
                Difference {
                    path: "/tags".to_string(),
                    file: None,
                    destination: Some(json!([])),
                },
            ]
        );
    }

    #[test]
    fn escapes_a_key_holding_a_slash_or_a_tilde() {
        let differences = compare_values(
            &json!({ "a/b": 1, "a~b": 1 }),
            &json!({ "a/b": 2, "a~b": 2 }),
        );
        assert_eq!(paths(&differences), vec!["/a~0b", "/a~1b"]);
    }

    #[test]
    fn reports_no_difference_for_equal_values() {
        let value = json!({ "a": [1, 2], "b": { "c": null } });
        assert!(compare_values(&value, &value).is_empty());
    }

    // The export comparison

    #[test]
    fn reports_a_changed_model_class_at_its_id_array() {
        let manifest = |model_class: &str| {
            json!({
                "commit": {
                    "manifest": {
                        "kwargs": {
                            "last": {
                                "id": ["langchain", "chat_models", "openai", model_class]
                            }
                        }
                    }
                }
            })
        };
        let differences = compare_export(&manifest("ChatOpenAI"), &manifest("ChatAnthropic"));

        assert_eq!(
            differences,
            vec![Difference {
                path: "/commit/manifest/kwargs/last/id".to_string(),
                file: Some(json!(["langchain", "chat_models", "openai", "ChatOpenAI"])),
                destination: Some(json!([
                    "langchain",
                    "chat_models",
                    "openai",
                    "ChatAnthropic"
                ])),
            }]
        );
    }

    #[test]
    fn reports_a_match_when_only_the_model_configuration_differs() {
        let file = json!({
            "commit": {
                "manifest": { "lc": 1 },
                "model_config": { "temperature": 0.0 },
                "model_provider": "ChatOpenAI"
            }
        });
        let destination = json!({
            "commit": {
                "manifest": { "lc": 1 },
                "model_config": { "temperature": 0.7, "max_tokens": 256 },
                "model_provider": "ChatAnthropic"
            }
        });
        assert!(compare_export(&file, &destination).is_empty());
    }

    #[test]
    fn reports_a_changed_repo_readme_at_its_path() {
        let differences = compare_export(
            &json!({ "repo": { "readme": "# one" } }),
            &json!({ "repo": { "readme": "# two" } }),
        );
        assert_eq!(paths(&differences), vec!["/repo/readme"]);
    }

    // The settings alias

    /// An assistant export body carrying `extra` beside its name and graph ID.
    fn assistant(extra: Value) -> Value {
        let mut body = json!({ "name": "support-bot", "graph_id": "support" });
        let members = body.as_object_mut().expect("the body is an object");
        for (key, value) in extra.as_object().expect("the extra fields are an object") {
            members.insert(key.clone(), value.clone());
        }
        json!({ "assistant": body })
    }

    #[test]
    fn a_config_only_file_matches_a_destination_that_holds_both_fields() {
        let file = assistant(json!({ "config": { "configurable": { "model": "gpt-4o" } } }));
        let destination = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" } },
            "context": { "model": "gpt-4o" }
        }));
        assert!(compare_export(&file, &destination).is_empty());
    }

    #[test]
    fn a_context_only_file_matches_a_destination_that_holds_both_fields() {
        let file = assistant(json!({ "context": { "model": "gpt-4o" } }));
        let destination = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" } },
            "context": { "model": "gpt-4o" }
        }));
        assert!(compare_export(&file, &destination).is_empty());
    }

    #[test]
    fn a_config_only_file_against_a_context_holding_other_values_is_one_difference() {
        let file = assistant(json!({ "config": { "configurable": { "model": "gpt-4o" } } }));
        let destination = assistant(json!({ "context": { "model": "claude-opus-4" } }));
        let differences = compare_export(&file, &destination);

        assert_eq!(
            differences,
            vec![Difference {
                path: "/assistant/config/configurable/model".to_string(),
                file: Some(json!("gpt-4o")),
                destination: Some(json!("claude-opus-4")),
            }]
        );
    }

    #[test]
    fn a_file_whose_context_was_edited_alone_reports_the_edit() {
        let file = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" } },
            "context": { "model": "claude-opus-4" }
        }));
        let destination = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" } },
            "context": { "model": "gpt-4o" }
        }));
        let differences = compare_export(&file, &destination);

        assert_eq!(
            differences,
            vec![Difference {
                path: "/assistant/context/model".to_string(),
                file: Some(json!("claude-opus-4")),
                destination: Some(json!("gpt-4o")),
            }]
        );
    }

    #[test]
    fn every_other_key_under_config_is_compared_as_it_stands() {
        let file = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" }, "recursion_limit": 25 }
        }));
        let destination = assistant(json!({
            "config": { "configurable": { "model": "gpt-4o" }, "recursion_limit": 10 }
        }));
        let differences = compare_export(&file, &destination);

        assert_eq!(
            differences,
            vec![Difference {
                path: "/assistant/config/recursion_limit".to_string(),
                file: Some(json!(25)),
                destination: Some(json!(10)),
            }]
        );
    }

    #[test]
    fn reports_settings_the_destination_lost() {
        let file = assistant(json!({ "config": { "configurable": { "model": "gpt-4o" } } }));
        let destination = assistant(json!({}));
        let differences = compare_export(&file, &destination);

        assert_eq!(
            differences,
            vec![Difference {
                path: CONFIGURABLE.to_string(),
                file: Some(json!({ "model": "gpt-4o" })),
                destination: None,
            }]
        );
    }

    #[test]
    fn reports_a_changed_assistant_metadata_key() {
        let file = assistant(json!({ "metadata": { "owner": "support" } }));
        let destination = assistant(json!({ "metadata": { "owner": "sales" } }));
        assert_eq!(
            paths(&compare_export(&file, &destination)),
            vec!["/assistant/metadata/owner"]
        );
    }
}
