# LangSmith SDK Evals Precedent Research

Research report for [Issue #367](https://github.com/codekiln/langstar/issues/367) - Part of the ls-evals-basic milestone.

## Executive Summary

The LangSmith Python SDK provides a flexible evaluation framework with two primary evaluator types:
1. **Heuristic evaluators** - Deterministic, zero-cost evaluators from LangChain (exact_match, regex_match, string_distance, etc.)
2. **LLM-as-judge evaluators** - Model-based evaluators using configurable LLMs (correctness, criteria, custom rubrics)

Key architectural patterns:
- Evaluators implement `RunEvaluator` interface with `evaluate_run(run, example)` signature
- Results use `EvaluationResult` with `key`, `score` (numeric), `value` (categorical/string), and optional `comment`
- Three feedback types: `continuous` (bounded numeric), `categorical` (enum values), `freeform` (text)
- The `evaluate()` function orchestrates running evaluators over datasets

## Table of Contents

1. [Core Architecture](#1-core-architecture)
2. [Heuristic Evaluators](#2-heuristic-evaluators)
3. [LLM-as-Judge Evaluators](#3-llm-as-judge-evaluators)
4. [Scoring Schemas](#4-scoring-schemas)
5. [Key Method Signatures](#5-key-method-signatures)
6. [Design Patterns for Langstar](#6-design-patterns-for-langstar)
7. [Online Evaluation (Server-Side Evaluators)](#7-online-evaluation-server-side-evaluators)
8. [Design Decisions for Langstar CLI](#8-design-decisions-for-langstar-cli)

---

## 1. Core Architecture

### 1.1 Module Structure

The evaluation module is located at `langsmith/evaluation/`:

```
evaluation/
├── __init__.py           # Public exports
├── _runner.py            # Main evaluate() function (sync)
├── _arunner.py           # Async evaluate functions
├── evaluator.py          # Core evaluator classes and types
├── string_evaluator.py   # Simple string-based evaluator
├── llm_evaluator.py      # LLM-as-judge evaluator
└── integrations/
    └── _langchain.py     # LangChain evaluator wrapper
```

Source: `/workspace/reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/evaluation/`

### 1.2 Core Types

#### EvaluationResult (evaluator.py:71-116)

```python
class EvaluationResult(BaseModel):
    """Evaluation result."""

    key: str
    """The aspect, metric name, or label for this evaluation."""
    score: SCORE_TYPE = None  # Union[StrictBool, StrictInt, StrictFloat, None]
    """The numeric score for this evaluation."""
    value: VALUE_TYPE = None  # Union[dict, str, None]
    """The value for this evaluation, if not numeric."""
    comment: Optional[str] = None
    """An explanation regarding the evaluation."""
    correction: Optional[dict] = None
    """What the correct value should be, if applicable."""
    evaluator_info: dict = Field(default_factory=dict)
    """Additional information about the evaluator."""
    feedback_config: Optional[Union[FeedbackConfig, dict]] = None
    """The configuration used to generate this feedback."""
    source_run_id: Optional[Union[uuid.UUID, str]] = None
    """The ID of the trace of the evaluator itself."""
    target_run_id: Optional[Union[uuid.UUID, str]] = None
    """The ID of the trace this evaluation is applied to."""
    extra: Optional[dict] = None
    """Metadata for the evaluator run."""
```

#### EvaluationResults (evaluator.py:118-126)

```python
class EvaluationResults(TypedDict, total=False):
    """Batch evaluation results - allows returning multiple metrics at once."""
    results: list[EvaluationResult]
```

#### RunEvaluator Interface (evaluator.py:129-154)

```python
class RunEvaluator:
    """Evaluator interface class."""

    @abstractmethod
    def evaluate_run(
        self,
        run: Run,
        example: Optional[Example] = None,
        evaluator_run_id: Optional[uuid.UUID] = None,
    ) -> Union[EvaluationResult, EvaluationResults]:
        """Evaluate an example."""

    async def aevaluate_run(
        self,
        run: Run,
        example: Optional[Example] = None,
        evaluator_run_id: Optional[uuid.UUID] = None,
    ) -> Union[EvaluationResult, EvaluationResults]:
        """Evaluate an example asynchronously."""
```

---

## 2. Heuristic Evaluators

### 2.1 Overview

Heuristic evaluators are deterministic, rule-based evaluators that don't require LLM calls. They are provided through LangChain's evaluation module and wrapped by `LangChainStringEvaluator`.

**Key characteristics:**
- Zero cost (no API calls)
- Deterministic results
- Fast execution
- Loaded via `langchain.evaluation.load_evaluator()`

### 2.2 Available Heuristic Evaluators

From `langsmith/evaluation/integrations/_langchain.py:30-144`:

| Evaluator | Description | Requires Reference | Requires Input |
|-----------|-------------|-------------------|----------------|
| `exact_match` | Exact string equality check | Yes | No |
| `regex_match` | Regex pattern matching | Yes | No |
| `string_distance` | Levenshtein/other distance metrics | Yes | No |
| `embedding_distance` | Semantic similarity via embeddings | Yes | No |

### 2.3 Usage Pattern

```python
from langsmith.evaluation import LangChainStringEvaluator, evaluate
import re

# Simple exact match
exact_match_evaluator = LangChainStringEvaluator("exact_match")

# Regex match with flags
regex_evaluator = LangChainStringEvaluator(
    "regex_match",
    config={"flags": re.IGNORECASE},
    prepare_data=lambda run, example: {
        "prediction": run.outputs["prediction"],
        "reference": example.outputs["answer"],
        "input": str(example.inputs),
    }
)

# String distance
string_distance_evaluator = LangChainStringEvaluator(
    "string_distance",
    config={"distance_metric": "levenshtein"},
    prepare_data=prepare_data,
)

# Embedding distance
embedding_evaluator = LangChainStringEvaluator("embedding_distance")
```

### 2.4 LangChainStringEvaluator Implementation

Source: `_langchain.py:146-279`

```python
class LangChainStringEvaluator:
    def __init__(
        self,
        evaluator: Union[StringEvaluator, str],  # Name or instance
        *,
        config: Optional[dict] = None,           # Evaluator config
        prepare_data: Optional[Callable] = None,  # Custom data mapper
    ):
        # If string, load from langchain
        if isinstance(evaluator, str):
            from langchain.evaluation import load_evaluator
            self.evaluator = load_evaluator(evaluator, **(config or {}))

    def as_run_evaluator(self) -> RunEvaluator:
        """Convert to RunEvaluator for use in evaluate()."""
```

### 2.5 Custom Heuristic Evaluators

The SDK supports custom evaluators via simple functions:

```python
from langsmith.evaluation import EvaluationResult

def custom_exact_match(run: Run, example: Example) -> dict:
    """Simple exact match evaluator."""
    pred = run.outputs.get("output", "")
    expected = example.outputs.get("answer", "")
    return {
        "key": "exact_match",
        "score": 1.0 if pred == expected else 0.0
    }

# Or return EvaluationResult directly
def custom_contains(run: Run, example: Example) -> EvaluationResult:
    pred = run.outputs.get("output", "")
    expected = example.outputs.get("expected", "")
    return EvaluationResult(
        key="contains",
        score=1.0 if expected in pred else 0.0,
        comment=f"Expected '{expected}' in output"
    )
```

---

## 3. LLM-as-Judge Evaluators

### 3.1 Overview

LLM-as-judge evaluators use language models to assess run outputs against criteria. Two main patterns exist:

1. **Built-in LLMEvaluator** - Native langsmith class with structured output
2. **LangChain criteria evaluators** - Via `LangChainStringEvaluator`

### 3.2 LLMEvaluator Class

Source: `llm_evaluator.py:76-295`

```python
class LLMEvaluator(RunEvaluator):
    """A class for building LLM-as-a-judge evaluators."""

    def __init__(
        self,
        *,
        prompt_template: Union[str, list[tuple[str, str]]],
        score_config: Union[CategoricalScoreConfig, ContinuousScoreConfig],
        map_variables: Optional[Callable[[Run, Optional[Example]], dict]] = None,
        model_name: str = "gpt-4o",
        model_provider: str = "openai",
        **kwargs,
    ):
```

**Key features:**
- Uses LangChain's `init_chat_model()` for model instantiation
- Supports any provider via `model_provider` parameter
- Uses structured output (JSON schema) for reliable scoring
- Prompt template can be string (user message) or list of (role, content) tuples

### 3.3 Score Configuration Types

#### CategoricalScoreConfig (llm_evaluator.py:13-19)

```python
class CategoricalScoreConfig(BaseModel):
    """Configuration for a categorical score."""
    key: str                              # Metric name
    choices: list[str]                    # Valid values e.g. ["Y", "N"]
    description: str                      # What this score measures
    include_explanation: bool = False     # Request chain-of-thought
    explanation_description: Optional[str] = None
```

#### ContinuousScoreConfig (llm_evaluator.py:22-30)

```python
class ContinuousScoreConfig(BaseModel):
    """Configuration for a continuous score."""
    key: str                              # Metric name
    min: float = 0                        # Minimum score
    max: float = 1                        # Maximum score
    description: str                      # What this score measures
    include_explanation: bool = False     # Request chain-of-thought
    explanation_description: Optional[str] = None
```

### 3.4 LLMEvaluator Usage Examples

Source: `tests/integration_tests/test_llm_evaluator.py:147-208`

```python
from langsmith.evaluation.llm_evaluator import (
    LLMEvaluator,
    CategoricalScoreConfig,
    ContinuousScoreConfig,
)

# Categorical (Y/N) evaluator
reference_accuracy = LLMEvaluator(
    prompt_template="Is the output accurate with respect to the expected output? "
                    "Y/N\nOutput: {output}\nExpected: {expected}",
    score_config=CategoricalScoreConfig(
        key="reference_accuracy",
        choices=["Y", "N"],
        description="Whether the output is accurate.",
        include_explanation=False,
    ),
)

# Multi-turn prompt with custom model
accuracy = LLMEvaluator(
    prompt_template=[
        ("system", "Is the output accurate? Y/N"),
        ("human", "Context: {context}\nQuestion: {question}\nOutput: {output}"),
    ],
    score_config=CategoricalScoreConfig(
        key="accuracy",
        choices=["Y", "N"],
        description="Output accuracy assessment.",
        include_explanation=True,  # Get explanation with score
    ),
    map_variables=lambda run, example: {
        "context": example.inputs.get("context", "") if example else "",
        "question": example.inputs.get("question", "") if example else "",
        "output": run.outputs.get("output", "") if run.outputs else "",
    },
    model_provider="anthropic",
    model_name="claude-3-haiku-20240307",
)

# Continuous (0-1) score
quality = LLMEvaluator(
    prompt_template="Rate the response quality from 0 to 1.\n{input}",
    score_config=ContinuousScoreConfig(
        key="quality",
        min=0,
        max=1,
        description="Overall response quality.",
    ),
)
```

### 3.5 LangChain Criteria Evaluators

Alternative approach using LangChain's built-in criteria evaluators:

```python
from langsmith.evaluation import LangChainStringEvaluator
from langchain_openai import ChatOpenAI
from langchain_anthropic import ChatAnthropic

# Criteria evaluator with custom LLM
criteria_evaluator = LangChainStringEvaluator(
    "criteria",
    config={
        "criteria": {
            "usefulness": "The prediction is useful if it is correct "
                         "and/or asks a useful followup question."
        },
        "llm": ChatOpenAI(model="gpt-4o"),
    },
)

# Labeled criteria (with reference)
labeled_criteria = LangChainStringEvaluator(
    "labeled_criteria",
    config={
        "criteria": {"accuracy": "Is the answer factually correct?"},
        "llm": ChatAnthropic(model="claude-3-opus-20240229"),
    },
    prepare_data=lambda run, example: {
        "prediction": run.outputs["prediction"],
        "reference": example.outputs["answer"],
        "input": str(example.inputs),
    },
)

# Labeled score string (likert-style)
scoring_evaluator = LangChainStringEvaluator(
    "labeled_score_string",
    config={
        "criteria": {
            "accuracy": "Score 1: Completely inaccurate\n"
                       "Score 5: Somewhat accurate\n"
                       "Score 10: Completely accurate"
        },
        "normalize_by": 10,  # Normalize to 0-1
        "llm": ChatAnthropic(model="claude-3-opus-20240229"),
    },
    prepare_data=prepare_data,
)
```

### 3.6 JSON Schema Generation for Structured Output

The LLMEvaluator generates JSON schemas for structured output:

Source: `llm_evaluator.py:33-73`

```python
def _create_score_json_schema(
    score_config: Union[CategoricalScoreConfig, ContinuousScoreConfig],
) -> dict:
    properties: dict[str, Any] = {}

    if isinstance(score_config, CategoricalScoreConfig):
        properties["score"] = {
            "type": "string",
            "enum": score_config.choices,
            "description": f"The score, one of {', '.join(score_config.choices)}.",
        }
    elif isinstance(score_config, ContinuousScoreConfig):
        properties["score"] = {
            "type": "number",
            "minimum": score_config.min,
            "maximum": score_config.max,
            "description": f"Score between {score_config.min} and {score_config.max}.",
        }

    if score_config.include_explanation:
        properties["explanation"] = {
            "type": "string",
            "description": score_config.explanation_description or "The explanation.",
        }

    return {
        "title": score_config.key,
        "description": score_config.description,
        "type": "object",
        "properties": properties,
        "required": ["score", "explanation"] if score_config.include_explanation else ["score"],
    }
```

---

## 4. Scoring Schemas

### 4.1 FeedbackConfig Types

Source: `schemas.py:690-701`

```python
class FeedbackConfig(TypedDict, total=False):
    """Represents _how_ a feedback value ought to be interpreted."""

    type: Literal["continuous", "categorical", "freeform"]
    """The type of feedback."""
    min: Optional[float]
    """The minimum value for continuous feedback."""
    max: Optional[float]
    """The maximum value for continuous feedback."""
    categories: Optional[list[FeedbackCategory]]
    """Valid categories for categorical feedback."""
```

### 4.2 Feedback Types Comparison

| Type | Score Field | Value Field | Use Case |
|------|-------------|-------------|----------|
| **continuous** | `float` in [min, max] | Optional string | Numeric scores (0-1, 1-10, etc.) |
| **categorical** | Optional numeric | `str` from enum | Y/N, Pass/Fail, A/B/C ratings |
| **freeform** | None | `str` | Comments, corrections, explanations |

### 4.3 FeedbackCategory Structure

Source: `schemas.py:681-688`

```python
class FeedbackCategory(TypedDict, total=False):
    """Specific value and label pair for feedback."""
    value: float
    """The numeric value (e.g., 0.0, 0.5, 1.0)."""
    label: Optional[str]
    """Human-readable label (e.g., "Poor", "Good", "Excellent")."""
```

### 4.4 Example: Likert Scale Configuration

```python
likert_config = FeedbackConfig(
    type="categorical",
    categories=[
        {"value": 1.0, "label": "Strongly Disagree"},
        {"value": 2.0, "label": "Disagree"},
        {"value": 3.0, "label": "Neutral"},
        {"value": 4.0, "label": "Agree"},
        {"value": 5.0, "label": "Strongly Agree"},
    ]
)
```

---

## 5. Key Method Signatures

### 5.1 Client.create_feedback()

Source: `client.py:6259-6398`

```python
def create_feedback(
    self,
    run_id: Optional[ID_TYPE] = None,
    key: str = "unnamed",
    *,
    score: Union[float, int, bool, None] = None,
    value: Union[str, dict, None] = None,
    trace_id: Optional[ID_TYPE] = None,
    correction: Union[dict, None] = None,
    comment: Union[str, None] = None,
    source_info: Optional[dict[str, Any]] = None,
    feedback_source_type: Union[FeedbackSourceType, str] = FeedbackSourceType.API,
    source_run_id: Optional[ID_TYPE] = None,
    feedback_id: Optional[ID_TYPE] = None,
    feedback_config: Optional[FeedbackConfig] = None,
    stop_after_attempt: int = 10,
    project_id: Optional[ID_TYPE] = None,
    comparative_experiment_id: Optional[ID_TYPE] = None,
    feedback_group_id: Optional[ID_TYPE] = None,
    extra: Optional[dict] = None,
    error: Optional[bool] = None,
    **kwargs: Any,
) -> Feedback:
    """Create feedback for a run."""
```

**Key parameters:**
- `run_id` / `trace_id` - Target run for feedback
- `key` - Metric name (e.g., "accuracy", "helpfulness")
- `score` - Numeric value (for continuous feedback)
- `value` - String/dict value (for categorical/freeform)
- `feedback_config` - Type and bounds configuration
- `source_run_id` - Links to evaluator's trace (for model-generated feedback)

### 5.2 evaluate() Function

Source: `_runner.py:88-300`

```python
def evaluate(
    target: Union[TARGET_T, Runnable, EXPERIMENT_T, tuple[EXPERIMENT_T, EXPERIMENT_T]],
    /,
    data: Optional[DATA_T] = None,
    evaluators: Optional[Sequence[EVALUATOR_T]] = None,
    summary_evaluators: Optional[Sequence[SUMMARY_EVALUATOR_T]] = None,
    metadata: Optional[dict] = None,
    experiment_prefix: Optional[str] = None,
    description: Optional[str] = None,
    max_concurrency: Optional[int] = 0,
    num_repetitions: int = 1,
    client: Optional[langsmith.Client] = None,
    blocking: bool = True,
    experiment: Optional[EXPERIMENT_T] = None,
    upload_results: bool = True,
    error_handling: Literal["log", "ignore"] = "log",
    **kwargs: Any,
) -> Union[ExperimentResults, ComparativeExperimentResults]:
```

**Type definitions:**
```python
TARGET_T = Union[Callable[[dict], dict], Callable[[dict, dict], dict]]
DATA_T = Union[str, uuid.UUID, Iterable[Example], Dataset]
EVALUATOR_T = Union[
    RunEvaluator,
    Callable[[Run, Optional[Example]], Union[EvaluationResult, EvaluationResults]],
    Callable[..., Union[dict, EvaluationResults, EvaluationResult]],
]
SUMMARY_EVALUATOR_T = Callable[
    [Sequence[Run], Sequence[Example]],
    Union[EvaluationResult, EvaluationResults],
]
```

### 5.3 Flexible Evaluator Function Signatures

The SDK normalizes various function signatures (from `evaluator.py:649-784`):

```python
# All of these work:

# Traditional signature
def eval1(run: Run, example: Example) -> dict:
    return {"score": 1.0}

# Simplified inputs
def eval2(inputs: dict, outputs: dict) -> dict:
    return {"score": 1.0}

# With reference
def eval3(outputs: dict, reference_outputs: dict) -> dict:
    return {"score": 1.0}

# With attachments
def eval4(inputs: dict, outputs: dict, attachments: dict) -> dict:
    return {"score": 1.0}
```

Supported argument names:
- `run` - Full Run object
- `example` - Full Example object
- `inputs` - `example.inputs`
- `outputs` - `run.outputs`
- `reference_outputs` - `example.outputs`
- `attachments` - `example.attachments`

---

## 6. Design Patterns for Langstar

### 6.1 Recommended Rust SDK Structure

Based on the Python SDK patterns, the Rust SDK should support:

```rust
// Core types
pub struct EvaluationResult {
    pub key: String,
    pub score: Option<f64>,           // Numeric score
    pub value: Option<String>,        // Categorical/string value
    pub comment: Option<String>,      // Explanation
    pub source_run_id: Option<Uuid>,  // Evaluator trace ID
    pub feedback_config: Option<FeedbackConfig>,
}

pub enum FeedbackType {
    Continuous { min: f64, max: f64 },
    Categorical { categories: Vec<FeedbackCategory> },
    Freeform,
}

pub struct FeedbackConfig {
    pub feedback_type: FeedbackType,
}
```

### 6.2 API Endpoints to Support

Based on the client methods:

| Operation | HTTP Method | Endpoint |
|-----------|-------------|----------|
| Create feedback | POST | `/feedback` |
| List feedback | GET | `/feedback` |
| Get feedback | GET | `/feedback/{feedback_id}` |
| Update feedback | PATCH | `/feedback/{feedback_id}` |
| Delete feedback | DELETE | `/feedback/{feedback_id}` |

### 6.3 CLI Commands Pattern

```bash
# Create feedback
langstar feedback create --run-id <id> --key accuracy --score 0.95

# List feedback for a run
langstar feedback list --run-id <id>

# List feedback for an experiment
langstar feedback list --experiment <name>
```

### 6.4 Key Insights for Implementation

1. **Flexible evaluator signatures** - Support both `(run, example)` and simplified `(inputs, outputs)` patterns
2. **Score vs Value** - Use `score` for numeric metrics, `value` for categorical
3. **FeedbackConfig is optional** - Only needed for first-time feedback key creation
4. **source_run_id** - Critical for linking evaluator traces to feedback
5. **Batch support** - `trace_id` enables background batching for latency-sensitive use

---

## 7. Online Evaluation (Server-Side Evaluators)

Research conducted for [Issue #381](https://github.com/codekiln/langstar/issues/381).

### 7.1 Overview

Online evaluation (also called "automation rules" or "run rules") allows you to automatically evaluate traces as they are ingested into LangSmith. Unlike offline evaluation (which runs evaluators locally via the SDK), online evaluation runs server-side within LangSmith's infrastructure.

**Key differences from offline evaluation:**

| Aspect | Offline Evaluation | Online Evaluation |
|--------|-------------------|------------------|
| Execution | Client-side (SDK) | Server-side (LangSmith) |
| Trigger | Manual via `evaluate()` | Automatic on trace ingestion |
| Sampling | All dataset examples | Configurable sampling rate |
| Code Runtime | Your environment | LangSmith sandboxed environment |
| Languages | Python/TypeScript | Python or JavaScript |

### 7.2 Automation Rules (RunRules)

Automation rules configure how online evaluators are triggered and applied.

#### 7.2.1 API Endpoints

| Operation | Method | Endpoint |
|-----------|--------|----------|
| List rules | GET | `/api/v1/runs/rules` |
| Create rule | POST | `/api/v1/runs/rules` |
| Update rule | PATCH | `/api/v1/runs/rules/{rule_id}` |
| Delete rule | DELETE | `/api/v1/runs/rules/{rule_id}` |
| Get rule logs | GET | `/api/v1/runs/rules/{rule_id}/logs` |
| Trigger rule manually | POST | `/api/v1/runs/rules/{rule_id}/trigger` |
| Trigger all rules | POST | `/api/v1/runs/rules/trigger` |

#### 7.2.2 RunRulesCreateSchema

Source: OpenAPI spec `RunRulesCreateSchema`

```typescript
interface RunRulesCreateSchema {
  // Required fields
  display_name: string;           // Human-readable rule name
  sampling_rate: number;          // 0.0-1.0, fraction of runs to evaluate

  // Targeting
  session_id?: string;            // Target specific tracing project
  dataset_id?: string;            // Reference dataset for evaluators

  // Filtering
  filter?: string;                // Run-level filter expression
  trace_filter?: string;          // Trace-level filter expression
  tree_filter?: string;           // Tree structure filter

  // Status
  is_enabled?: boolean;           // Default: true

  // Actions
  add_to_annotation_queue_id?: string;  // Send matching runs to queue
  add_to_dataset_id?: string;           // Add matching runs to dataset
  add_to_dataset_prefer_correction?: boolean;

  // Evaluators
  evaluators?: EvaluatorTopLevel[];      // LLM-as-judge evaluators
  code_evaluators?: CodeEvaluatorTopLevel[];  // Code evaluators

  // Advanced
  backfill_from?: string;         // ISO datetime to backfill from
  use_corrections_dataset?: boolean;
  num_few_shot_examples?: number;
  evaluator_version?: number;
  group_by?: "thread_id";         // Group runs by thread

  // Integrations
  alerts?: RunRulesPagerdutyAlertSchema[];
  webhooks?: RunRulesWebhookSchema[];
}
```

#### 7.2.3 Sampling Configuration

The `sampling_rate` field controls what percentage of matching runs are evaluated:
- `1.0` = 100% of runs (all matching runs)
- `0.1` = 10% of runs (random sampling)
- `0.01` = 1% of runs

**Example:** To evaluate 10% of production traces:
```json
{
  "display_name": "Production Quality Check",
  "session_id": "<project-uuid>",
  "sampling_rate": 0.1,
  "code_evaluators": [...]
}
```

#### 7.2.4 Filtering

Three filter types are available for targeting specific runs:

| Filter Type | Description | Example |
|-------------|-------------|---------|
| `filter` | Run-level attributes | `eq(status, "error")` |
| `trace_filter` | Root trace attributes | `has(metadata, "production")` |
| `tree_filter` | Run tree structure | Run type, depth, etc. |

### 7.3 Code Evaluators

Code evaluators allow you to write custom evaluation logic that runs server-side.

#### 7.3.1 CodeEvaluatorTopLevel Schema

```typescript
interface CodeEvaluatorTopLevel {
  code: string;              // Evaluator source code
  language?: "python" | "javascript";  // Default: "python"
}
```

#### 7.3.2 Execution Environment

**Languages supported:**
- Python (default)
- JavaScript

**Function signature (Python):**
```python
def evaluate(inputs: dict, outputs: dict, reference_outputs: dict | None) -> dict:
    """
    Args:
        inputs: The inputs to the run (example.inputs)
        outputs: The outputs from the run (run.outputs)
        reference_outputs: Expected outputs from dataset (example.outputs), if available

    Returns:
        dict with 'score' (numeric) and/or 'value' (categorical) and optional 'comment'
    """
    # Return format:
    return {
        "score": 1.0,  # or 0.0-1.0 numeric
        "value": "pass",  # or categorical value
        "comment": "Explanation of the score"
    }
```

**Function signature (JavaScript):**
```javascript
function evaluate({ inputs, outputs, referenceOutputs }) {
    return {
        score: 1.0,
        value: "pass",
        comment: "Explanation"
    };
}
```

#### 7.3.3 Example Code Evaluators

**Exact Match (Python):**
```python
def evaluate(inputs, outputs, reference_outputs):
    if not reference_outputs:
        return {"score": None, "comment": "No reference available"}

    actual = outputs.get("output", "")
    expected = reference_outputs.get("output", "")

    return {
        "score": 1.0 if actual == expected else 0.0,
        "comment": f"Expected: {expected[:100]}"
    }
```

**Contains Check (Python):**
```python
def evaluate(inputs, outputs, reference_outputs):
    output = outputs.get("output", "").lower()
    expected_term = reference_outputs.get("must_contain", "").lower()

    return {
        "score": 1.0 if expected_term in output else 0.0,
        "value": "pass" if expected_term in output else "fail"
    }
```

**JSON Validity (Python):**
```python
import json

def evaluate(inputs, outputs, reference_outputs):
    output = outputs.get("output", "")
    try:
        json.loads(output)
        return {"score": 1.0, "value": "valid_json"}
    except json.JSONDecodeError as e:
        return {"score": 0.0, "value": "invalid_json", "comment": str(e)}
```

**Regex Match (Python):**
```python
import re

def evaluate(inputs, outputs, reference_outputs):
    pattern = reference_outputs.get("pattern", "")
    output = outputs.get("output", "")

    match = re.search(pattern, output)
    return {
        "score": 1.0 if match else 0.0,
        "comment": f"Pattern: {pattern}"
    }
```

#### 7.3.4 Available Libraries

The server-side execution environment provides access to standard library modules. Based on the common evaluator patterns, the following are typically available:
- `json` - JSON parsing
- `re` - Regular expressions
- `math` - Math operations

**Note:** External libraries (numpy, pandas, etc.) are NOT available in the sandboxed environment. Code evaluators should use only standard library functions.

### 7.4 Structured Evaluators (LLM-as-Judge)

Structured evaluators use LLMs to assess outputs, with structured output schemas ensuring consistent results.

#### 7.4.1 EvaluatorStructuredOutput Schema

```typescript
interface EvaluatorStructuredOutput {
  // Prompt configuration
  hub_ref?: string;           // LangChain Hub prompt reference
  prompt?: [string, string][];  // Array of [role, content] tuples
  template_format?: string;   // Template format (e.g., "f-string")

  // Output schema
  schema?: object;            // JSON schema for structured output

  // Variable mapping
  variable_mapping?: {
    [variable_name: string]: string;  // Maps template vars to run/example fields
  };

  // Model configuration
  model?: {
    provider?: string;        // e.g., "openai", "anthropic"
    model?: string;          // e.g., "gpt-4", "claude-3-opus"
    // Additional model parameters
  };
}
```

#### 7.4.2 Variable Mapping

The `variable_mapping` field maps template variables to data sources:

| Source Path | Description |
|-------------|-------------|
| `run.inputs` | Run input data |
| `run.outputs` | Run output data |
| `example.inputs` | Dataset example inputs |
| `example.outputs` | Dataset example expected outputs |

**Example:**
```json
{
  "structured": {
    "prompt": [
      ["system", "Evaluate if the answer is factually correct."],
      ["human", "Question: {question}\nAnswer: {answer}\nExpected: {expected}"]
    ],
    "variable_mapping": {
      "question": "run.inputs.question",
      "answer": "run.outputs.response",
      "expected": "example.outputs.correct_answer"
    },
    "schema": {
      "type": "object",
      "properties": {
        "score": {"type": "number", "minimum": 0, "maximum": 1},
        "reasoning": {"type": "string"}
      },
      "required": ["score"]
    },
    "model": {
      "provider": "openai",
      "model": "gpt-4o"
    }
  }
}
```

### 7.5 Online vs Offline Evaluation Decision Matrix

| Use Case | Recommended | Rationale |
|----------|-------------|-----------|
| Development iteration | Offline | Fast feedback, local debugging |
| CI/CD testing | Offline | Deterministic, versioned |
| Production monitoring | Online | Automatic, sampling |
| Cost optimization | Online | Server-side, sampling |
| Custom complex logic | Offline | Full library access |
| Simple checks | Online | Code evaluators |
| LLM-as-judge at scale | Online | Server handles rate limits |
| Dataset experiments | Offline | Full control |

### 7.6 API Implementation Notes for Langstar

#### 7.6.1 Rust Types

```rust
/// Code evaluator for server-side execution
pub struct CodeEvaluator {
    pub code: String,
    pub language: Option<CodeEvaluatorLanguage>,
}

pub enum CodeEvaluatorLanguage {
    Python,
    JavaScript,
}

/// Structured (LLM-as-judge) evaluator
pub struct StructuredEvaluator {
    pub hub_ref: Option<String>,
    pub prompt: Option<Vec<(String, String)>>,  // [(role, content)]
    pub template_format: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub variable_mapping: Option<HashMap<String, String>>,
    pub model: Option<ModelConfig>,
}

/// Automation rule configuration
pub struct RunRule {
    pub id: Uuid,
    pub display_name: String,
    pub session_id: Option<Uuid>,
    pub sampling_rate: f64,
    pub is_enabled: bool,
    pub filter: Option<String>,
    pub evaluators: Option<Vec<StructuredEvaluator>>,
    pub code_evaluators: Option<Vec<CodeEvaluator>>,
    // ... additional fields
}
```

#### 7.6.2 CLI Commands Pattern

```bash
# List automation rules
langstar rules list --project <name>

# Create a code evaluator rule
langstar rules create \
  --name "Exact Match Check" \
  --project <name> \
  --sampling-rate 0.1 \
  --code-evaluator-file ./evaluator.py

# Create an LLM-as-judge rule
langstar rules create \
  --name "Quality Check" \
  --project <name> \
  --sampling-rate 0.05 \
  --hub-ref "langchain/correctness-evaluator"

# Trigger a rule manually
langstar rules trigger <rule-id>

# View rule logs
langstar rules logs <rule-id>
```

---

## 8. Design Decisions for Langstar CLI

Research conducted for [Issue #368](https://github.com/codekiln/langstar/issues/368).

This section documents DX (developer experience) design decisions for implementing evaluation commands in the langstar CLI, ensuring consistency with existing commands (`runs query`, `dataset`, etc.).

### 8.1 DX Consistency Analysis

#### 8.1.1 Existing CLI Patterns

The langstar CLI follows consistent patterns across commands:

| Pattern | `runs query` | `dataset` | Recommended for `eval` |
|---------|--------------|-----------|------------------------|
| **Filter syntax** | `--filter <expr>` | - | `--filter <expr>` |
| **Convenience filters** | `--tag`, `--meta KEY=VALUE` | `--name`, `--name-contains` | `--evaluator-type`, `--score-key` |
| **Output format** | `-o/--output table|json|json-pretty` | `--json` flag | `-o/--output` (prefer explicit) |
| **Limit** | `-l/--limit N` | `-l/--limit N` | `-l/--limit N` |
| **Resource ID** | `--project UUID` | `<dataset_id>` positional | See 8.3.1 |
| **Time filters** | `--since`, `--until`, `--preset` | - | `--since`, `--preset` |

#### 8.1.2 Argument Naming Conventions

Following existing patterns:
- **Long names**: kebab-case (`--evaluator-type`, not `--evaluator_type`)
- **Short names**: Single letter for frequent options (`-l` for limit, `-o` for output)
- **Flags**: `--json`, `--yes`/`-y` for confirmation skip
- **Value enums**: Explicit values via `ValueEnum` trait

### 8.2 Evaluator Configuration Patterns

#### 8.2.1 Evaluator Type Selection

**Recommended pattern:** Use `--evaluator` with type-specific subcommands or options.

```bash
# Option A: Type as primary argument (Recommended)
langstar eval run <id> --evaluator exact_match
langstar eval run <id> --evaluator llm_judge --judge-model gpt-4o

# Option B: Separate commands per type
langstar eval exact-match <id>
langstar eval llm-judge <id> --judge-model gpt-4o
```

**Recommendation:** Option A with `--evaluator <TYPE>` for consistency with how `runs query` handles `--run-type`. This keeps the command structure flat and discoverable.

#### 8.2.2 Evaluator Type Enum

```rust
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EvaluatorType {
    // Heuristic evaluators (zero-cost, deterministic)
    ExactMatch,
    Contains,
    RegexMatch,
    JsonValid,
    StringDistance,

    // LLM-as-judge evaluators (API cost, configurable)
    LlmJudge,
    Correctness,
    Helpfulness,
    Custom,
}
```

#### 8.2.3 Heuristic Evaluator Options

| Evaluator | Options | Example |
|-----------|---------|---------|
| `exact_match` | `--ignore-case`, `--ignore-whitespace` | `--evaluator exact_match --ignore-case` |
| `contains` | `--ignore-case`, `--expected <STRING>` | `--evaluator contains --expected "success"` |
| `regex_match` | `--pattern <REGEX>`, `--flags <FLAGS>` | `--evaluator regex_match --pattern "\\d+"` |
| `json_valid` | `--schema-file <PATH>` (optional) | `--evaluator json_valid` |
| `string_distance` | `--metric <METRIC>`, `--threshold <FLOAT>` | `--evaluator string_distance --metric levenshtein` |

#### 8.2.4 LLM-as-Judge Configuration

```bash
# Basic usage with defaults
langstar eval run <id> --evaluator llm_judge

# Full configuration
langstar eval run <id> \
  --evaluator llm_judge \
  --judge-model gpt-4o \
  --judge-provider openai \
  --rubric-file criteria.txt \
  --score-type categorical \
  --score-choices "pass,fail" \
  --include-explanation
```

**LLM Judge Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--judge-model` | String | `gpt-4o` | Model name for LLM judge |
| `--judge-provider` | Enum | `openai` | Model provider (openai, anthropic, etc.) |
| `--rubric` | String | - | Inline rubric text |
| `--rubric-file` | Path | - | Path to rubric file |
| `--score-type` | Enum | `categorical` | `categorical` or `continuous` |
| `--score-choices` | String | `Y,N` | Comma-separated categorical choices |
| `--score-min` | Float | `0.0` | Minimum for continuous scores |
| `--score-max` | Float | `1.0` | Maximum for continuous scores |
| `--include-explanation` | Flag | false | Request chain-of-thought reasoning |

### 8.3 CLI Command Structure

#### 8.3.1 Primary Commands

```bash
# Evaluate a single run
langstar eval run <RUN_ID> --evaluator <TYPE> [OPTIONS]

# Evaluate runs from a dataset
langstar eval dataset <DATASET_ID> --evaluator <TYPE> [OPTIONS]

# List available evaluator types
langstar eval types

# Create feedback manually
langstar feedback create --run-id <ID> --key <KEY> --score <SCORE>

# List feedback for a run
langstar feedback list --run-id <ID>
```

#### 8.3.2 Resource Identification Patterns

Consistent with existing commands:

```bash
# Positional for primary resource (like `dataset get <ID>`)
langstar eval run <RUN_ID> --evaluator exact_match

# Multiple resources via repeated flag (like `runs query --project`)
langstar eval run --run-id <ID1> --run-id <ID2> --evaluator exact_match

# Reference dataset for comparison
langstar eval run <RUN_ID> --evaluator exact_match --reference-dataset <DATASET_ID>
```

#### 8.3.3 Filtering and Batch Operations

```bash
# Evaluate all runs in a project with filtering
langstar eval batch \
  --project <PROJECT_NAME> \
  --evaluator exact_match \
  --filter 'eq(status, "success")' \
  --since 24h \
  --limit 100

# Evaluate using convenience filters
langstar eval batch \
  --project <PROJECT_NAME> \
  --evaluator llm_judge \
  --tag production \
  --meta model=gpt-4o
```

### 8.4 Output Formats

#### 8.4.1 Single Run Evaluation Output

**Table format (default):**
```
Evaluation Results for run 123e4567

Key          Score    Value    Comment
───────────────────────────────────────────
exact_match  1.0      -        -
accuracy     -        pass     Output matches expected
```

**JSON format (`-o json`):**
```json
{
  "run_id": "123e4567-e89b-12d3-a456-426614174000",
  "evaluations": [
    {
      "key": "exact_match",
      "score": 1.0,
      "value": null,
      "comment": null
    }
  ]
}
```

#### 8.4.2 LLM Judge Output with Reasoning

**Table format with explanation:**
```
Evaluation Results for run 123e4567

Key          Score    Value    Comment
───────────────────────────────────────────────────────────────
correctness  0.85     -        The answer is mostly correct but...

Explanation:
The response correctly identifies the main concept but omits
a key detail about error handling.
```

**JSON format:**
```json
{
  "run_id": "123e4567-e89b-12d3-a456-426614174000",
  "evaluations": [
    {
      "key": "correctness",
      "score": 0.85,
      "value": null,
      "comment": "The answer is mostly correct but omits error handling",
      "explanation": "The response correctly identifies the main concept...",
      "evaluator_info": {
        "model": "gpt-4o",
        "provider": "openai"
      }
    }
  ]
}
```

#### 8.4.3 Batch Evaluation Summary

**Table format:**
```
Batch Evaluation Summary (100 runs)

Evaluator     Avg Score    Pass Rate    Failed
────────────────────────────────────────────────
exact_match   0.72         72%          28
llm_judge     0.85         85%          15

Detailed results: langstar eval results <EXPERIMENT_ID>
```

#### 8.4.4 Machine-Readable Batch Output

**JSONL format (`-o jsonl`):**
```jsonl
{"run_id":"123e4567...","key":"exact_match","score":1.0}
{"run_id":"223e4567...","key":"exact_match","score":0.0}
```

### 8.5 Configuration Integration

#### 8.5.1 Config File Support

Evaluator presets can be defined in `langstar.toml`:

```toml
[eval]
default_evaluator = "exact_match"
default_judge_model = "gpt-4o"
default_judge_provider = "openai"

[eval.presets.production_check]
evaluator = "llm_judge"
judge_model = "claude-3-opus-20240229"
judge_provider = "anthropic"
rubric_file = "~/.langstar/rubrics/production.txt"
include_explanation = true

[eval.presets.quick_check]
evaluator = "exact_match"
ignore_case = true
ignore_whitespace = true
```

**Usage:**
```bash
# Use a preset
langstar eval run <ID> --preset production_check

# Override preset options
langstar eval run <ID> --preset production_check --judge-model gpt-4-turbo
```

#### 8.5.2 Environment Variable Mapping

| Env Var | Config Key | CLI Flag | Precedence |
|---------|------------|----------|------------|
| `LANGSTAR_EVAL_MODEL` | `eval.default_judge_model` | `--judge-model` | CLI > Env > Config |
| `LANGSTAR_EVAL_PROVIDER` | `eval.default_judge_provider` | `--judge-provider` | CLI > Env > Config |

### 8.6 Error Handling and User Feedback

#### 8.6.1 Progress Indicators

For batch operations:
```
Evaluating runs... [████████░░░░] 42/100 (llm_judge)
```

#### 8.6.2 Error Messages

```bash
# Missing required option
Error: --evaluator is required
Hint: Available types: exact_match, contains, regex_match, json_valid, llm_judge

# Invalid evaluator type
Error: Unknown evaluator type 'foo'
Available types:
  Heuristic: exact_match, contains, regex_match, json_valid, string_distance
  LLM Judge: llm_judge, correctness, helpfulness, custom

# Missing reference data
Error: Evaluator 'exact_match' requires reference outputs
Hint: Use --reference-dataset <ID> or ensure the run has associated example data
```

### 8.7 Design Decision Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Evaluator selection | `--evaluator <TYPE>` | Consistent with `--run-type` in runs query |
| Judge model config | `--judge-model`, `--judge-provider` | Explicit, matches SDK patterns |
| Rubric input | `--rubric` or `--rubric-file` | Flexible for inline or file-based |
| Score type | `--score-type categorical|continuous` | Matches SDK's CategoricalScoreConfig/ContinuousScoreConfig |
| Output format | `-o/--output table|json|jsonl` | Consistent with runs query |
| Batch filtering | Reuse runs query filter syntax | DX consistency, zero learning curve |
| Config presets | `[eval.presets.*]` in config | Reduces repetition for common workflows |

### 8.8 Future Considerations

1. **Online evaluation rules**: CLI support for creating/managing automation rules (Section 7)
2. **Custom evaluator upload**: Support for uploading Python code evaluators
3. **Comparative evaluation**: Side-by-side comparison of multiple runs
4. **Streaming results**: Real-time output for long-running batch evaluations

---

## References

- Python SDK source: `/workspace/reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/`
- OpenAPI spec: `/workspace/reference/openapi/langchain/langsmith/openapi.json`
- Extracted schemas: `/workspace/reference/api-specs/langsmith/evals-schemas.json`
- Key files analyzed:
  - `evaluation/evaluator.py` - Core types and interfaces
  - `evaluation/llm_evaluator.py` - LLM-as-judge implementation
  - `evaluation/_runner.py` - evaluate() function
  - `evaluation/integrations/_langchain.py` - LangChain wrapper
  - `schemas.py` - Feedback and FeedbackConfig types
  - `client.py` - create_feedback() and evaluate() methods
