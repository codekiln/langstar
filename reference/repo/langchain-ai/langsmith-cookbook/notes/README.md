# langsmith-cookbook

## Repository Information

- **Repository**: [langchain-ai/langsmith-cookbook](https://github.com/langchain-ai/langsmith-cookbook)
- **Date Created**: 2025-11-28
- **Cloned to**: `/workspace/reference/repo/langchain-ai/langsmith-cookbook/code`
- **Official Docs**: https://docs.smith.langchain.com/

## Purpose

This repository provides **usage patterns and real-world examples** for LangSmith features. It's our reference for understanding:
- How developers **use** LangSmith APIs in practice
- Common patterns and best practices
- End-to-end workflows for tracing, testing, and evaluation
- What data structures and responses to expect

Use this cookbook to:
1. **Understand user workflows** before implementing SDK methods
2. **Validate API design** against real-world usage patterns
3. **Create test cases** based on proven examples
4. **Write CLI help text** that matches user mental models

## Repository Structure

The cookbook is organized by use case:

```
langsmith-cookbook/
├── tracing-examples/          # Traces & runs - relevant for ls-runs-query
├── testing-examples/          # Datasets & evaluation - relevant for ls-datasets
├── feedback-examples/         # Feedback & annotation - relevant for ls-annotation-queues
├── optimization/              # Performance tuning patterns
├── fine-tuning-examples/      # Model fine-tuning workflows
├── hub-examples/              # LangChain Hub prompt management
└── typescript-testing-examples/  # JS/TS patterns (if we support TS later)
```

## Key Examples by Milestone

### For `ls-runs-query` (Milestone 3)

**Must-read examples:**
- `tracing-examples/traceable/tracing_without_langchain.ipynb` - Core tracing concepts
- `tracing-examples/rest/rest.ipynb` - **Direct REST API usage** (shows request/response formats)
- `tracing-examples/nesting-tools/nest_runs_within_tools.ipynb` - Understanding run hierarchies

**Key learnings:**
- Run data structure (inputs, outputs, metadata, parent-child relationships)
- Filtering and query patterns
- How users debug traces in practice

### For `ls-datasets` (Milestone 5)

**Must-read examples:**
- `testing-examples/using-fixed-sources/using_fixed_sources.ipynb` - Dataset creation patterns
- `testing-examples/dynamic-data/testing_dynamic_data.ipynb` - Datasets with changing data
- `testing-examples/backtesting/backtesting.ipynb` - **Converting runs to datasets** (key workflow!)
- `testing-examples/export-test-to-csv/export-test-to-csv.ipynb` - Data export patterns
- `testing-examples/download-feedback-and-examples/download_example.ipynb` - Fetching examples

**Key learnings:**
- Dataset types (chat, key-value, custom)
- Example structure and metadata
- Pagination and filtering patterns
- Dataset versioning and as_of queries

### For `ls-annotation-queues` (Milestone 4)

**Must-read examples:**
- `feedback-examples/algorithmic-feedback/algorithmic_feedback.ipynb` - Automated feedback pipelines
- `feedback-examples/realtime-algorithmic-feedback/realtime_feedback.ipynb` - Real-time feedback
- `feedback-examples/streamlit/` - **Interactive feedback capture** (shows UX patterns)

**Key learnings:**
- Feedback vs annotation workflows
- How annotation queues fit into evaluation lifecycle
- User expectations for review/labeling interfaces

## How to Use This Reference

### When Implementing SDK Features

1. **Start with the cookbook example** for your feature area
2. **Run the notebooks** (if they don't require external services)
3. **Identify the API calls** being made and data structures used
4. **Map to OpenAPI spec** to understand the canonical API contract
5. **Implement SDK methods** that support these usage patterns

### Example Workflow: Implementing Dataset Listing

```bash
# 1. Read the cookbook example
cd reference/repo/langchain-ai/langsmith-cookbook/code/
jupyter notebook testing-examples/backtesting/backtesting.ipynb

# 2. Identify the SDK calls
# Look for: client.list_datasets(...), client.create_dataset(...)

# 3. Understand parameters and return types
# Note: filtering options, pagination, metadata structure

# 4. Cross-reference with OpenAPI spec
# Check: /reference/api-specs/langsmith_openapi.yaml

# 5. Implement in Rust SDK
# Match the Python SDK's ergonomics in Rust idioms
```

## Quick Reference Links

**Main README**: `code/README.md` - Full catalog of all examples

**REST API Example**: `code/tracing-examples/rest/rest.ipynb` - Shows raw API calls (bypasses SDK)

**OpenAPI Spec Reference**: Mentioned in README - LangSmith SDK repository contains `openapi/openapi.yaml`

## Notes & Findings

### Pattern: REST API Structure

The REST API example (`tracing-examples/rest/rest.ipynb`) is particularly valuable because it shows:
- Raw HTTP requests without SDK abstraction
- Exact request/response formats
- Authentication patterns
- Error handling expectations

### Pattern: Dataset Workflows

Common dataset workflows from examples:
1. **Create** → **Add Examples** → **Run Evaluation** → **Iterate**
2. **Production Runs** → **Convert to Dataset** → **Regression Testing** (backtesting pattern)
3. **Export** → **Analyze Offline** → **Re-import** (data portability)

### Pattern: Tracing & Debugging

Debugging workflow shown in examples:
1. **Trace** the execution
2. **Fetch** the trace by ID or filter criteria
3. **Analyze** the nested run structure
4. **Add** to dataset for regression testing if needed

## Additional Resources

- **LangSmith Docs**: https://docs.smith.langchain.com/
- **LangSmith Python SDK**: https://github.com/langchain-ai/langsmith-sdk
- **OpenAPI Spec**: Check LangSmith SDK repo for `openapi/openapi.yaml`

