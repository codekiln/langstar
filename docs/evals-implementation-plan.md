# Langstar Evaluations Feature - Implementation Plan

## Overview

This document outlines the implementation of the `langstar eval` command group for creating, running, and managing LangSmith evaluations on datasets.

## Architecture

### Component Structure

```
langstar/
├── cli/src/commands/eval.rs          # CLI command definitions and argument parsing
├── sdk/src/evaluations.rs            # Evaluation types and API interactions
├── sdk/src/evaluators.rs             # Heuristic and LLM-judge evaluator implementations
├── cli/tests/eval_command_test.rs    # CLI command tests
└── sdk/tests/evaluations_test.rs     # SDK evaluation tests
```

### Command Hierarchy

```
langstar eval
├── create      # Create new evaluation configuration
├── run         # Execute evaluation on dataset
├── list        # List all evaluations
├── get         # Get details of specific evaluation
└── export      # Export evaluation results
```

## Evaluator Types

### 1. Heuristic Evaluators (Zero-Cost)

Deterministic evaluators that run locally without API calls:

| Evaluator           | Function                 | Use Case                       | Score Range |
| ------------------- | ------------------------ | ------------------------------ | ----------- |
| **exact_match**     | Exact string equality    | Validation of exact outputs    | 0.0 or 1.0  |
| **contains**        | Substring presence check | Keyword/phrase verification    | 0.0 or 1.0  |
| **regex_match**     | Regex pattern matching   | Format validation              | 0.0 or 1.0  |
| **json_valid**      | JSON syntax validation   | JSON output verification       | 0.0 or 1.0  |
| **string_distance** | Levenshtein distance     | Fuzzy matching, typo detection | Continuous  |

**Implementation**: Located in `sdk/src/evaluators.rs`

### 2. LLM-as-Judge Evaluators

Model-based evaluators that use LLM to score outputs:

**Configuration**:

- `judge_model`: Model name (e.g., "claude-3-5-sonnet-20241022")
- `judge_provider`: Provider (e.g., "anthropic", "openai")
- `judge_prompt_file`: Path to rubric/prompt file
- `score_type`: Categorical or Continuous
- `include_reasoning`: Whether to include explanation

**Score Types**:

| Type            | Score Field         | Value Field      | Example Use Cases                     |
| --------------- | ------------------- | ---------------- | ------------------------------------- |
| **Categorical** | Optional numeric    | String from enum | Pass/Fail, Y/N, Rating scales (A/B/C) |
| **Continuous**  | Float in [min, max] | Optional string  | Numeric scores (0-1, 1-10, 0-100)     |

**Implementation**: Located in `sdk/src/evaluations.rs`

## CLI Commands

### `eval create`

Create a new evaluation configuration.

**Required Arguments**:

- `--name <STRING>` - Evaluation name
- `--dataset <STRING>` - Dataset ID or name
- `--evaluator <TYPE>` - Evaluator type (exact-match, contains, regex-match, json-valid, string-distance, llm-judge)

**LLM Judge Arguments** (required when `--evaluator llm-judge`):

- `--judge-model <STRING>` - Judge model name
- `--judge-provider <STRING>` - Judge model provider
- `--judge-prompt-file <PATH>` - Path to judge prompt/rubric file
- `--score-type <categorical|continuous>` - Scoring type
- `--score-choices <LIST>` - Comma-separated choices for categorical scoring
- `--score-min <FLOAT>` - Minimum score for continuous scoring
- `--score-max <FLOAT>` - Maximum score for continuous scoring
- `--include-reasoning` - Include reasoning in output

**Output Options**:

- `--json` - Output as JSON

**Example - Heuristic Evaluator**:

```bash
langstar eval create \
  --name "exact-match-validation" \
  --dataset "my-test-dataset" \
  --evaluator exact-match
```

**Example - LLM Judge with Categorical Scoring**:

```bash
langstar eval create \
  --name "response-quality-judge" \
  --dataset "customer-support-dataset" \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubrics/quality-rubric.txt" \
  --score-type categorical \
  --score-choices "Poor,Fair,Good,Excellent" \
  --include-reasoning
```

