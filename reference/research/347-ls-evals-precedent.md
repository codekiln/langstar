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

## References

- Python SDK source: `/workspace/reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/`
- Key files analyzed:
  - `evaluation/evaluator.py` - Core types and interfaces
  - `evaluation/llm_evaluator.py` - LLM-as-judge implementation
  - `evaluation/_runner.py` - evaluate() function
  - `evaluation/integrations/_langchain.py` - LangChain wrapper
  - `schemas.py` - Feedback and FeedbackConfig types
  - `client.py` - create_feedback() and evaluate() methods
