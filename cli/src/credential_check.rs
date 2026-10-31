//! The credential check that export, compare, verify and import share.
//!
//! The check walks every string under a JSON value and flags the ones that read
//! like a credential. Export refuses to write a file that holds one, and the
//! report prints `<redacted>` in its place. The check skips a LangChain secret
//! reference, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, because
//! that holds a secret's name and never its value, and it skips the empty
//! string.
//!
//! # The word list
//!
//! A string is flagged when the name of its key, or the name of the key of the
//! array it sits in, holds one of these words: `secret`, `token`, `password`,
//! `passwd`, `pwd`, `credential`, `credentials`, `apikey`, `authorization`,
//! `cookie`. It is also flagged when the key holds `key` or `keys` right after
//! one of `api`, `access`, `private`, `secret`, `signing`, `account`.
//!
//! A key name splits into words at `_`, `-`, `.`, a space and each change from a
//! lower-case to an upper-case letter, and case is ignored. So `openai_api_key`,
//! `OPENAI_API_KEY`, `x-api-key`, `Authorization`, `privateKey` and `api_keys`
//! are flagged, while `max_tokens`, `cache_key`, `author` and `keywords` pass.
//!
//! # The value shapes
//!
//! Under any key, a string is flagged when it holds one of these. HTTP reads
//! the `Bearer` and `Basic` scheme names in any case, so the check does too.
//!
//! - a **bearer token**: `Bearer ` and at least 16 token characters after it;
//! - a **Basic value with a user and password**: `Basic ` and a base64 payload
//!   that decodes to printable text with a colon in it, however short, so
//!   `Basic dXNlcjpwYXNz` is flagged for the `user:pass` it holds;
//! - a **PEM private key**: `-----BEGIN` and `PRIVATE KEY-----`;
//! - a **URL with a password**: `scheme://user:password@host` or
//!   `scheme://:password@host`, which covers database and queue connection
//!   strings;
//! - a **connection string with a password**: a `Password=`, `Pwd=`,
//!   `AccountKey=`, `SharedAccessKey=` or `SharedAccessSignature=` with a value
//!   after it;
//! - a **JSON Web Token**: three base64url parts, the first starting `eyJ`;
//! - a **provider token**: one of the published prefixes `sk-` (OpenAI and
//!   Anthropic), `lsv2_` (LangSmith), `ghp_`, `gho_`, `ghs_`, `github_pat_`
//!   (GitHub), `xoxb-`, `xoxp-`, `xoxa-` (Slack), `AKIA` and `ASIA` followed by
//!   16 upper-case letters or digits (AWS), `AIza` (Google) and `hf_` (Hugging
//!   Face), each with enough characters after it to rule out an ordinary word.

// Groups 5 through 10 of the export-import change call these from the prompt
// and assistant commands. Until those land, the unit tests are the only callers.
#![allow(dead_code)]

use crate::json_pointer;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;

/// What the report prints in place of a flagged string.
pub const REDACTED: &str = "<redacted>";

/// The words that flag a string by the name of its key.
const CREDENTIAL_WORDS: &[&str] = &[
    "secret",
    "token",
    "password",
    "passwd",
    "pwd",
    "credential",
    "credentials",
    "apikey",
    "authorization",
    "cookie",
];

/// The words that flag a `key` or `keys` that follows one of them.
const WORDS_BEFORE_KEY: &[&str] = &["api", "access", "private", "secret", "signing", "account"];

/// The published token prefixes, each flagged with at least `MIN_TOKEN_TAIL`
/// characters after it.
const TOKEN_PREFIXES: &[&str] = &[
    "sk-",
    "lsv2_",
    "ghp_",
    "gho_",
    "ghs_",
    "github_pat_",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "AIza",
    "hf_",
];

/// The AWS access key prefixes, each followed by 16 upper-case letters or digits.
const AWS_PREFIXES: &[&str] = &["AKIA", "ASIA"];