**Example - LLM Judge with Continuous Scoring**:

```bash
langstar eval create \
  --name "relevance-score" \
  --dataset "qa-dataset" \
  --evaluator llm-judge \
  --judge-model "gpt-4" \
  --judge-provider "openai" \
  --judge-prompt-file "./rubrics/relevance.txt" \
  --score-type continuous \
  --score-min 0.0 \
  --score-max 1.0
```

### `eval run`

Execute an evaluation on a dataset.

**Required Arguments**:

- `<EVAL_ID>` - UUID of evaluation to run

**Optional Arguments**:

- `--preview <N>` - Preview mode: only run on first N examples
- `--dry-run` - Validate configuration without executing
- `--json` - Output as JSON

**Example**:

```bash
# Full evaluation run
langstar eval run 550e8400-e29b-41d4-a716-446655440000

# Preview with first 10 examples
langstar eval run 550e8400-e29b-41d4-a716-446655440000 --preview 10

# Dry run (validate only)
langstar eval run 550e8400-e29b-41d4-a716-446655440000 --dry-run
```

### `eval list`

List evaluations with optional filtering.

**Optional Arguments**:

- `--name <STRING>` - Filter by evaluation name
- `--dataset <UUID>` - Filter by dataset ID
- `--limit <N>` - Maximum number of results
- `--offset <N>` - Skip first N results
- `--json` - Output as JSON

**Example**:

```bash
# List all evaluations
langstar eval list

# Filter by name
langstar eval list --name "response-quality"

# Filter by dataset
langstar eval list --dataset 550e8400-e29b-41d4-a716-446655440000

# Paginated results
langstar eval list --limit 20 --offset 40
```

### `eval get`

Get detailed information about a specific evaluation.

**Required Arguments**:

- `<EVAL_ID>` - UUID of evaluation

**Optional Arguments**:

- `--json` - Output as JSON

**Example**:

```bash
langstar eval get 550e8400-e29b-41d4-a716-446655440000
```

### `eval export`

Export evaluation results to file.

**Required Arguments**:

- `<EVAL_ID>` - UUID of evaluation

**Optional Arguments**:

- `--format <csv|json|jsonl>` - Export format (default: jsonl)
- `--output <PATH>` - Output file path (default: stdout)
- `--include-metadata` - Include run metadata in export

**Example**:

```bash
# Export to JSONL (default)
langstar eval export 550e8400-e29b-41d4-a716-446655440000

# Export to CSV file
langstar eval export 550e8400-e29b-41d4-a716-446655440000 \
  --format csv \
  --output results.csv

# Export with metadata
langstar eval export 550e8400-e29b-41d4-a716-446655440000 \
  --format json \
  --output results.json \
  --include-metadata
```

## Environment Variables

| Variable            | Required    | Used By               | Description                                                         |
| ------------------- | ----------- | --------------------- | ------------------------------------------------------------------- |
| `LANGSMITH_API_KEY` | Yes         | All commands          | LangSmith API authentication key                                    |
| `LANGSMITH_API_URL` | No          | All commands          | LangSmith API base URL (default: `https://api.smith.langchain.com`) |
| `ANTHROPIC_API_KEY` | Conditional | LLM judge (Anthropic) | Required for Anthropic judge models                                 |
| `OPENAI_API_KEY`    | Conditional | LLM judge (OpenAI)    | Required for OpenAI judge models                                    |

**Configuration Priority** (highest to lowest):

1. Command-line flags
2. Environment variables
3. Configuration file (`~/.langstar/config.toml`)
4. Default values

## SDK Public API

### Module: `langstar_sdk::evaluations`

**Types**:

