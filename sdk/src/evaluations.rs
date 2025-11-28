//! Evaluation types for LangSmith evaluations API.
//!
//! This module provides types for managing evaluations and feedback in LangSmith.
//! Evaluations can be heuristic (deterministic, zero-cost) or LLM-based (model-judged).
//!
//! # API Reference
//!
//! - Feedback endpoints: `/api/v1/feedback`
//! - Run rules endpoints: `/api/v1/runs/rules`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::evaluations::{FeedbackCreate, FeedbackType, FeedbackConfig};
//!
//! let config = FeedbackConfig {
//!     feedback_type: FeedbackType::Continuous,
//!     min: Some(0.0),
//!     max: Some(1.0),
//!     categories: None,
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// ============================================================================
// Feedback Types
// ============================================================================

/// Feedback type enum.
///
/// Determines how feedback values are interpreted and validated.
///
/// # API Reference
///
/// OpenAPI enum: `["continuous", "categorical", "freeform"]`
///
/// # Feedback Types Comparison
///
/// | Type | Score Field | Value Field | Use Case |
/// |------|-------------|-------------|----------|
/// | **Continuous** | `float` in [min, max] | Optional string | Numeric scores (0-1, 1-10, etc.) |
/// | **Categorical** | Optional numeric | `str` from enum | Y/N, Pass/Fail, A/B/C ratings |
/// | **Freeform** | None | `str` | Comments, corrections, explanations |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    /// Continuous numeric feedback with min/max bounds
    Continuous,
    /// Categorical feedback with predefined choices
    Categorical,
    /// Freeform text feedback
    Freeform,
}

/// Specific value and label pair for categorical feedback.
///
/// # API Reference
///
/// Maps to `FeedbackCategory` in OpenAPI spec.
///
/// # Example
///
/// ```
/// use langstar_sdk::evaluations::FeedbackCategory;
///
/// let category = FeedbackCategory {
///     value: 1.0,
///     label: Some("Excellent".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackCategory {
    /// Numeric value for this category (e.g., 0.0, 0.5, 1.0)
    pub value: f64,

    /// Human-readable label (e.g., "Poor", "Good", "Excellent")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Feedback configuration schema.
///
/// Represents how a feedback value ought to be interpreted.
///
/// # API Reference
///
/// Maps to `FeedbackConfig` in OpenAPI spec.
///
/// # Examples
///
/// ```
/// use langstar_sdk::evaluations::{FeedbackConfig, FeedbackType, FeedbackCategory};
///
/// // Continuous feedback (0.0 to 1.0)
/// let continuous = FeedbackConfig {
///     feedback_type: FeedbackType::Continuous,
///     min: Some(0.0),
///     max: Some(1.0),
///     categories: None,
/// };
///
/// // Categorical feedback (Y/N)
/// let categorical = FeedbackConfig {
///     feedback_type: FeedbackType::Categorical,
///     min: None,
///     max: None,
///     categories: Some(vec![
///         FeedbackCategory { value: 1.0, label: Some("Y".to_string()) },
///         FeedbackCategory { value: 0.0, label: Some("N".to_string()) },
///     ]),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Type of feedback (continuous, categorical, or freeform)
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,

    /// Minimum value for continuous feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value for continuous feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Valid categories for categorical feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<FeedbackCategory>>,
}

/// Feedback source type enum.
///
/// Indicates where the feedback originated.
///
/// # API Reference
///
/// Maps to feedback source discriminator types in OpenAPI spec.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSourceType {
    /// Feedback from the LangSmith web app
    App,
    /// Feedback from API calls
    Api,
    /// Feedback from LLM models (LLM-as-judge)
    Model,
    /// Feedback from automated evaluation rules
    AutoEval,
}

/// Feedback source information.
///
/// # API Reference
///
/// Maps to `FeedbackSource` schemas in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSource {
    /// Type of feedback source
    #[serde(rename = "type")]
    pub source_type: FeedbackSourceType,

    /// Additional metadata about the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

// ============================================================================
// Feedback CRUD Types
// ============================================================================

