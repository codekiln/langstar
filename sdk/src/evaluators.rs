//! Evaluator implementations for LangSmith evaluations.
//!
//! This module provides implementations of heuristic evaluators and utilities
//! for LLM-as-judge evaluators.

use crate::evaluations::{EvaluationResult, FeedbackConfig, FeedbackType, LlmJudgeConfig};
use regex::Regex;
use serde_json::Value;

// ============================================================================
// Heuristic Evaluators
// ============================================================================

/// Evaluates exact string match between output and expected value.
///
/// # Arguments
/// * `output` - The actual output to evaluate
/// * `expected` - The expected value to match against
///
/// # Returns
/// * `1.0` if strings match exactly (case-sensitive)
/// * `0.0` if strings don't match
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::exact_match;
///
/// assert_eq!(exact_match("hello", "hello"), 1.0);
/// assert_eq!(exact_match("hello", "Hello"), 0.0);
/// assert_eq!(exact_match("hello", "world"), 0.0);
/// ```
pub fn exact_match(output: &str, expected: &str) -> f64 {
    if output == expected { 1.0 } else { 0.0 }
}

/// Evaluates if output contains expected substring.
///
/// # Arguments
/// * `output` - The actual output to evaluate
/// * `expected` - The substring to search for
///
/// # Returns
/// * `1.0` if output contains expected substring (case-sensitive)
/// * `0.0` if substring not found
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::contains;
///
/// assert_eq!(contains("hello world", "world"), 1.0);
/// assert_eq!(contains("hello world", "World"), 0.0);
/// assert_eq!(contains("hello world", "foo"), 0.0);
/// ```
pub fn contains(output: &str, expected: &str) -> f64 {
    if output.contains(expected) { 1.0 } else { 0.0 }
}

/// Evaluates if output matches a regular expression pattern.
///
/// # Arguments
/// * `output` - The actual output to evaluate
/// * `pattern` - The regex pattern to match against
///
/// # Returns
/// * `Ok(1.0)` if output matches the pattern
/// * `Ok(0.0)` if output doesn't match
/// * `Err(String)` if the regex pattern is invalid
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::regex_match;
///
/// assert_eq!(regex_match("test123", r"^\w+\d+$").unwrap(), 1.0);
/// assert_eq!(regex_match("test", r"^\d+$").unwrap(), 0.0);
/// assert!(regex_match("test", r"[invalid(").is_err());
/// ```
pub fn regex_match(output: &str, pattern: &str) -> Result<f64, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;

    Ok(if re.is_match(output) { 1.0 } else { 0.0 })
}

/// Evaluates if output is valid JSON.
///
/// # Arguments
/// * `output` - The string to validate as JSON
///
/// # Returns
/// * `1.0` if output is valid JSON
/// * `0.0` if output is not valid JSON
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::json_valid;
///
/// assert_eq!(json_valid(r#"{"key": "value"}"#), 1.0);
/// assert_eq!(json_valid(r#"[1, 2, 3]"#), 1.0);
/// assert_eq!(json_valid(r#""simple string""#), 1.0);
/// assert_eq!(json_valid(r#"{invalid json}"#), 0.0);
/// assert_eq!(json_valid("not json at all"), 0.0);
/// ```
pub fn json_valid(output: &str) -> f64 {
    match serde_json::from_str::<Value>(output) {
        Ok(_) => 1.0,
        Err(_) => 0.0,
    }
}