```rust
pub enum FeedbackType {
    Continuous,
    Categorical,
    Freeform,
}

pub struct FeedbackCategory {
    pub value: f64,
    pub label: Option<String>,
}

pub struct FeedbackConfig {
    pub feedback_type: FeedbackType,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub categories: Option<Vec<FeedbackCategory>>,
}

pub struct LlmJudgeConfig {
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub score_type: ScoreType,
    pub include_reasoning: bool,
}

pub enum ScoreType {
    Categorical,
    Continuous,
}

pub struct EvaluationResult {
    pub score: Option<f64>,
    pub value: Option<String>,
    pub reasoning: Option<String>,
}
```

### Module: `langstar_sdk::evaluators`

**Heuristic Evaluator Functions**:

```rust
/// Returns 1.0 if exact match, 0.0 otherwise
pub fn exact_match(output: &str, expected: &str) -> f64

/// Returns 1.0 if output contains expected, 0.0 otherwise
pub fn contains(output: &str, expected: &str) -> f64

/// Returns Ok(1.0) if regex matches, Ok(0.0) if not, Err if invalid regex
pub fn regex_match(output: &str, pattern: &str) -> Result<f64, String>

/// Returns 1.0 if valid JSON, 0.0 otherwise
pub fn json_valid(output: &str) -> f64

/// Returns Levenshtein distance (lower = more similar)
pub fn string_distance(output: &str, expected: &str) -> f64
```

## Testing Strategy

### Unit Tests

Located in:

- `cli/tests/eval_command_test.rs` - CLI argument parsing and help text
- `sdk/tests/evaluations_test.rs` - SDK type serialization and logic

**Run**: `cargo test --package langstar-cli --test eval_command_test`

### Integration Tests

Require `LANGSMITH_API_KEY` environment variable.

**Run**: `cargo test --package langstar-sdk --test evaluations_test`

## Documentation Deliverables

### 1. README.md Updates

Add `## Evaluations` section with:

- Overview of eval feature
- Quick start examples
- Links to detailed docs

### 2. Inline Rustdoc Comments

All public items in:

- `sdk/src/evaluations.rs`
- `sdk/src/evaluators.rs`
- `cli/src/commands/eval.rs`

**Requirements**:

- Module-level documentation with examples
- Function/type documentation with:
  - Purpose and use cases
  - Arguments/fields with types
  - Return values/errors
  - Usage examples (with ``#[doc = "```"]`` blocks)
  - API reference links where applicable

### 3. Configuration Documentation

Document in `docs/configuration.md`:

- Required environment variables per evaluator type
- Configuration file format for evals
- Precedence rules

### 4. Examples and Recipes

Create `docs/examples/evaluations.md` with:

- End-to-end evaluation workflow
- Heuristic evaluator examples for each type
- LLM judge examples (categorical and continuous)
- Judge prompt/rubric examples
- Exporting and analyzing results

## Implementation Status

✅ **Completed**:

- CLI command structure and argument parsing (cli/src/commands/eval.rs:1-400)
- SDK evaluation types (sdk/src/evaluations.rs:1-250)
- Heuristic evaluator implementations (sdk/src/evaluators.rs:1-150)
- LLM judge configuration types (sdk/src/evaluations.rs:100-200)
- CLI tests (cli/tests/eval_command_test.rs:1-300)
- SDK tests (sdk/tests/evaluations_test.rs:1-200)

🚧 **In Progress** (Issue #374):

- Documentation updates
- Rustdoc comments for all public APIs
- README eval section
- Environment variable documentation
- Usage examples

## References

- **LangSmith API Docs**: https://api.smith.langchain.com/openapi.json
- **Feedback API**: `/api/v1/feedback`
- **Run Rules API**: `/api/v1/runs/rules`
- **Issue Tracker**: https://github.com/codekiln/langstar/issues/347 (parent epic)
- **Current Issue**: https://github.com/codekiln/langstar/issues/374

## Notes

- All evaluator scores are normalized to [0.0, 1.0] range where applicable
- Heuristic evaluators run locally (zero API cost)
- LLM judge evaluators incur API costs per evaluation
- The `--preview` flag is recommended for testing expensive LLM judge configs
- Judge prompt files should be plain text with clear scoring instructions
