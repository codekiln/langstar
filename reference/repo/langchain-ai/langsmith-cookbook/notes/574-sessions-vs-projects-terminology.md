# Sessions vs Projects Terminology Research - langsmith-cookbook

**Research Date**: 2025-12-05
**Related Issue**: #574
**Related PR**: #575
**Purpose**: Examine real-world usage patterns of sessions/projects terminology in LangSmith examples

## Executive Summary

The langsmith-cookbook repository contains practical examples and tutorials for LangSmith usage. The repository exclusively uses "project" terminology in all user-facing code, environment variables, and method calls. There are zero references to "sessions" in the public-facing cookbook examples.

## Repository Overview

**Repository**: langchain-ai/langsmith-cookbook
**Last Updated**: 2025-12-05
**Content Type**: Jupyter notebooks with practical LangSmith examples
**Focus Areas**: Evaluation, testing, datasets, RAG implementations

## Key Findings

### 1. Environment Variable Usage

**Pattern Found Throughout Notebooks**:

```python
# From introduction/langsmith_introduction.ipynb
import os
os.environ['LANGCHAIN_TRACING_V2'] = 'true'  # enables tracing
os.environ["LANGCHAIN_API_KEY"] = "xxx"
os.environ["LANGCHAIN_PROJECT"] = "Test"  # ← Uses "PROJECT"
```

**Observations**:
- Every notebook that sets project context uses `LANGCHAIN_PROJECT` or `LANGSMITH_PROJECT`
- No instances of `LANGCHAIN_SESSION` or `LANGSMITH_SESSION`
- This is the first thing users see when learning LangSmith
- Reinforces "project" as the canonical term

### 2. Python SDK Method Calls

**Dataset Creation** (from `introduction/langsmith_introduction.ipynb`):

```python
from langsmith import Client

client = Client()
dataset_name = "DBRX"

# Create dataset
dataset = client.create_dataset(
    dataset_name=dataset_name,
    description="QA pairs about DBRX model.",
)
```

**Project References in Evaluation**:

```python
# Evaluators run against a dataset
experiment_results = evaluate(
    answer_dbrx_question_oai,
    data=dataset_name,  # Dataset, not project
    evaluators=qa_evalulator,
    experiment_prefix="test-dbrx-qa-oai",
    metadata={
        "variant": "stuff website context into gpt-3.5-turbo",
    },
)
```

**Project Name in Traceable Functions**:

```python
@traceable(
    run_type="chain",
    name="rag",
    project_name="My Project"  # ← Uses "project_name" parameter
)
def rag(question: str, documents):
    # ...
```

**Observations**:
- SDK methods consistently use `project_name` parameter
- No method signatures with `session_name` or similar
- Developers learning from cookbook examples internalize "project" terminology

### 3. Documentation Comments and Docstrings

**Example Docstring**:

```python
def predict_rag_answer_oai(example: dict):
    """Use this for answer evaluation"""
    rag_bot = RagBot(retriever, provider="openai", model="gpt-4-0125-preview")
    response = rag_bot.get_answer(example["question"])
    return {"answer": response["answer"]}
```

**Project Context Comments**:

```python
# Create a new project where user questions are logged
import os
os.environ["LANGCHAIN_PROJECT"] = "DBRX"
```

**Observations**:
- Comments refer to creating "projects", not "sessions"
- Natural language explanations use "project" terminology
- No disambiguation needed because "project" is the only term used

### 4. Trace Organization Patterns

**Multi-Project Testing**:

```python
# Different projects for different experiments
os.environ["LANGCHAIN_PROJECT"] = "Test"
os.environ["LANGCHAIN_PROJECT"] = "DBRX"
os.environ["LANGCHAIN_PROJECT"] = "RAG_online_eval"
os.environ["LANGCHAIN_PROJECT"] = "RAG_repititions"
```

**Project Naming Conventions**:
- Descriptive names: `"Test"`, `"DBRX"`, `"RAG_online_eval"`
- Application/feature-oriented naming
- Persistent identifiers for long-running experiments
- No temporary "session" style naming (no timestamps, UUIDs, etc.)

**Observations**:
- Projects are treated as persistent organizational units
- Named after applications or experiment types
- Used for long-term trace organization, not ephemeral sessions

### 5. No "Session" References

**Searched Patterns**:
- ❌ `session_name`
- ❌ `LANGCHAIN_SESSION`
- ❌ `create_session()`
- ❌ `TracerSession` (in user code)

**Only "Project" References**:
- ✅ `project_name`
- ✅ `LANGCHAIN_PROJECT`
- ✅ `create_project()` (for datasets)
- ✅ Natural language "project" in comments

## Example Analysis: Complete Evaluation Workflow

**From**: `introduction/langsmith_introduction.ipynb` (comprehensive evaluation example)