/// Request to create feedback for a run.
///
/// # API Reference
///
/// Request body for `POST /feedback`
///
/// # Example
///
/// ```no_run
/// use langstar_sdk::evaluations::{FeedbackCreate, FeedbackType, FeedbackConfig};
/// use uuid::Uuid;
///
/// let request = FeedbackCreate {
///     key: "accuracy".to_string(),
///     run_id: Some(Uuid::new_v4()),
///     score: Some(0.95),
///     value: None,
///     comment: Some("Correct answer".to_string()),
///     feedback_config: Some(FeedbackConfig {
///         feedback_type: FeedbackType::Continuous,
///         min: Some(0.0),
///         max: Some(1.0),
///         categories: None,
///     }),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedbackCreate {
    /// Metric name or key (required, max 180 chars)
    pub key: String,

    /// Numeric score (for continuous feedback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// String or categorical value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,

    /// Optional explanation or comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Correction data (what the correct output should be)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction: Option<Value>,

    /// Target run ID for the feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,

    /// Target session/project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,

    /// Target trace ID (alternative to run_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,

    /// Optional client-provided feedback ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Feedback source information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_source: Option<FeedbackSource>,

    /// Feedback configuration (type and bounds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_config: Option<FeedbackConfig>,

    /// Feedback group ID (for grouping related feedback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_group_id: Option<Uuid>,

    /// Comparative experiment ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparative_experiment_id: Option<Uuid>,

    /// Custom creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Custom modification timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,

    /// Whether this feedback represents an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,
}

/// Feedback schema (response type).
///
/// Represents feedback as returned by the API.
///
/// # API Reference
///
/// Maps to `FeedbackSchema` in OpenAPI spec.
/// Returned by `GET /feedback`, `GET /feedback/{id}`, `POST /feedback`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// Unique identifier for the feedback
    pub id: Uuid,

    /// Metric name or key
    pub key: String,

    /// Numeric score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// String or categorical value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,

    /// Optional explanation or comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Correction data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction: Option<Value>,

    /// Target run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,

    /// Target session/project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,

    /// Target trace ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,

    /// When the feedback was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// When the feedback was last modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,

    /// Run start time (for queries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    /// Feedback source information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_source: Option<FeedbackSource>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,

    /// Feedback thread ID (for grouping conversations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_thread_id: Option<String>,

    /// Feedback group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_group_id: Option<Uuid>,

    /// Comparative experiment ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparative_experiment_id: Option<Uuid>,
}

/// Request to update existing feedback.
///
/// # API Reference
///
/// Request body for `PATCH /feedback/{id}`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedbackUpdate {
    /// New score value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// New categorical or string value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,

    /// New comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// New correction data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction: Option<Value>,

    /// New feedback configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_config: Option<FeedbackConfig>,
}

// ============================================================================
// Evaluation Result Types
// ============================================================================

/// Evaluation result from an evaluator.
///
/// Represents the output of a single evaluator run.
///
/// # API Reference
///
/// Based on Python SDK `EvaluationResult` type (evaluation/evaluator.py:71-116)
///
/// # Example
///
/// ```
/// use langstar_sdk::evaluations::{EvaluationResult, FeedbackConfig, FeedbackType};
///
/// let result = EvaluationResult {
///     key: "exact_match".to_string(),
///     score: Some(1.0),
///     value: None,
///     comment: Some("Output matches expected".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvaluationResult {
    /// The metric name or label for this evaluation
    pub key: String,

    /// Numeric score (for continuous metrics)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// Categorical or string value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,

    /// Explanation or comment about the evaluation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// What the correct value should be
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction: Option<Value>,

    /// Additional info about the evaluator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluator_info: Option<Value>,

    /// Configuration used for this feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_config: Option<FeedbackConfig>,

    /// ID of the evaluator's trace (for LLM-as-judge)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<Uuid>,

    /// ID of the run being evaluated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_run_id: Option<Uuid>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

// ============================================================================
// Evaluator Configuration Types
// ============================================================================

/// Heuristic evaluator types.
///
/// Deterministic, zero-cost evaluators that don't require LLM calls.
///
/// # API Reference
///
/// Based on Python SDK heuristic evaluators (evaluation/integrations/_langchain.py)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeuristicEvaluator {
    /// Exact string match
    ExactMatch,
    /// Substring contains check
    Contains,
    /// Regular expression match
    RegexMatch,
    /// JSON validity check
    JsonValid,
    /// String distance metrics (Levenshtein, etc.)
    StringDistance,
}