/// Calculates Levenshtein distance between two strings.
///
/// # Arguments
/// * `output` - The first string
/// * `expected` - The second string
///
/// # Returns
/// The Levenshtein distance (number of single-character edits needed)
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::levenshtein_distance;
///
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("hello", "hello"), 0);
/// assert_eq!(levenshtein_distance("", "abc"), 3);
/// ```
#[allow(clippy::needless_range_loop)]
pub fn levenshtein_distance(output: &str, expected: &str) -> usize {
    let output_len = output.chars().count();
    let expected_len = expected.chars().count();

    if output_len == 0 {
        return expected_len;
    }
    if expected_len == 0 {
        return output_len;
    }

    let mut matrix = vec![vec![0; expected_len + 1]; output_len + 1];

    // Initialize first row and column
    for i in 0..=output_len {
        matrix[i][0] = i;
    }
    for j in 0..=expected_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    let output_chars: Vec<char> = output.chars().collect();
    let expected_chars: Vec<char> = expected.chars().collect();

    for i in 1..=output_len {
        for j in 1..=expected_len {
            let cost = if output_chars[i - 1] == expected_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[output_len][expected_len]
}

/// Evaluates string similarity using normalized Levenshtein distance.
///
/// # Arguments
/// * `output` - The actual output to evaluate
/// * `expected` - The expected value to compare against
///
/// # Returns
/// A similarity score between 0.0 and 1.0, where:
/// * `1.0` means strings are identical
/// * `0.0` means strings are completely different
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::string_distance;
///
/// assert_eq!(string_distance("hello", "hello"), 1.0);
/// assert!(string_distance("kitten", "sitting") > 0.5);
/// assert!(string_distance("abc", "xyz") < 0.5);
/// ```
pub fn string_distance(output: &str, expected: &str) -> f64 {
    let max_len = output.chars().count().max(expected.chars().count());

    if max_len == 0 {
        return 1.0; // Both strings are empty
    }

    let distance = levenshtein_distance(output, expected);
    1.0 - (distance as f64 / max_len as f64)
}

// ============================================================================
// LLM-as-Judge Utilities
// ============================================================================

/// Formats a prompt for LLM-as-judge evaluation.
///
/// # Arguments
/// * `config` - LLM judge configuration including rubric
/// * `input` - The input that was provided to the system
/// * `output` - The output generated by the system
/// * `reference` - Optional reference/expected output
///
/// # Returns
/// A formatted prompt string ready to send to an LLM
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::format_judge_prompt;
/// use langstar_sdk::evaluations::{LlmJudgeConfig, ScoreType};
///
/// let config = LlmJudgeConfig {
///     model: "gpt-4".to_string(),
///     provider: None,
///     score_type: ScoreType::Categorical,
///     choices: Some(vec!["Y".to_string(), "N".to_string()]),
///     min: None,
///     max: None,
///     rubric: Some("Is the answer correct?".to_string()),
///     include_reasoning: false,
/// };
///
/// let prompt = format_judge_prompt(&config, "What is 2+2?", "4", Some("4"));
/// assert!(prompt.contains("Is the answer correct?"));
/// assert!(prompt.contains("Input: What is 2+2?"));
/// assert!(prompt.contains("Output: 4"));
/// ```
pub fn format_judge_prompt(
    config: &LlmJudgeConfig,
    input: &str,
    output: &str,
    reference: Option<&str>,
) -> String {
    let mut prompt = String::new();

    // Add rubric/criteria
    if let Some(rubric) = &config.rubric {
        prompt.push_str("Evaluation Criteria:\n");
        prompt.push_str(rubric);
        prompt.push_str("\n\n");
    }

    // Add input
    prompt.push_str("Input: ");
    prompt.push_str(input);
    prompt.push_str("\n\n");

    // Add output to evaluate
    prompt.push_str("Output: ");
    prompt.push_str(output);
    prompt.push_str("\n\n");

    // Add reference if provided
    if let Some(ref_output) = reference {
        prompt.push_str("Reference Output: ");
        prompt.push_str(ref_output);
        prompt.push_str("\n\n");
    }

    // Add scoring instructions
    match config.score_type {
        crate::evaluations::ScoreType::Categorical => {
            if let Some(choices) = &config.choices {
                prompt.push_str(&format!(
                    "Provide your evaluation as one of: {}\n",
                    choices.join(", ")
                ));
            }
        }
        crate::evaluations::ScoreType::Continuous => {
            let min = config.min.unwrap_or(0.0);
            let max = config.max.unwrap_or(1.0);
            prompt.push_str(&format!(
                "Provide your evaluation as a numeric score between {} and {}\n",
                min, max
            ));
        }
    }

    if config.include_reasoning {
        prompt.push_str("\nPlease include your reasoning for this evaluation.");
    }

    prompt
}

/// Converts evaluator result to EvaluationResult type.
///
/// # Arguments
/// * `key` - The metric name
/// * `score` - The numeric score (if applicable)
/// * `comment` - Optional comment or reasoning
///
/// # Returns
/// An EvaluationResult ready to be sent to LangSmith
///
/// # Example
/// ```
/// use langstar_sdk::evaluators::to_evaluation_result;
///
/// let result = to_evaluation_result("exact_match", Some(1.0), Some("Output matches expected".to_string()));
/// assert_eq!(result.key, "exact_match");
/// assert_eq!(result.score, Some(1.0));
/// ```
pub fn to_evaluation_result(
    key: &str,
    score: Option<f64>,
    comment: Option<String>,
) -> EvaluationResult {
    EvaluationResult {
        key: key.to_string(),
        score,
        comment,
        feedback_config: Some(FeedbackConfig {
            feedback_type: FeedbackType::Continuous,
            min: Some(0.0),
            max: Some(1.0),
            categories: None,
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Heuristic Evaluator Tests
    // ========================================================================

    #[test]
    fn test_exact_match_equal() {
        assert_eq!(exact_match("hello", "hello"), 1.0);
        assert_eq!(exact_match("", ""), 1.0);
        assert_eq!(exact_match("Test123", "Test123"), 1.0);
    }

    #[test]
    fn test_exact_match_not_equal() {
        assert_eq!(exact_match("hello", "Hello"), 0.0);
        assert_eq!(exact_match("hello", "world"), 0.0);
        assert_eq!(exact_match("test", "test "), 0.0);
    }

    #[test]
    fn test_contains_found() {
        assert_eq!(contains("hello world", "world"), 1.0);
        assert_eq!(contains("hello world", "hello"), 1.0);
        assert_eq!(contains("hello world", "o w"), 1.0);
    }

    #[test]
    fn test_contains_not_found() {
        assert_eq!(contains("hello world", "World"), 0.0);
        assert_eq!(contains("hello world", "foo"), 0.0);
        assert_eq!(contains("", "test"), 0.0);
    }

    #[test]
    fn test_regex_match_valid() {
        assert_eq!(regex_match("test123", r"^\w+\d+$").unwrap(), 1.0);
        assert_eq!(regex_match("abc", r"^[a-z]+$").unwrap(), 1.0);
        assert_eq!(regex_match("Test123", r"^[A-Z]\w+\d+$").unwrap(), 1.0);
    }

    #[test]
    fn test_regex_match_invalid() {
        assert_eq!(regex_match("test", r"^\d+$").unwrap(), 0.0);
        assert_eq!(regex_match("123", r"^[a-z]+$").unwrap(), 0.0);
    }

    #[test]
    fn test_regex_match_invalid_pattern() {
        let result = regex_match("test", r"[invalid(");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid regex pattern"));
    }

    #[test]
    fn test_json_valid_true() {
        assert_eq!(json_valid(r#"{"key": "value"}"#), 1.0);
        assert_eq!(json_valid(r#"[1, 2, 3]"#), 1.0);
        assert_eq!(json_valid(r#""simple string""#), 1.0);
        assert_eq!(json_valid("123"), 1.0);
        assert_eq!(json_valid("true"), 1.0);
        assert_eq!(json_valid("null"), 1.0);
    }

    #[test]
    fn test_json_valid_false() {
        assert_eq!(json_valid(r#"{invalid json}"#), 0.0);
        assert_eq!(json_valid("not json at all"), 0.0);
        assert_eq!(json_valid(r#"{"unclosed": "#), 0.0);
        assert_eq!(json_valid(""), 0.0);
    }

    #[test]
    fn test_levenshtein_distance_equal() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_distance_substitution() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn test_levenshtein_distance_insertion_deletion() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("hello", "hell"), 1);
    }

    #[test]
    fn test_string_distance_identical() {
        assert_eq!(string_distance("hello", "hello"), 1.0);
        assert_eq!(string_distance("", ""), 1.0);
    }

    #[test]
    fn test_string_distance_similar() {
        let score = string_distance("kitten", "sitting");
        assert!(score > 0.5 && score < 1.0);
    }

    #[test]
    fn test_string_distance_different() {
        let score = string_distance("abc", "xyz");
        assert_eq!(score, 0.0);
    }

    // ========================================================================
    // LLM Judge Utilities Tests
    // ========================================================================

    #[test]
    fn test_format_judge_prompt_with_rubric() {
        let config = LlmJudgeConfig {
            model: "gpt-4".to_string(),
            provider: None,
            score_type: crate::evaluations::ScoreType::Categorical,
            choices: Some(vec!["Y".to_string(), "N".to_string()]),
            min: None,
            max: None,
            rubric: Some("Is the answer correct?".to_string()),
            include_reasoning: false,
        };

        let prompt = format_judge_prompt(&config, "What is 2+2?", "4", Some("4"));

        assert!(prompt.contains("Is the answer correct?"));
        assert!(prompt.contains("Input: What is 2+2?"));
        assert!(prompt.contains("Output: 4"));
        assert!(prompt.contains("Reference Output: 4"));
        assert!(prompt.contains("Provide your evaluation as one of: Y, N"));
    }

    #[test]
    fn test_format_judge_prompt_continuous_score() {
        let config = LlmJudgeConfig {
            model: "gpt-4".to_string(),
            provider: None,
            score_type: crate::evaluations::ScoreType::Continuous,
            choices: None,
            min: Some(0.0),
            max: Some(10.0),
            rubric: Some("Rate the quality".to_string()),
            include_reasoning: true,
        };

        let prompt = format_judge_prompt(&config, "test input", "test output", None);

        assert!(prompt.contains("Rate the quality"));
        assert!(prompt.contains("between 0 and 10"));
        assert!(prompt.contains("include your reasoning"));
    }

    #[test]
    fn test_format_judge_prompt_without_reference() {
        let config = LlmJudgeConfig {
            model: "gpt-4".to_string(),
            provider: None,
            score_type: crate::evaluations::ScoreType::Categorical,
            choices: Some(vec!["Pass".to_string(), "Fail".to_string()]),
            min: None,
            max: None,
            rubric: None,
            include_reasoning: false,
        };

        let prompt = format_judge_prompt(&config, "input", "output", None);

        assert!(prompt.contains("Input: input"));
        assert!(prompt.contains("Output: output"));
        assert!(!prompt.contains("Reference Output"));
    }

    #[test]
    fn test_to_evaluation_result() {
        let result =
            to_evaluation_result("test_metric", Some(0.85), Some("Good result".to_string()));

        assert_eq!(result.key, "test_metric");
        assert_eq!(result.score, Some(0.85));
        assert_eq!(result.comment, Some("Good result".to_string()));
        assert!(result.feedback_config.is_some());

        let config = result.feedback_config.unwrap();
        assert_eq!(config.feedback_type, FeedbackType::Continuous);
        assert_eq!(config.min, Some(0.0));
        assert_eq!(config.max, Some(1.0));
    }
}