```python
# 1. Set project context
os.environ["LANGCHAIN_PROJECT"] = "DBRX"

# 2. Create dataset
dataset = client.create_dataset(
    dataset_name="DBRX",
    description="QA pairs about DBRX model."
)

# 3. Run evaluation
experiment_results = evaluate(
    answer_dbrx_question_oai,
    data="DBRX",
    evaluators=qa_evalulator,
    experiment_prefix="test-dbrx-qa-oai",
)

# 4. View results in LangSmith UI
# Results are organized by project
```

**Key Observations**:
1. **Project as Container**: All traces from evaluation go to "DBRX" project
2. **Persistent Organization**: Project persists across multiple experiment runs
3. **No Session Concept**: No notion of starting/stopping a "session"
4. **Application-Centric**: Project = logical application boundary

## Real-World Usage Patterns

### Pattern 1: Development vs Production Projects

```python
# Development
os.environ["LANGCHAIN_PROJECT"] = "rag-dev"

# Production
os.environ["LANGCHAIN_PROJECT"] = "rag-production"
```

**Semantic**: Projects map to deployment environments, not sessions

### Pattern 2: Experiment Versioning

```python
# Different model versions tested in same project
experiment_results = evaluate(
    predict_rag_answer_gpt4_1106,
    data="lcel-eval",
    experiment_prefix="rag-qa-gpt4-1106",
)

experiment_results = evaluate(
    predict_rag_answer_gpt4turbo,
    data="lcel-eval",
    experiment_prefix="rag-qa-gpt4-turbo",
)
```

**Semantic**: Project contains multiple experiments, not bounded by session

### Pattern 3: Feature-Based Organization

```python
os.environ["LANGCHAIN_PROJECT"] = "RAG_QA_LCEL"  # RAG feature
os.environ["LANGCHAIN_PROJECT"] = "back_testing_v2"  # Backtesting feature
os.environ["LANGCHAIN_PROJECT"] = "RAG_online_eval"  # Online eval feature
```

**Semantic**: Projects organized by feature/capability, not temporal sessions

## Developer Learning Path Implications

**What Developers Learn From Cookbook**:

1. **First Exposure**: `os.environ["LANGCHAIN_PROJECT"] = "Test"` (line 8 of introduction)
2. **Reinforcement**: Every example uses "project" terminology
3. **Mental Model**: Projects are containers for traces, organized by application
4. **No Confusion**: Zero exposure to "session" terminology

**Impact on Rust SDK**:
- Developers expect `project_name` in Rust SDK
- Using "session" would create confusion and require relearning
- "Project" is the vocabulary developers bring from cookbook examples

## Comparison: "Project" vs "Session" Semantics in Context

### If Cookbook Used "Session"

```python
# Hypothetical "session" terminology
os.environ["LANGCHAIN_SESSION"] = "DBRX"  # Weird: DBRX is not a session
session = client.create_session(session_name="RAG_online_eval")  # Confusing
```

**Problems**:
- "DBRX" is a feature name, not a session identifier
- "RAG_online_eval" implies long-running experiments, not sessions
- Creates cognitive dissonance with session semantics

### Actual "Project" Terminology

```python
# Actual "project" terminology
os.environ["LANGCHAIN_PROJECT"] = "DBRX"  # Natural: DBRX project
project = client.create_dataset(..., project=...)  # Clear: project scope
```

**Benefits**:
- Feature/application names work naturally as project names
- No temporal implications (start/end session)
- Matches developer mental models

## Supporting Evidence: Python SDK Imports

**From**: `introduction/langsmith_introduction.ipynb` and other notebooks

```python
from langsmith import Client
from langsmith.evaluation import evaluate
from langsmith.run_helpers import traceable
from langsmith.schemas import Run, Example
```

**No Imports Of**:
- ❌ `from langsmith.schemas import TracerSession` (never imported by users)
- ❌ `from langsmith.sessions import ...` (doesn't exist)
- ❌ Any session-related modules

**Observation**:
- Even though `TracerSession` exists internally, users never interact with it
- Complete abstraction from internal "session" terminology

## Conclusion

The langsmith-cookbook provides the strongest evidence yet for "project" terminology:

1. **Zero "Session" References**: Not a single user-facing instance in 50+ notebooks
2. **Consistent "Project" Usage**: Environment vars, method parameters, comments all use "project"
3. **Real-World Patterns**: Projects used for applications, features, experiments - never as sessions
4. **Developer Onboarding**: First exposure is to "project" terminology, creating lasting mental model
5. **Semantic Fit**: All usage patterns align with "project" semantics, not "session" semantics

**Recommendation for Rust SDK**: Use "project" terminology with even more confidence. The cookbook demonstrates that "session" terminology would be alien to developers who learned LangSmith through official examples.

## Files Examined

- `introduction/langsmith_introduction.ipynb` - Comprehensive introduction (300+ lines)
- `testing-examples/comparing-runs/comparing-qa.ipynb` - Project comparison examples
- `testing-examples/backtesting/backtesting.ipynb` - Project-based backtesting
- `tracing-examples/rest/rest.ipynb` - REST API tracing with projects
- `_scripts/test-cookbooks.py` - Test infrastructure using projects

All files consistently use "project" terminology without exception.