/// How many characters a prefixed token needs after its prefix. Short enough to
/// catch a real token, long enough to leave `sk-` on its own alone.
const MIN_TOKEN_TAIL: usize = 16;

/// The keys of a connection string. A value after one of them flags the string.
const CONNECTION_STRING_KEYS: &[&str] = &[
    "password=",
    "pwd=",
    "accountkey=",
    "sharedaccesskey=",
    "sharedaccesssignature=",
];

/// The rule that flagged a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// The name of the string's key, or of the array's key, holds a credential word.
    KeyName,
    /// `Bearer ` and a token.
    BearerToken,
    /// `Basic ` and a base64 payload holding a user and a password.
    BasicValue,
    /// `-----BEGIN` and `PRIVATE KEY-----`.
    PemPrivateKey,
    /// `scheme://user:password@host`.
    UrlWithPassword,
    /// A `Password=`, `Pwd=`, `AccountKey=`, `SharedAccessKey=` or
    /// `SharedAccessSignature=` with a value after it.
    ConnectionStringWithPassword,
    /// Three base64url parts, the first starting `eyJ`.
    JsonWebToken,
    /// A provider's published token prefix.
    ProviderToken,
}

impl Rule {
    /// How the refusal and the report name this rule.
    pub fn name(self) -> &'static str {
        match self {
            Rule::KeyName => "key name",
            Rule::BearerToken => "bearer token",
            Rule::BasicValue => "Basic value with a user and password",
            Rule::PemPrivateKey => "PEM private key",
            Rule::UrlWithPassword => "URL with a password",
            Rule::ConnectionStringWithPassword => "connection string with a password",
            Rule::JsonWebToken => "JSON Web Token",
            Rule::ProviderToken => "provider token",
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One flagged string: where it sits and what flagged it. The string itself is
/// left out, so nothing that carries this leaks the credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flagged {
    /// The JSON Pointer path of the string.
    pub path: String,
    /// The rule that flagged it.
    pub rule: Rule,
}

/// Walk every string under `value` and return the flagged ones, in the order the
/// walk met them.
pub fn find_credentials(value: &Value) -> Vec<Flagged> {
    let mut found = Vec::new();
    walk(value, None, String::new(), &mut found);
    found
}

/// What export tells a user whose prompt or assistant holds a credential.
///
/// The message names each path and the rule that flagged it, and holds no part
/// of any flagged string.
pub fn refusal_message(flagged: &[Flagged]) -> String {
    let mut message = String::from("This would write a credential to the export file:\n");
    for item in flagged {
        message.push_str(&format!("  {} ({})\n", item.path, item.rule));
    }
    message.push_str(
        "Move each one into the workspace's secrets or the deployment's environment \
         variables. To write a string that holds no credential, read it first and name \
         its path with --not-a-secret.",
    );
    message
}

/// The names of every LangChain secret reference under `value`, sorted and
/// without repeats.
///
/// A prompt's manifest names the API key its model reads rather than carrying
/// its value, as `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, and
/// export writes these names to `requires.secrets`.
pub fn secret_names(value: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_secret_names(value, &mut names);
    names
}

/// Replace every flagged string under `value` with `<redacted>`, leaving a
/// secret reference as it is.
///
/// `path` is where `value` sits, so that a difference at a flagged string is
/// redacted too: the key-name rule reads the key from the end of the path.
pub fn redact_at(path: &str, value: &Value) -> Value {
    redact(value, json_pointer::key_of(path).as_deref())
}

/// Whether `value` is a LangChain secret reference.
///
/// A reference holds `lc` set to 1, `type` and `id` and nothing else. This
/// returns false for an object with extra keys or with an `lc` other than 1.
/// The credential check and the redaction then look inside it as they would any
/// object, and find a password beside those keys or inside `lc`.
pub fn is_secret_reference(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == 3
        && object.get("lc").and_then(Value::as_u64) == Some(1)
        && object.get("type").and_then(Value::as_str) == Some("secret")
        && object
            .get("id")
            .and_then(Value::as_array)
            .is_some_and(|id| id.iter().all(Value::is_string))
}

fn walk(value: &Value, key: Option<&str>, path: String, found: &mut Vec<Flagged>) {
    if is_secret_reference(value) {
        return;
    }
    match value {
        Value::String(text) => {
            if let Some(rule) = flag(text, key) {
                found.push(Flagged { path, rule });
            }
        }
        Value::Array(items) => {
            for (position, item) in items.iter().enumerate() {
                walk(item, key, json_pointer::index(&path, position), found);
            }
        }
        Value::Object(members) => {
            for (member_key, member) in members {
                walk(
                    member,
                    Some(member_key),
                    json_pointer::child(&path, member_key),
                    found,
                );
            }
        }
        _ => {}
    }
}

fn redact(value: &Value, key: Option<&str>) -> Value {
    if is_secret_reference(value) {
        return value.clone();
    }
    match value {
        Value::String(text) if flag(text, key).is_some() => Value::String(REDACTED.to_string()),
        Value::Array(items) => Value::Array(items.iter().map(|item| redact(item, key)).collect()),
        Value::Object(members) => Value::Object(
            members
                .iter()
                .map(|(member_key, member)| (member_key.clone(), redact(member, Some(member_key))))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn collect_secret_names(value: &Value, names: &mut BTreeSet<String>) {
    if is_secret_reference(value) {
        if let Some(id) = value.get("id").and_then(Value::as_array) {
            names.extend(id.iter().filter_map(Value::as_str).map(str::to_string));
        }
        return;
    }
    match value {
        Value::Array(items) => items
            .iter()
            .for_each(|item| collect_secret_names(item, names)),
        Value::Object(members) => members
            .values()
            .for_each(|member| collect_secret_names(member, names)),
        _ => {}
    }
}

/// The rule that flags `text` under `key`, or `None` when nothing does.
fn flag(text: &str, key: Option<&str>) -> Option<Rule> {
    if text.is_empty() {
        return None;
    }
    if key.is_some_and(key_name_flags) {
        return Some(Rule::KeyName);
    }
    value_shape(text)
}

fn key_name_flags(key: &str) -> bool {
    let words = key_words(key);
    words.iter().enumerate().any(|(position, word)| {
        CREDENTIAL_WORDS.contains(&word.as_str())
            || ((word == "key" || word == "keys")
                && position > 0
                && WORDS_BEFORE_KEY.contains(&words[position - 1].as_str()))
    })
}

/// Split a key name into lower-case words at `_`, `-`, `.`, a space and each
/// change from a lower-case to an upper-case letter.
fn key_words(key: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut previous_was_lower = false;
    for character in key.chars() {
        if matches!(character, '_' | '-' | '.' | ' ') {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            previous_was_lower = false;
            continue;
        }
        if character.is_uppercase() && previous_was_lower && !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
        previous_was_lower = character.is_lowercase();
        word.extend(character.to_lowercase());
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn value_shape(text: &str) -> Option<Rule> {
    if bearer_token(text) {
        return Some(Rule::BearerToken);
    }
    if basic_value(text) {
        return Some(Rule::BasicValue);
    }
    if pem_private_key(text) {
        return Some(Rule::PemPrivateKey);
    }
    if url_with_password(text) {
        return Some(Rule::UrlWithPassword);
    }
    if connection_string_with_password(text) {
        return Some(Rule::ConnectionStringWithPassword);
    }
    if json_web_token(text) {
        return Some(Rule::JsonWebToken);
    }
    if provider_token(text) {
        return Some(Rule::ProviderToken);
    }
    None
}

fn is_token_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '-' | '_' | '.' | '~' | '+' | '/' | '=')
}

/// The runs of letters, digits, `-` and `_` in `text`. A provider token is one
/// whole run whether it stands alone, follows `=` in an assignment or a URL
/// query, or ends a sentence.
fn provider_token_candidates(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| {
        !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    })
    .filter(|candidate| !candidate.is_empty())
}

/// Each run of base64url characters, dots and `=` padding that starts at an
/// `eyJ`, with a dot that ends a sentence trimmed off. Starting at `eyJ` finds
/// a JSON Web Token after `=` in an assignment or a URL query.
///
/// An `eyJ` inside a run already read starts no new run, so text such as
/// `eyJeyJeyJ…` takes one pass rather than a pass to the end for each `eyJ`.
fn json_web_token_candidates(text: &str) -> impl Iterator<Item = &str> {
    let mut read_to = 0;
    text.match_indices("eyJ").filter_map(move |(at, _)| {
        if at < read_to {
            return None;
        }
        let from = &text[at..];
        let end = from
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '=')))
            .unwrap_or(from.len());
        read_to = at + end;
        Some(from[..end].trim_end_matches('.'))
    })
}