/// LLM-as-judge configuration.
///
/// Configuration for using an LLM to evaluate outputs.
///
/// # API Reference
///
/// Based on Python SDK `LLMEvaluator` (evaluation/llm_evaluator.py)
///
/// # Example
///
/// ```
/// use langstar_sdk::evaluations::{LlmJudgeConfig, ScoreType};
///
/// let config = LlmJudgeConfig {
///     model: "gpt-4o".to_string(),
///     provider: Some("openai".to_string()),
///     score_type: ScoreType::Categorical,
///     choices: Some(vec!["Y".to_string(), "N".to_string()]),
///     min: None,
///     max: None,
///     rubric: Some("Is the answer correct?".to_string()),
///     include_reasoning: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmJudgeConfig {
    /// Model name (e.g., "gpt-4o", "claude-3-opus-20240229")
    pub model: String,

    /// Model provider (e.g., "openai", "anthropic")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Type of scoring (categorical or continuous)
    pub score_type: ScoreType,

    /// Valid choices for categorical scoring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,

    /// Minimum value for continuous scoring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value for continuous scoring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Evaluation rubric or criteria
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric: Option<String>,

    /// Whether to include chain-of-thought reasoning
    pub include_reasoning: bool,
}

/// Score type for LLM-as-judge evaluators.
///
/// # API Reference
///
/// Based on Python SDK score config types (evaluation/llm_evaluator.py:13-30)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScoreType {
    /// Categorical scoring (e.g., Y/N, Pass/Fail)
    Categorical,
    /// Continuous numeric scoring (e.g., 0.0 to 1.0)
    Continuous,
}

/// Evaluator type enum.
///
/// Discriminates between heuristic and LLM-based evaluators.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvaluatorType {
    /// Heuristic evaluator (deterministic, zero-cost)
    Heuristic {
        /// Type of heuristic evaluator
        evaluator: HeuristicEvaluator,
        /// Optional configuration (e.g., regex pattern, ignore case)
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<Value>,
    },
    /// LLM-as-judge evaluator
    LlmJudge {
        /// LLM judge configuration
        config: LlmJudgeConfig,
    },
}

// ============================================================================
// Online Evaluation (Run Rules) Types
// ============================================================================

/// Code evaluator for server-side execution.
///
/// # API Reference
///
/// Maps to `CodeEvaluatorTopLevel` in OpenAPI spec.
///
/// # Example
///
/// ```
/// use langstar_sdk::evaluations::{CodeEvaluator, CodeEvaluatorLanguage};
///
/// let evaluator = CodeEvaluator {
///     code: r#"
/// def evaluate(inputs, outputs, reference_outputs):
///     return {"score": 1.0 if outputs.get("answer") == "42" else 0.0}
/// "#.to_string(),
///     language: Some(CodeEvaluatorLanguage::Python),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEvaluator {
    /// Evaluator source code
    pub code: String,

    /// Programming language (default: Python)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<CodeEvaluatorLanguage>,
}

/// Programming language for code evaluators.
///
/// # API Reference
///
/// OpenAPI enum: `["python", "javascript"]`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CodeEvaluatorLanguage {
    /// Python evaluator
    Python,
    /// JavaScript evaluator
    JavaScript,
}

