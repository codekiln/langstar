# OpenAPI Validation Report: LangSmith Evaluations (Feedback System)

**Issue**: #369 - 347.2-openapi-validation
**Parent**: #347 (ls-evals-basic milestone)
**Date**: 2025-11-28
**Status**: Complete

---

## Executive Summary

Validation of LangSmith's evaluation system against the OpenAPI specification. This report documents the feedback/evaluation API endpoints and schemas discovered through analysis of `reference/openapi/langchain/langsmith/openapi.json`.

**Key Findings:**
1. ✅ LangSmith uses "Feedback" as the primary concept for evaluations
2. ✅ 11 feedback/evaluation endpoints discovered
3. ✅ 36 schemas related to feedback/evaluation
4. ⚡ **Discovery**: Two evaluator types - structured (LLM-based) and code (heuristic)
5. ⚡ **Discovery**: Three feedback types: continuous, categorical, and freeform
6. ⚡ **Discovery**: Feedback formulas allow composite metrics from multiple feedbacks
7. ⚡ **Discovery**: Four feedback sources: app, api, model, and auto_eval

---

## 1. Endpoint Inventory

### 1.1 Core Feedback Endpoints

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/api/v1/feedback` | GET, POST | List and create feedback |
| `/api/v1/feedback/{feedback_id}` | GET, PATCH, DELETE | Read, update, delete feedback |
| `/api/v1/feedback/eager` | POST | Create feedback with immediate evaluation |
| `/feedback/batch` | POST | Batch create feedback |

### 1.2 Feedback Configuration Endpoints

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/api/v1/feedback-configs` | GET, POST, PATCH | Manage feedback configurations (evaluator definitions) |

### 1.3 Feedback Formula Endpoints

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/api/v1/feedback/formulas` | GET, POST | List and create feedback formulas |
| `/api/v1/feedback/formulas/{feedback_formula_id}` | GET, PUT, DELETE | Read, update, delete formulas |

### 1.4 Token-Based Feedback Endpoints

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/api/v1/feedback/tokens` | GET, POST | Manage feedback tokens |
| `/api/v1/feedback/tokens/{token}` | GET, POST | Use token to submit feedback |