fn bearer_token(text: &str) -> bool {
    auth_payloads(text, "Bearer ")
        .any(|after| after.chars().take_while(|c| is_token_character(*c)).count() >= MIN_TOKEN_TAIL)
}

fn basic_value(text: &str) -> bool {
    auth_payloads(text, "Basic ").any(|after| {
        let payload: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
            .collect();
        decodes_to_user_and_password(&payload)
    })
}

/// Whether `payload` decodes to `user:password`. The bytes are read without a
/// character set, because a client may encode the pair in Latin-1 rather than
/// UTF-8, so a `:` and no ASCII control byte is enough.
fn decodes_to_user_and_password(payload: &str) -> bool {
    if payload.is_empty() {
        return false;
    }
    let Ok(bytes) = BASE64.decode(payload) else {
        return false;
    };
    bytes.contains(&b':') && !bytes.iter().any(u8::is_ascii_control)
}

fn pem_private_key(text: &str) -> bool {
    text.contains("-----BEGIN") && text.contains("PRIVATE KEY-----")
}

fn url_with_password(text: &str) -> bool {
    occurrences(text, "://").any(|after| {
        let authority_end = after.find(['/', '?', '#']).unwrap_or(after.len());
        let authority = &after[..authority_end];
        let Some(at) = authority.rfind('@') else {
            return false;
        };
        let userinfo = &authority[..at];
        userinfo
            .find(':')
            .is_some_and(|colon| !userinfo[colon + 1..].is_empty())
    })
}