/// Structured (LLM-as-judge) evaluator for online evaluation.
///
/// # API Reference
///
/// Maps to `EvaluatorStructuredOutput` in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredEvaluator {
    /// LangChain Hub prompt reference (e.g., "langchain/correctness-evaluator")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub_ref: Option<String>,

    /// Prompt as array of (role, content) tuples
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Vec<(String, String)>>,

    /// Template format (e.g., "f-string")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_format: Option<String>,

    /// JSON schema for structured output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,

    /// Maps template variables to run/example fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_mapping: Option<Value>,

    /// Model configuration (provider, model name, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Value>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_type_serialization() {
        let continuous = FeedbackType::Continuous;
        let json = serde_json::to_string(&continuous).unwrap();
        assert_eq!(json, "\"continuous\"");

        let categorical = FeedbackType::Categorical;
        let json = serde_json::to_string(&categorical).unwrap();
        assert_eq!(json, "\"categorical\"");

        let freeform = FeedbackType::Freeform;
        let json = serde_json::to_string(&freeform).unwrap();
        assert_eq!(json, "\"freeform\"");
    }

    #[test]
    fn test_feedback_config_continuous() {
        let config = FeedbackConfig {
            feedback_type: FeedbackType::Continuous,
            min: Some(0.0),
            max: Some(1.0),
            categories: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"continuous\""));
        assert!(json.contains("\"min\":0.0"));
        assert!(json.contains("\"max\":1.0"));
        assert!(!json.contains("categories"));
    }

    #[test]
    fn test_feedback_config_categorical() {
        let config = FeedbackConfig {
            feedback_type: FeedbackType::Categorical,
            min: None,
            max: None,
            categories: Some(vec![
                FeedbackCategory {
                    value: 1.0,
                    label: Some("Y".to_string()),
                },
                FeedbackCategory {
                    value: 0.0,
                    label: Some("N".to_string()),
                },
            ]),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"categorical\""));
        assert!(json.contains("\"categories\""));
        assert!(!json.contains("\"min\""));
    }

    #[test]
    fn test_feedback_create_serialization() {
        let request = FeedbackCreate {
            key: "accuracy".to_string(),
            score: Some(0.95),
            comment: Some("Good answer".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"key\":\"accuracy\""));
        assert!(json.contains("\"score\":0.95"));
        assert!(json.contains("\"comment\":\"Good answer\""));
    }

    #[test]
    fn test_evaluation_result_serialization() {
        let result = EvaluationResult {
            key: "exact_match".to_string(),
            score: Some(1.0),
            comment: Some("Match found".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"key\":\"exact_match\""));
        assert!(json.contains("\"score\":1.0"));
        assert!(json.contains("\"comment\":\"Match found\""));
    }

    #[test]
    fn test_heuristic_evaluator_serialization() {
        let evaluator = HeuristicEvaluator::ExactMatch;
        let json = serde_json::to_string(&evaluator).unwrap();
        assert_eq!(json, "\"exact_match\"");

        let evaluator = HeuristicEvaluator::RegexMatch;
        let json = serde_json::to_string(&evaluator).unwrap();
        assert_eq!(json, "\"regex_match\"");
    }

    #[test]
    fn test_score_type_serialization() {
        let categorical = ScoreType::Categorical;
        let json = serde_json::to_string(&categorical).unwrap();
        assert_eq!(json, "\"categorical\"");

        let continuous = ScoreType::Continuous;
        let json = serde_json::to_string(&continuous).unwrap();
        assert_eq!(json, "\"continuous\"");
    }

    #[test]
    fn test_code_evaluator_language_serialization() {
        let python = CodeEvaluatorLanguage::Python;
        let json = serde_json::to_string(&python).unwrap();
        assert_eq!(json, "\"python\"");

        let js = CodeEvaluatorLanguage::JavaScript;
        let json = serde_json::to_string(&js).unwrap();
        assert_eq!(json, "\"javascript\"");
    }

    #[test]
    fn test_evaluator_type_serialization() {
        let heuristic = EvaluatorType::Heuristic {
            evaluator: HeuristicEvaluator::ExactMatch,
            config: None,
        };
        let json = serde_json::to_string(&heuristic).unwrap();
        assert!(json.contains("\"type\":\"heuristic\""));
        assert!(json.contains("\"evaluator\":\"exact_match\""));

        let llm = EvaluatorType::LlmJudge {
            config: LlmJudgeConfig {
                model: "gpt-4o".to_string(),
                provider: Some("openai".to_string()),
                score_type: ScoreType::Categorical,
                choices: Some(vec!["Y".to_string(), "N".to_string()]),
                min: None,
                max: None,
                rubric: None,
                include_reasoning: false,
            },
        };
        let json = serde_json::to_string(&llm).unwrap();
        assert!(json.contains("\"type\":\"llm_judge\""));
        assert!(json.contains("\"model\":\"gpt-4o\""));
    }
}