### 1.5 Public Sharing Endpoints

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/api/v1/public/{share_token}/feedbacks` | GET | Get feedback via share token |
| `/api/v1/public/{share_token}/datasets/feedback` | GET | Get dataset feedback via share token |

**Total**: 11 endpoints across 5 categories

---

## 2. Schema Analysis

### 2.1 Core Feedback Schema

**FeedbackCreateSchema** - Used for creating evaluation results:

```json
{
  "required": ["key"],
  "properties": {
    "key": {"type": "string", "maxLength": 180},
    "score": {"anyOf": ["number", "integer", "boolean", "null"]},
    "value": {"anyOf": ["number", "integer", "boolean", "string", "object", "null"]},
    "comment": {"type": "string", "nullable": true},
    "correction": {"anyOf": ["object", "string", "null"]},
    "run_id": {"type": "string", "format": "uuid", "nullable": true},
    "session_id": {"type": "string", "format": "uuid", "nullable": true},
    "trace_id": {"type": "string", "format": "uuid", "nullable": true},
    "feedback_source": {"oneOf": ["AppFeedbackSource", "APIFeedbackSource", "ModelFeedbackSource", "AutoEvalFeedbackSource"]},
    "feedback_config": {"$ref": "FeedbackConfig"},
    "error": {"type": "boolean", "nullable": true}
  }
}
```

**Key Fields:**
- `key`: Feedback/evaluator identifier (e.g., "correctness", "helpfulness")
- `score`: Numeric, boolean, or null score
- `value`: More flexible value field (can be structured data)
- `comment`: Human-readable explanation
- `correction`: Suggested correction (for RLHF/feedback loops)
- `run_id`, `session_id`, `trace_id`: Links to evaluated run/trace
- `feedback_source`: Discriminated union of source types
- `error`: Flag for evaluation errors

### 2.2 Feedback Configuration (Evaluator Definition)

**FeedbackConfig** - Defines the feedback/evaluation schema:

```json
{
  "required": ["type"],
  "properties": {
    "type": {"enum": ["continuous", "categorical", "freeform"]},
    "min": {"type": "number", "nullable": true},
    "max": {"type": "number", "nullable": true},
    "categories": {"type": "array", "items": "FeedbackCategory", "nullable": true}
  }
}
```

**Feedback Types:**
1. **continuous**: Numeric score with min/max bounds (e.g., 0.0 to 1.0)
2. **categorical**: Multiple choice from predefined categories
3. **freeform**: Unstructured text feedback

**FeedbackCategory** - For categorical feedback:
```json
{
  "required": ["value"],
  "properties": {
    "value": {"type": "number"},
    "label": {"type": "string", "nullable": true}
  }
}
```

### 2.3 Evaluator Types

#### 2.3.1 Structured Evaluator (LLM-as-Judge)

**EvaluatorTopLevel** - Wraps structured output evaluator:
```json
{
  "required": ["structured"],
  "properties": {
    "structured": {"$ref": "EvaluatorStructuredOutput"}
  }
}
```

**EvaluatorStructuredOutput** - LLM-based evaluator configuration:
```json
{
  "properties": {
    "hub_ref": {"type": "string", "nullable": true},
    "prompt": {"type": "array", "items": ["string", "string"], "nullable": true},
    "template_format": {"type": "string", "nullable": true},
    "schema": {"type": "object", "nullable": true},
    "variable_mapping": {"type": "object", "additionalProperties": "string", "nullable": true},
    "model": {"type": "object", "nullable": true}
  }
}
```

**Key Fields:**
- `hub_ref`: Reference to prompt in LangChain Hub
- `prompt`: Array of role/content tuples for judge prompt
- `template_format`: Format of prompt template (e.g., "f-string", "mustache")
- `schema`: JSON schema for structured output
- `variable_mapping`: Maps run fields to prompt variables (e.g., `{"input": "input", "output": "output"}`)
- `model`: Model configuration for judge (provider, model name, parameters)

#### 2.3.2 Code Evaluator (Heuristic)

**CodeEvaluatorTopLevel** - Code-based evaluator:
```json
{
  "required": ["code"],
  "properties": {
    "code": {"type": "string"},
    "language": {"enum": ["python", "javascript"], "default": "python", "nullable": true}
  }
}
```

**Use Cases:**
- Exact match
- String contains
- Regex matching
- JSON validation
- Custom heuristic functions

### 2.4 Feedback Sources

Four types of feedback sources (discriminated union):

1. **AppFeedbackSource** - Manual feedback from LangSmith UI
2. **APIFeedbackSource** - Programmatic feedback via API
3. **ModelFeedbackSource** - Feedback from LLM responses
4. **AutoEvalFeedbackSource** - Automated evaluation results

**AutoEvalFeedbackSource** (most relevant for evals):
```json
{
  "properties": {
    "type": {"const": "auto_eval"},
    "metadata": {"type": "object", "nullable": true}
  }
}
```

### 2.5 Feedback Formulas

**FeedbackFormula** - Composite metrics using weighted aggregation:
```json
{
  "required": ["feedback_key", "aggregation_type", "formula_parts", "id", "created_at", "modified_at"],
  "properties": {
    "id": {"type": "string", "format": "uuid"},
    "dataset_id": {"type": "string", "format": "uuid", "nullable": true},
    "session_id": {"type": "string", "format": "uuid", "nullable": true},
    "feedback_key": {"type": "string"},
    "aggregation_type": {"type": "string", "enum": ["sum", "avg"]},
    "formula_parts": {"type": "array", "items": "FeedbackFormulaWeightedVariable", "minItems": 1, "maxItems": 50},
    "created_at": {"type": "string", "format": "date-time"},
    "modified_at": {"type": "string", "format": "date-time"}
  }
}
```

**FeedbackFormulaWeightedVariable**:
```json
{
  "required": ["part_type", "key", "weight"],
  "properties": {
    "part_type": {"type": "string", "const": "weighted_key"},
    "key": {"type": "string", "minLength": 1},
    "weight": {"type": "number"}
  }
}
```

**Example Use Case:**
Combine "correctness", "helpfulness", and "toxicity" scores using weighted sum:
```json
{
  "feedback_key": "quality",
  "aggregation_type": "sum",
  "formula_parts": [
    {"part_type": "weighted_key", "key": "correctness", "weight": 0.5},
    {"part_type": "weighted_key", "key": "helpfulness", "weight": 0.3},
    {"part_type": "weighted_key", "key": "toxicity", "weight": -0.2}
  ]
}
```

---

## 3. Key Discoveries

### 3.1 Terminology: "Feedback" = "Evaluation"

LangSmith uses "feedback" as the umbrella term for evaluations. This includes:
- Manual human feedback
- Automated evaluations (heuristic or LLM-based)
- Model-generated scores

### 3.2 Two Evaluator Paradigms

| Type | Schema | Use Case | Execution |
|------|--------|----------|-----------|
| **Structured** | `EvaluatorStructuredOutput` | LLM-as-judge evaluations | Calls LLM with prompt + schema |
| **Code** | `CodeEvaluatorTopLevel` | Heuristic evaluations | Executes Python/JS code |

### 3.3 Feedback Configuration Flexibility

Feedback configs define:
- **Type**: continuous (numeric), categorical (enum), freeform (text)
- **Bounds**: min/max for continuous scores
- **Categories**: Predefined options for categorical

This allows both simple pass/fail (boolean) and nuanced Likert-scale (0-5) evaluations.

### 3.4 Feedback Formulas for Composite Metrics

Formulas enable:
- Weighted combinations of multiple evaluations (sum or average)
- Simple aggregation types: "sum" (weighted sum) or "avg" (weighted average)
- Each formula can combine 1-50 feedback keys with specific weights

### 3.5 Four Feedback Sources

| Source | Type | Description |
|--------|------|-------------|
| `app` | Manual | Human annotation in UI |
| `api` | Programmatic | Direct API calls |
| `model` | LLM-generated | Model's own evaluation |
| `auto_eval` | Automated | Heuristic or LLM judge |

---

## 4. jq Queries Used

### 4.1 List All Feedback/Eval Endpoints
```bash
jq '.paths | keys | map(select(. | test("feedback|evaluator"; "i")))' \
  reference/openapi/langchain/langsmith/openapi.json