fn connection_string_with_password(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    CONNECTION_STRING_KEYS.iter().any(|key| {
        // Stop at the first character that is not a space, so text that repeats a
        // key with no `;`, as in `pwd=pwd=pwd=`, takes one pass rather than
        // a pass to the end for each key.
        occurrences(&lowered, key).any(|after| {
            after
                .chars()
                .take_while(|c| *c != ';')
                .any(|c| !c.is_whitespace())
        })
    })
}

fn json_web_token(text: &str) -> bool {
    json_web_token_candidates(text).any(|candidate| {
        let parts: Vec<&str> = candidate.split('.').collect();
        parts.len() == 3
            && parts.iter().all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '='))
            })
    })
}

fn provider_token(text: &str) -> bool {
    provider_token_candidates(text).any(|token| {
        TOKEN_PREFIXES.iter().any(|prefix| {
            token
                .strip_prefix(prefix)
                .is_some_and(|tail| tail.len() >= MIN_TOKEN_TAIL)
        }) || AWS_PREFIXES.iter().any(|prefix| {
            token.strip_prefix(prefix).is_some_and(|tail| {
                tail.len() >= 16
                    && tail[..16]
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            })
        })
    })
}

/// The text after each HTTP authentication scheme in `text`, in any ASCII
/// case, past the spaces that separate the scheme from its payload. HTTP
/// allows more than one space there, as in `Basic  dXNlcjpwYXNz`.
fn auth_payloads<'a>(text: &'a str, scheme: &'static str) -> impl Iterator<Item = &'a str> {
    occurrences_ignoring_case(text, scheme).map(|after| after.trim_start_matches(' '))
}