```

### 4.2 Extract Feedback Endpoints to File
```bash
jq '.paths | with_entries(select(.key | test("feedback|evaluator"; "i")))' \
  reference/openapi/langchain/langsmith/openapi.json \
  > reference/api-specs/langsmith/evals-endpoints.json
```

### 4.3 Extract Feedback/Eval Schemas
```bash
jq '.components.schemas | with_entries(select(.key | test("feedback|evaluator|eval"; "i")))' \
  reference/openapi/langchain/langsmith/openapi.json \
  > reference/api-specs/langsmith/evals-schemas.json
```

### 4.4 Get Specific Schema
```bash
jq '.components.schemas.EvaluatorStructuredOutput' \
  reference/openapi/langchain/langsmith/openapi.json

jq '.components.schemas.CodeEvaluatorTopLevel' \
  reference/openapi/langchain/langsmith/openapi.json
```

### 4.5 List Endpoint Methods
```bash
jq 'to_entries | map({path: .key, methods: (.value | keys)})' \
  reference/api-specs/langsmith/evals-endpoints.json
```

### 4.6 Get Feedback Type Enum
```bash
jq '.components.schemas.FeedbackType' \
  reference/api-specs/langsmith/evals-schemas.json
```

---

## 5. Recommendations for Implementation (#347)

### 5.1 SDK Type Definitions (Phase: SDK Types)

**Core Feedback Types:**
```rust
// In sdk/src/types/feedback.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackCreate {
    pub key: String,  // Max 180 chars
    pub score: Option<FeedbackScore>,
    pub value: Option<serde_json::Value>,
    pub comment: Option<String>,
    pub correction: Option<FeedbackCorrection>,
    pub run_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub trace_id: Option<Uuid>,
    pub feedback_source: Option<FeedbackSource>,
    pub feedback_config: Option<FeedbackConfig>,
    pub error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeedbackScore {
    Number(f64),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeedbackCorrection {
    Object(serde_json::Value),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackConfig {
    pub r#type: FeedbackType,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub categories: Option<Vec<FeedbackCategory>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    Continuous,
    Categorical,
    Freeform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackCategory {
    pub value: f64,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeedbackSource {
    App { metadata: Option<serde_json::Value> },
    Api { metadata: Option<serde_json::Value> },
    Model { metadata: Option<serde_json::Value> },
    AutoEval { metadata: Option<serde_json::Value> },
}
```

**Evaluator Types:**
```rust
// In sdk/src/types/evaluators.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluatorStructured {
    pub hub_ref: Option<String>,
    pub prompt: Option<Vec<(String, String)>>,
    pub template_format: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub variable_mapping: Option<HashMap<String, String>>,
    pub model: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeEvaluator {
    pub code: String,
    pub language: Option<EvaluatorLanguage>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvaluatorLanguage {
    Python,
    Javascript,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackFormula {
    pub id: Uuid,
    pub dataset_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub feedback_key: String,
    pub aggregation_type: AggregationType,
    pub formula_parts: Vec<FeedbackFormulaPart>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationType {
    Sum,
    Avg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackFormulaPart {
    pub part_type: String,  // Always "weighted_key"
    pub key: String,
    pub weight: f64,
}
```

### 5.2 SDK Client Methods (Phase: SDK Client)

```rust
// In sdk/src/client.rs

impl LangSmithClient {
    // Core feedback operations
    pub async fn create_feedback(&self, feedback: FeedbackCreate) -> Result<Feedback>;
    pub async fn list_feedback(&self, run_id: Option<Uuid>) -> Result<Vec<Feedback>>;
    pub async fn get_feedback(&self, feedback_id: Uuid) -> Result<Feedback>;
    pub async fn update_feedback(&self, feedback_id: Uuid, update: FeedbackUpdate) -> Result<Feedback>;
    pub async fn delete_feedback(&self, feedback_id: Uuid) -> Result<()>;

    // Batch operations
    pub async fn create_feedback_batch(&self, feedbacks: Vec<FeedbackCreate>) -> Result<Vec<Feedback>>;
    pub async fn create_feedback_eager(&self, feedback: FeedbackCreate) -> Result<Feedback>;

    // Feedback configs (evaluator definitions)
    pub async fn list_feedback_configs(&self, keys: Option<Vec<String>>) -> Result<Vec<FeedbackConfigSchema>>;
    pub async fn create_feedback_config(&self, config: CreateFeedbackConfig) -> Result<FeedbackConfigSchema>;
    pub async fn update_feedback_config(&self, update: UpdateFeedbackConfig) -> Result<FeedbackConfigSchema>;

    // Feedback formulas
    pub async fn list_feedback_formulas(&self) -> Result<Vec<FeedbackFormula>>;
    pub async fn create_feedback_formula(&self, formula: FeedbackFormulaCreate) -> Result<FeedbackFormula>;
    pub async fn get_feedback_formula(&self, formula_id: Uuid) -> Result<FeedbackFormula>;
    pub async fn update_feedback_formula(&self, formula_id: Uuid, formula: FeedbackFormulaUpdate) -> Result<FeedbackFormula>;
    pub async fn delete_feedback_formula(&self, formula_id: Uuid) -> Result<()>;
}
```

### 5.3 CLI Commands (Phase: CLI Implementation)

```rust
// In cli/src/commands/evals.rs

#[derive(Subcommand)]
pub enum EvalsCommand {
    /// Create feedback/evaluation for a run
    Create {
        /// Run ID to evaluate
        #[arg(long)]
        run_id: Uuid,

        /// Feedback key (evaluator name)
        #[arg(long)]
        key: String,

        /// Score (numeric, boolean, or string)
        #[arg(long)]
        score: Option<String>,

        /// Comment/reasoning
        #[arg(long)]
        comment: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List feedback for a run
    List {
        /// Run ID
        #[arg(long)]
        run_id: Option<Uuid>,

        /// Feedback key filter
        #[arg(long)]
        key: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create or update feedback config (evaluator definition)
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Manage feedback formulas
    Formula {
        #[command(subcommand)]
        command: FormulaCommand,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// List feedback configs
    List {
        /// Filter by keys
        #[arg(long)]
        keys: Option<Vec<String>>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create feedback config
    Create {
        /// Config key
        #[arg(long)]
        key: String,

        /// Type: continuous, categorical, freeform
        #[arg(long)]
        type_: String,

        /// Min value (for continuous)
        #[arg(long)]
        min: Option<f64>,

        /// Max value (for continuous)
        #[arg(long)]
        max: Option<f64>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum FormulaCommand {
    /// List formulas
    List {
        #[arg(long)]
        json: bool,
    },

    /// Create formula
    Create {
        /// Feedback key for the formula result
        #[arg(long)]
        feedback_key: String,

        /// Aggregation type: sum or avg
        #[arg(long)]
        aggregation_type: String,

        /// Formula parts (format: key=weight, e.g., "correctness=0.5")
        #[arg(long)]
        parts: Vec<String>,

        #[arg(long)]
        json: bool,
    },

    /// Get formula
    Get {
        /// Formula ID
        formula_id: Uuid,

        #[arg(long)]
        json: bool,
    },
}
```

### 5.4 Priority Guidance

**Phase 1 (MVP)**: Basic Feedback CRUD
- Implement `FeedbackCreate`, `FeedbackConfig`, `FeedbackSource`
- SDK methods: `create_feedback`, `list_feedback`, `get_feedback`
- CLI: `langstar evals create`, `langstar evals list`

**Phase 2**: Feedback Configs
- Implement feedback config types (continuous, categorical, freeform)
- SDK methods: feedback config CRUD
- CLI: `langstar evals config create/list`

**Phase 3**: Advanced Features
- Feedback formulas
- Batch operations
- Code evaluators
- Structured evaluators (LLM-as-judge)

### 5.5 Open Questions for Research (#367)

Since issue #367 (research on Python SDK evaluators) is still open, these questions should be answered:

1. **How are structured evaluators (LLM-as-judge) executed?**
   - Is the `model` field a complete model config (provider, name, params)?
   - How are variables mapped from run data to prompt?
   - Is structured output schema enforced server-side or client-side?

2. **How are code evaluators sandboxed/executed?**
   - Are they executed server-side or client-side?
   - What's available in the execution context (run data, dataset data)?
   - Are there security constraints on code evaluators?

3. **What are common evaluator patterns?**
   - Example exact_match, contains, regex implementations
   - Example LLM-as-judge prompts for correctness/helpfulness
   - Best practices for variable_mapping

4. **How do feedback formulas handle missing values?**
   - What happens if a variable's feedback doesn't exist?
   - Are there default values or does the formula fail?

5. **Pagination for feedback listing?**
   - Does `GET /api/v1/feedback` support pagination?
   - What query parameters are available (run_id, key, limit, offset)?

---

## 6. Saved Artifacts

The following files were saved to `reference/api-specs/langsmith/`:

- **`evals-endpoints.json`** (45K) - All 11 feedback/evaluation endpoints
- **`evals-schemas.json`** (38K) - All 36 related schemas

---

## 7. Summary

| Category | Count | Details |
|----------|-------|---------|
| Endpoints Discovered | 11 | Feedback CRUD, configs, formulas, tokens |
| Schemas Discovered | 36 | Feedback types, evaluators, sources, formulas |
| Evaluator Types | 2 | Structured (LLM), Code (heuristic) |
| Feedback Types | 3 | Continuous, categorical, freeform |
| Feedback Sources | 4 | App, API, model, auto_eval |
| Composite Metric Support | Yes | Via feedback formulas |

**Recommendation**: Implement basic feedback CRUD first (Phase 1), then configs (Phase 2), then advanced features (Phase 3). Complete research issue #367 to understand Python SDK patterns before implementing evaluator execution logic.

---

## 8. Related Issues

- **#347** - Parent epic: ls-evals-basic milestone
- **#367** - Prerequisite research: langsmith-sdk evals precedent (merged via PR #376)
- **#369** - This validation issue

---

## 9. Online vs Offline Evaluation

This section clarifies a key architectural distinction in LangSmith evaluations.

### 9.1 Terminology

| Term | Also Known As | Where Runs | Trigger | Data Source |
|------|---------------|------------|---------|-------------|
| **Offline Evaluation** | Dataset-based, batch | Client-side | Manual / CI | Datasets |
| **Online Evaluation** | Production, real-time | Server-side | Automatic | Production traces |

### 9.2 Offline Evaluation (Client-Side)

Covered in Python SDK research (#367):
- Uses `evaluate()` function from `langsmith.evaluation`
- Evaluators run locally in your Python/JS environment
- Triggered manually or via CI pipelines
- Evaluates runs against dataset examples

```python
from langsmith import evaluate

# Client-side evaluation
results = evaluate(
    my_chain,
    data="my-dataset",
    evaluators=[exact_match, my_custom_evaluator],
)
```

### 9.3 Online Evaluation (Server-Side)

Configured via `/api/v1/feedback-configs` endpoint - evaluators run **within LangSmith**:

#### Code Evaluators (`CodeEvaluatorTopLevel`)

Custom Python or JavaScript code executed server-side:

```json
{
  "code": "def evaluate(run, example):\n    return {'score': 1 if run.outputs == example.outputs else 0}",
  "language": "python"
}
```

**Use cases:**
- Exact match checks
- Regex validation
- JSON schema validation
- Custom business logic

**Key constraints:**
- Sandboxed execution environment
- Limited library availability
- No network access from evaluator code

#### Structured Evaluators (`EvaluatorStructuredOutput`)

LLM-as-judge evaluators with configurable prompts:

```json
{
  "hub_ref": "langchain-ai/correctness",
  "prompt": [["system", "Evaluate correctness..."], ["human", "{input}\n{output}"]],
  "schema": {"type": "object", "properties": {"score": {"type": "number"}}},
  "variable_mapping": {"input": "inputs.question", "output": "outputs.answer"},
  "model": {"provider": "openai", "model": "gpt-4o"}
}
```

### 9.4 Automation Rules

Online evaluations are triggered via **automation rules** (configured in LangSmith UI):

1. **Sampling rules** - Run evaluator on X% of production traces
2. **Filter rules** - Run evaluator only on traces matching criteria
3. **Project rules** - Run evaluator on all traces in a project

The `evaluator_rules` field in dataset schemas (array of UUIDs) links datasets to their associated automation rules.

### 9.5 Feedback Source Distinction

| Source Type | Constant | Description |
|-------------|----------|-------------|
| `app` | Manual UI | Human annotation via LangSmith UI |
| `api` | Programmatic | Direct API calls |
| `model` | LLM-generated | Model's own evaluation |
| **`auto_eval`** | **Automated** | **Online evaluators (code or structured)** |

The `auto_eval` source type indicates feedback generated by online evaluation.

### 9.6 Implications for Langstar

For the Rust SDK, we should:

1. **Support feedback config CRUD** - To define online evaluators
2. **Support `auto_eval` source type** - To distinguish online vs offline results
3. **Consider code evaluator testing** - Local validation before deploying
4. **Document the online/offline distinction** - Help users choose appropriate approach

---

## 10. Next Steps

1. ~~Complete research on Python SDK evaluator implementation (#367)~~ (merged)
2. Create SDK type definitions based on schemas above
3. Implement SDK client methods for feedback CRUD
4. Add CLI commands for creating and listing feedback
5. Implement feedback config management
6. Add support for feedback formulas
7. Implement code evaluator execution (if client-side)
8. Implement structured evaluator execution (LLM-as-judge)