/// Each place `needle` appears in `haystack` in any ASCII case, as the text
/// that follows it. Lowercasing ASCII keeps every byte where it was, so the
/// positions found in the lowercased copy slice the original.
fn occurrences_ignoring_case<'a>(
    haystack: &'a str,
    needle: &'static str,
) -> impl Iterator<Item = &'a str> {
    let positions: Vec<usize> = haystack
        .to_ascii_lowercase()
        .match_indices(&needle.to_ascii_lowercase())
        .map(|(at, _)| at)
        .collect();
    positions
        .into_iter()
        .map(move |at| &haystack[at + needle.len()..])
}

/// Each place `needle` appears in `haystack`, as the text that follows it.
fn occurrences<'a>(haystack: &'a str, needle: &'a str) -> impl Iterator<Item = &'a str> {
    haystack
        .match_indices(needle)
        .map(move |(at, _)| &haystack[at + needle.len()..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The `prompt_with_model` manifest from the LangSmith Python SDK's
    /// integration tests, which `references/langsmith-sdk.md` links. Its chat
    /// model reads its API key through a secret reference.
    fn prompt_with_model() -> Value {
        json!({
            "id": ["langsmith", "playground", "PromptPlayground"],
            "lc": 1,
            "type": "constructor",
            "kwargs": {
                "last": {
                    "id": ["langchain", "schema", "runnable", "RunnableBinding"],
                    "lc": 1,
                    "type": "constructor",
                    "kwargs": {
                        "bound": {
                            "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                            "lc": 1,
                            "type": "constructor",
                            "kwargs": {
                                "openai_api_key": {
                                    "id": ["OPENAI_API_KEY"],
                                    "lc": 1,
                                    "type": "secret"
                                }
                            }
                        },
                        "kwargs": {}
                    }
                },
                "first": {
                    "id": ["langchain", "prompts", "chat", "ChatPromptTemplate"],
                    "lc": 1,
                    "type": "constructor",
                    "kwargs": {
                        "messages": [{
                            "id": ["langchain", "prompts", "chat", "SystemMessagePromptTemplate"],
                            "lc": 1,
                            "type": "constructor",
                            "kwargs": {
                                "prompt": {
                                    "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                                    "lc": 1,
                                    "type": "constructor",
                                    "kwargs": {
                                        "template": "You are a chatbot.",
                                        "input_variables": [],
                                        "template_format": "f-string"
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        })
    }

    fn rule_under(key: &str, text: &str) -> Option<Rule> {
        let found = find_credentials(&json!({ key: text }));
        found.first().map(|flagged| flagged.rule)
    }

    fn flagged_under_note(text: &str) -> Option<Rule> {
        rule_under("note", text)
    }

    // The secret reference walk

    #[test]
    fn lists_the_secret_a_manifest_reads() {
        let names = secret_names(&prompt_with_model());
        assert_eq!(
            names.into_iter().collect::<Vec<_>>(),
            vec!["OPENAI_API_KEY".to_string()]
        );
    }

    #[test]
    fn lists_no_secret_for_a_manifest_without_one() {
        let manifest = json!({
            "id": ["langsmith", "playground", "PromptPlayground"],
            "lc": 1,
            "type": "constructor",
            "kwargs": { "first": {}, "last": {} }
        });
        assert!(secret_names(&manifest).is_empty());
    }

    // The key-name rule

    #[test]
    fn flags_a_string_under_a_credential_key_name() {
        for key in [
            "openai_api_key",
            "OPENAI_API_KEY",
            "x-api-key",
            "Authorization",
            "Cookie",
            "access_token",
            "refresh_token",
            "client_secret",
            "db_password",
            "privateKey",
            "signingKey",
            "credentials",
        ] {
            assert_eq!(
                rule_under(key, "a-plain-looking-value"),
                Some(Rule::KeyName),
                "expected the key name rule to flag {key}"
            );
        }
    }

    #[test]
    fn flags_a_string_in_an_array_under_a_credential_key_name() {
        let found = find_credentials(&json!({ "api_keys": ["a-plain-looking-value"] }));
        assert_eq!(
            found,
            vec![Flagged {
                path: "/api_keys/0".to_string(),
                rule: Rule::KeyName,
            }]
        );
    }

    #[test]
    fn passes_a_string_under_an_ordinary_key_name() {
        for key in ["max_tokens", "cache_key", "author", "keywords"] {
            assert_eq!(
                rule_under(key, "a-plain-looking-value"),
                None,
                "expected {key} to pass"
            );
        }
    }

    #[test]
    fn passes_a_number_and_a_boolean_under_a_credential_key_name() {
        assert!(find_credentials(&json!({ "api_key": 4096 })).is_empty());
        assert!(find_credentials(&json!({ "password": true })).is_empty());
    }

    // The value shapes

    #[test]
    fn flags_a_bearer_token_under_a_neutral_key() {
        assert_eq!(
            flagged_under_note("Bearer abcdefghijklmnopqrst"),
            Some(Rule::BearerToken)
        );
    }

    #[test]
    fn flags_a_basic_value_under_a_neutral_key() {
        // "admin:password1", twenty base64 characters.
        assert_eq!(
            flagged_under_note("Basic YWRtaW46cGFzc3dvcmQx"),
            Some(Rule::BasicValue)
        );
    }

    #[test]
    fn flags_a_bearer_token_or_basic_value_whatever_the_scheme_case() {
        for text in [
            "bearer abcdefghijklmnopqrst",
            "BEARER abcdefghijklmnopqrst",
            "basic dXNlcjpwYXNz",
            "BASIC dXNlcjpwYXNz",
        ] {
            assert!(
                flagged_under_note(text).is_some(),
                "{text:?} should be flagged"
            );
        }
    }

    #[test]
    fn flags_a_bearer_token_or_basic_value_after_several_spaces() {
        for text in ["Bearer  abcdefghijklmnopqrst", "Basic   dXNlcjpwYXNz"] {
            assert!(
                flagged_under_note(text).is_some(),
                "{text:?} should be flagged"
            );
        }
    }

    #[test]
    fn checks_long_repeated_text_in_one_pass() {
        let start = std::time::Instant::now();
        for text in ["pwd=".repeat(200_000), "eyJ".repeat(200_000)] {
            flag(&text, Some("note"));
        }
        assert!(
            start.elapsed() < std::time::Duration::from_secs(5),
            "the check took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn flags_a_short_basic_value_that_holds_a_user_and_password() {
        // Twelve base64 characters holding `user:pass`.
        assert_eq!(
            flagged_under_note("Basic dXNlcjpwYXNz"),
            Some(Rule::BasicValue)
        );
    }

    #[test]
    fn flags_a_basic_value_whose_password_is_latin_1() {
        use base64::Engine;
        // `user:pässwort` with ä as the single Latin-1 byte 0xE4, which is not UTF-8.
        let text = format!("Basic {}", BASE64.encode(b"user:p\xe4sswort"));
        assert_eq!(flagged_under_note(&text), Some(Rule::BasicValue));
        assert_eq!(
            redact_at("/note", &Value::String(text)),
            Value::String(REDACTED.to_string())
        );
    }

    #[test]
    fn flags_each_pem_private_key_under_a_neutral_key() {
        for label in ["PRIVATE KEY", "RSA PRIVATE KEY", "OPENSSH PRIVATE KEY"] {
            let block = format!("-----BEGIN {label}-----\nMIIEpAIBAAKCAQEA\n-----END {label}-----");
            assert_eq!(
                flagged_under_note(&block),
                Some(Rule::PemPrivateKey),
                "expected a {label} block to be flagged"
            );
        }
    }

    #[test]
    fn flags_a_url_with_a_password_under_a_neutral_key() {
        assert_eq!(
            flagged_under_note("postgresql://app:s3cret-pw@host/db"),
            Some(Rule::UrlWithPassword)
        );
        assert_eq!(
            flagged_under_note("redis://:s3cret-pw@host:6379"),
            Some(Rule::UrlWithPassword)
        );
    }

    #[test]
    fn flags_a_connection_string_with_a_password_under_a_neutral_key() {
        assert_eq!(
            flagged_under_note("Server=db;User Id=app;Password=s3cret-pw"),
            Some(Rule::ConnectionStringWithPassword)
        );
        assert_eq!(
            flagged_under_note(
                "DefaultEndpointsProtocol=https;AccountName=store;AccountKey=YWJjZGVmZ2hpamtsbW5vcA==;"
            ),
            Some(Rule::ConnectionStringWithPassword)
        );
    }

    #[test]
    fn flags_a_json_web_token_under_a_neutral_key() {
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(flagged_under_note(token), Some(Rule::JsonWebToken));
    }

    #[test]
    fn flags_a_token_with_each_published_prefix_under_a_neutral_key() {
        let tails = "abcdefghijklmnopqrstuvwxyz0123456789";
        for prefix in TOKEN_PREFIXES {
            let token = format!("{prefix}{tails}");
            assert_eq!(
                flagged_under_note(&token),
                Some(Rule::ProviderToken),
                "expected a token starting {prefix} to be flagged"
            );
        }
        for prefix in AWS_PREFIXES {
            let token = format!("{prefix}ABCDEFGH12345678");
            assert_eq!(
                flagged_under_note(&token),
                Some(Rule::ProviderToken),
                "expected a token starting {prefix} to be flagged"
            );
        }
    }

    #[test]
    fn flags_a_provider_token_in_an_assignment_a_url_query_and_a_sentence() {
        let github = format!("ghp_{}", "a1".repeat(18));
        for text in [
            "x=sk-proj-0123456789abcdefghij".to_string(),
            format!("https://example.test/?api_key={github}"),
            format!("https://example.test/?a=1&token={github}&b=2"),
            "aws=AKIAABCDEFGH12345678".to_string(),
            "the key is sk-proj-0123456789abcdefghij.".to_string(),
        ] {
            assert_eq!(
                flagged_under_note(&text),
                Some(Rule::ProviderToken),
                "expected {text:?} to be flagged"
            );
        }
    }

    #[test]
    fn flags_a_json_web_token_in_an_assignment_a_url_query_and_a_sentence() {
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        for text in [
            format!("The token is {token}."),
            format!("token={token}"),
            format!("https://example.test/callback?id_token={token}&state=x"),
        ] {
            assert_eq!(
                flagged_under_note(&text),
                Some(Rule::JsonWebToken),
                "expected {text:?} to be flagged"
            );
        }
    }

    // Ordinary text

    #[test]
    fn passes_ordinary_text_under_a_neutral_key() {
        for text in [
            "Basic instructions follow.",
            "Bearer in mind",
            "https://example.com/path",
            "postgresql://host/db",
            "sk-",
            "-----BEGIN PUBLIC KEY-----",
            // A prefix inside a longer word is not a token.
            "risk-0123456789abcdefghijklmnop",
        ] {
            assert_eq!(flagged_under_note(text), None, "expected {text:?} to pass");
        }
    }

    // What the check skips

    #[test]
    fn skips_a_secret_reference_under_a_credential_key_name() {
        let value = json!({
            "openai_api_key": { "id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret" }
        });
        assert!(find_credentials(&value).is_empty());
    }

    #[test]
    fn walks_an_object_that_holds_a_secret_reference_and_more() {
        let value = json!({
            "note": {
                "id": ["OPENAI_API_KEY"],
                "lc": 1,
                "type": "secret",
                "password": "plain-looking-password"
            }
        });
        assert_eq!(
            find_credentials(&value),
            vec![Flagged {
                path: "/note/password".to_string(),
                rule: Rule::KeyName,
            }]
        );
        assert!(secret_names(&value).is_empty());
    }

    #[test]
    fn skips_an_empty_string_under_a_credential_key_name() {
        assert!(find_credentials(&json!({ "password": "" })).is_empty());
    }

    #[test]
    fn escapes_a_flagged_key_holding_a_slash_or_a_tilde() {
        let found = find_credentials(&json!({
            "a/b": { "api_key": "a-plain-looking-value" },
            "c~d": { "api_key": "a-plain-looking-value" }
        }));
        let paths: Vec<&str> = found.iter().map(|flagged| flagged.path.as_str()).collect();
        assert_eq!(paths, vec!["/a~1b/api_key", "/c~0d/api_key"]);
    }

    // The refusal message

    #[test]
    fn the_refusal_message_holds_the_path_and_the_rule_and_no_part_of_the_string() {
        let secret = "lsv2_pt_0123456789abcdef0123456789abcdef";
        let found = find_credentials(&json!({ "commit": { "note": secret } }));
        let message = refusal_message(&found);

        assert!(message.contains("/commit/note"));
        assert!(message.contains("provider token"));
        assert!(!message.contains(secret));
        for run in secret.as_bytes().windows(8) {
            let run = std::str::from_utf8(run).expect("the token is ASCII");
            assert!(
                !message.contains(run),
                "the message holds the run {run:?} of the flagged string"
            );
        }
    }

    // Redaction

    #[test]
    fn redacts_a_string_flagged_by_the_key_at_the_end_of_the_path() {
        let redacted = redact_at(
            "/assistant/config/configurable/openai_api_key",
            &json!("sk-0123456789abcdef"),
        );
        assert_eq!(redacted, json!(REDACTED));
    }

    #[test]
    fn redacts_a_flagged_string_inside_an_object_and_leaves_the_rest() {
        let redacted = redact_at(
            "/assistant/config/configurable/db",
            &json!({ "user": "app", "password": "s3cret-pw" }),
        );
        assert_eq!(redacted, json!({ "user": "app", "password": REDACTED }));
    }

    #[test]
    fn walks_a_secret_reference_whose_lc_is_not_1() {
        let value = json!({
            "openai": {
                "id": ["OPENAI_API_KEY"],
                "lc": { "password": "plain-looking-password" },
                "type": "secret"
            }
        });
        assert_eq!(
            find_credentials(&value),
            vec![Flagged {
                path: "/openai/lc/password".to_string(),
                rule: Rule::KeyName,
            }]
        );
        assert_eq!(
            redact_at("", &value)["openai"]["lc"]["password"],
            json!(REDACTED)
        );
    }

    #[test]
    fn redacts_a_password_beside_the_members_of_a_secret_reference() {
        let redacted = redact_at(
            "/assistant/config/configurable/openai",
            &json!({
                "id": ["OPENAI_API_KEY"],
                "lc": 1,
                "type": "secret",
                "password": "plain-looking-password"
            }),
        );
        assert_eq!(
            redacted,
            json!({
                "id": ["OPENAI_API_KEY"],
                "lc": 1,
                "type": "secret",
                "password": REDACTED
            })
        );
    }

    #[test]
    fn leaves_a_secret_reference_unchanged() {
        let reference = json!({ "id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret" });
        let redacted = redact_at("/assistant/config/configurable/openai_api_key", &reference);
        assert_eq!(redacted, reference);
    }
}
