# Langstar Evaluations Guide

Complete guide to running evaluations on LangSmith datasets using the `langstar eval` command.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Environment Variables](#environment-variables)
- [Evaluator Types](#evaluator-types)
- [Quick Start](#quick-start)
- [Heuristic Evaluators](#heuristic-evaluators)
- [LLM-as-Judge Evaluators](#llm-as-judge-evaluators)
- [Judge Prompts and Rubrics](#judge-prompts-and-rubrics)
- [Running and Managing Evaluations](#running-and-managing-evaluations)
- [Exporting Results](#exporting-results)
- [Complete Workflow Examples](#complete-workflow-examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

Langstar evaluations allow you to automatically score outputs against datasets using either:

- **Heuristic evaluators**: Fast, deterministic, zero-cost local evaluations
- **LLM-as-judge evaluators**: Model-based scoring using LLM reasoning

Evaluations help you:

- Test model performance on standardized datasets
- Compare different prompts or models
- Monitor quality metrics over time
- Validate outputs against expected results

## Prerequisites

### Required

1. **LangSmith Account**: Sign up at https://smith.langchain.com
2. **LangSmith API Key**: Get from https://smith.langchain.com/settings
3. **Dataset**: Create a dataset with examples to evaluate

### Optional (for LLM-as-judge)

4. **LLM Provider API Key**: Anthropic or OpenAI key for judge models

## Environment Variables

### Core Configuration

| Variable            | Required | Description              | Example                                     |
| ------------------- | -------- | ------------------------ | ------------------------------------------- |
| `LANGSMITH_API_KEY` | ✅ Yes   | LangSmith authentication | `lsv2_pt_...`                               |
| `LANGSMITH_API_URL` | No       | Custom API endpoint      | `https://api.smith.langchain.com` (default) |

### LLM Judge Configuration

| Variable            | Required When         | Description       | Example      |
| ------------------- | --------------------- | ----------------- | ------------ |
| `ANTHROPIC_API_KEY` | Using Anthropic judge | Anthropic API key | `sk-ant-...` |
| `OPENAI_API_KEY`    | Using OpenAI judge    | OpenAI API key    | `sk-...`     |

### Setting Environment Variables

**Linux/macOS:**

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export ANTHROPIC_API_KEY="<your-api-key>"
```

**Windows (PowerShell):**

```powershell
$env:LANGSMITH_API_KEY="<your-api-key>"
$env:ANTHROPIC_API_KEY="<your-api-key>"
```

**Persistent Configuration:**

Create `~/.langstar/config.toml`:

```toml
[langsmith]
api_key = "<your-api-key>"
api_url = "https://api.smith.langchain.com"

[providers.anthropic]
api_key = "<your-anthropic-key>"

[providers.openai]
api_key = "<your-openai-key>"
```

## Evaluator Types

### Heuristic Evaluators (Zero-Cost)

| Evaluator         | Use Case              | Score Range | Example                           |
| ----------------- | --------------------- | ----------- | --------------------------------- |
| `exact-match`     | Exact string equality | 0.0 or 1.0  | Validating exact answers          |
| `contains`        | Substring presence    | 0.0 or 1.0  | Keyword detection                 |
| `regex-match`     | Pattern matching      | 0.0 or 1.0  | Format validation (emails, dates) |
| `json-valid`      | JSON syntax check     | 0.0 or 1.0  | API response validation           |
| `string-distance` | Fuzzy matching        | 0.0 to 1.0  | Typo tolerance, similarity        |

### LLM-as-Judge Evaluators (API Calls Required)

| Feature            | Categorical                   | Continuous                  |
| ------------------ | ----------------------------- | --------------------------- |
| **Score Type**     | Predefined choices            | Numeric range               |
| **Use Case**       | Pass/Fail, Rating scales      | Quality scores, percentages |
| **Example Scores** | Y/N, Poor/Fair/Good/Excellent | 0.0-1.0, 1-10               |
| **Cost**           | Per API call                  | Per API call                |

## Quick Start

### Step 1: Create a Dataset

```bash
# Create dataset
langstar dataset create --name "qa-validation" --data-type kv

# Add examples (prepare examples.jsonl)
langstar dataset import <dataset-id> --file examples.jsonl
```

**Example `examples.jsonl`:**

```jsonl
{"input": {"question": "What is 2+2?"}, "output": {"answer": "4"}}
{"input": {"question": "Capital of France?"}, "output": {"answer": "Paris"}}
{"input": {"question": "Largest planet?"}, "output": {"answer": "Jupiter"}}
```

### Step 2: Create an Evaluation

**Heuristic evaluator:**

```bash
langstar eval create \
  --name "exact-match-qa" \
  --dataset "qa-validation" \
  --evaluator exact-match
```

**LLM judge:**

```bash
langstar eval create \
  --name "quality-check" \
  --dataset "qa-validation" \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubric.txt" \
  --score-type categorical \
  --score-choices "Incorrect,Partially Correct,Correct" \
  --include-reasoning
```

### Step 3: Run the Evaluation

```bash
# Get evaluation ID from previous command
langstar eval run <eval-id>

# Or preview with first 5 examples
langstar eval run <eval-id> --preview 5
```

### Step 4: View Results

```bash
# Get evaluation details
langstar eval get <eval-id>

# Export to CSV
langstar eval export <eval-id> --format csv --output results.csv
```

## Heuristic Evaluators

### Exact Match

Tests for exact string equality (case-sensitive).

**Use Cases:**

- Multiple choice answers
- Exact code output validation
- Binary classification

**Example:**

```bash
langstar eval create \
  --name "exact-answer-check" \
  --dataset "trivia-dataset" \
  --evaluator exact-match
```

**Score Logic:**

- `1.0` if output exactly matches expected
- `0.0` otherwise

### Contains

Checks if output contains expected substring (case-sensitive).

**Use Cases:**

- Keyword presence detection
- Required phrase validation
- Content filtering

**Example:**

```bash
langstar eval create \
  --name "keyword-presence" \
  --dataset "content-moderation" \
  --evaluator contains
```

**Score Logic:**

- `1.0` if expected substring found in output
- `0.0` otherwise

### Regex Match

Validates output against regular expression pattern.

**Use Cases:**

- Email/phone format validation
- Date/time format checking
- Structured output verification

**Example:**

```bash
langstar eval create \
  --name "email-format-check" \
  --dataset "email-extraction" \
  --evaluator regex-match
```

**Score Logic:**

- `1.0` if output matches regex pattern
- `0.0` otherwise
- `Error` if regex pattern is invalid

### JSON Valid

Validates that output is syntactically valid JSON.

**Use Cases:**

- API response validation
- Structured data output
- JSON generation tasks

**Example:**

```bash
langstar eval create \
  --name "json-output-validation" \
  --dataset "api-responses" \
  --evaluator json-valid
```

**Score Logic:**

- `1.0` if output is valid JSON
- `0.0` otherwise

### String Distance

Normalized Levenshtein distance for fuzzy matching.

**Use Cases:**

- Typo tolerance
- Approximate matching
- Similarity scoring

**Example:**

```bash
langstar eval create \
  --name "fuzzy-matching" \
  --dataset "autocomplete-suggestions" \
  --evaluator string-distance
```

**Score Logic:**

- `1.0` = identical strings
- `0.0` = completely different
- Range: Continuous between 0.0 and 1.0

**Calculation:**

```
similarity = 1.0 - (levenshtein_distance / max_string_length)
```

## LLM-as-Judge Evaluators

### Overview

LLM-as-judge uses a language model to evaluate outputs based on a custom rubric or prompt.

**Key Components:**

1. **Judge Model**: The LLM that will evaluate (e.g., Claude, GPT-4)
2. **Judge Provider**: API provider (anthropic, openai)
3. **Judge Prompt**: Instructions/rubric for evaluation
4. **Score Type**: Categorical or continuous scoring
5. **Reasoning**: Optional explanation of score

### Categorical Scoring

Use when you want predefined rating categories.

**Example: Binary Classification**

```bash
langstar eval create \
  --name "pass-fail-check" \
  --dataset "code-outputs" \
  --evaluator llm-judge \
  --judge-model "gpt-4o" \
  --judge-provider "openai" \
  --judge-prompt-file "./rubrics/pass-fail.txt" \
  --score-type categorical \
  --score-choices "Fail,Pass" \
  --include-reasoning
```

**Example Rubric (`rubrics/pass-fail.txt`):**

```
Evaluate whether the code output is correct.

Criteria:
- PASS: Output matches expected result exactly
- FAIL: Output differs from expected result

Return either "Pass" or "Fail".
```

**Example: Multi-Level Rating**

```bash
langstar eval create \
  --name "quality-rating" \
  --dataset "customer-responses" \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubrics/quality-scale.txt" \
  --score-type categorical \
  --score-choices "Poor,Fair,Good,Excellent" \
  --include-reasoning
```

**Example Rubric (`rubrics/quality-scale.txt`):**

```
Evaluate the quality of this customer service response.

Rating Scale:
- POOR: Unhelpful, rude, or incorrect information
- FAIR: Partially helpful but missing key information
- GOOD: Helpful and accurate with minor issues
- EXCELLENT: Exceptionally helpful, accurate, and empathetic

Return one of: Poor, Fair, Good, Excellent
```

### Continuous Scoring

Use when you want numeric scores on a continuous scale.

**Example: 0-1 Scale**

```bash
langstar eval create \
  --name "relevance-score" \
  --dataset "search-results" \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubrics/relevance.txt" \
  --score-type continuous \
  --score-min 0.0 \
  --score-max 1.0 \
  --include-reasoning
```

**Example Rubric (`rubrics/relevance.txt`):**

```
Evaluate how relevant this search result is to the query.

Scoring:
- 0.0 = Completely irrelevant
- 0.5 = Somewhat relevant
- 1.0 = Perfectly relevant

Return a score between 0.0 and 1.0 (use decimals).
```

**Example: 1-10 Scale**

```bash
langstar eval create \
  --name "content-quality" \
  --dataset "article-generation" \
  --evaluator llm-judge \
  --judge-model "gpt-4" \
  --judge-provider "openai" \
  --judge-prompt-file "./rubrics/article-quality.txt" \
  --score-type continuous \
  --score-min 1.0 \
  --score-max 10.0
```

**Example Rubric (`rubrics/article-quality.txt`):**

```
Rate the quality of this generated article on a scale of 1-10.

Criteria:
- Grammar and spelling (0-3 points)
- Clarity and coherence (0-3 points)
- Factual accuracy (0-4 points)

Return a number between 1 and 10.
```

## Judge Prompts and Rubrics

### Best Practices

1. **Be Specific**: Clearly define evaluation criteria
2. **Use Examples**: Show what different scores look like
3. **Specify Format**: Tell the judge exactly how to respond
4. **Include Context**: Provide any necessary background information
5. **Test First**: Use `--preview` mode to validate rubric

### Rubric Template

```
[TASK DESCRIPTION]
Briefly describe what you're evaluating.

[EVALUATION CRITERIA]
List specific criteria the judge should consider:
- Criterion 1: Description
- Criterion 2: Description
- Criterion 3: Description

[SCORING GUIDANCE]
For categorical: Define each category clearly
For continuous: Define the scale with anchors

[OUTPUT FORMAT]
Specify exactly how the judge should respond:
- For categorical: "Return one of: [choices]"
- For continuous: "Return a number between X and Y"

[OPTIONAL: EXAMPLES]
Show 1-2 examples of inputs and expected scores.
```

### Example Rubrics

**Factual Accuracy (Categorical):**

```
Evaluate whether the answer is factually accurate.

Check against your knowledge:
- INCORRECT: Contains factual errors
- PARTIALLY_CORRECT: Mostly accurate with minor errors
- CORRECT: Completely accurate

Return one of: Incorrect, Partially_Correct, Correct
```

**Helpfulness (Continuous):**

```
Rate how helpful this response is to the user (0.0 to 1.0).

Criteria:
- Directly addresses the question (0-0.4)
- Provides actionable information (0-0.3)
- Clear and easy to understand (0-0.3)

Return a score between 0.0 and 1.0.
```

**Toxicity Detection (Categorical):**

```
Classify the toxicity level of this text.

Categories:
- SAFE: No harmful content
- MILDLY_TOXIC: Minor offensive language
- TOXIC: Clearly harmful or offensive content

Return one of: Safe, Mildly_Toxic, Toxic
```

## Running and Managing Evaluations

### Preview Mode

Test your configuration on a subset of examples before running full evaluation.

```bash
# Preview first 10 examples
langstar eval run <eval-id> --preview 10

# Good for:
# - Testing expensive LLM judge configurations
# - Validating rubric clarity
# - Debugging evaluation logic
```

### Dry Run

Validate configuration without executing evaluation.

```bash
langstar eval run <eval-id> --dry-run

# Checks:
# - Dataset exists and is accessible
# - Evaluation configuration is valid
# - Required API keys are present
# - Judge prompts are readable
```

### Listing Evaluations

```bash
# List all evaluations
langstar eval list

# Filter by name
langstar eval list --name "quality"

# Filter by dataset
langstar eval list --dataset <dataset-id>

# Filter by evaluator type
langstar eval list --evaluator-type llm-judge

# Limit results
langstar eval list --limit 20
```

### Getting Evaluation Details

```bash
# View evaluation configuration and status
langstar eval get <eval-id>

# JSON output for scripting
langstar eval get <eval-id> --json
```

## Exporting Results

### Export Formats

**CSV (Tabular):**

```bash
langstar eval export <eval-id> --format csv --output results.csv
```

**Output:**

```csv
example_id,key,score,value,comment
550e8400-...,accuracy,1.0,Pass,Correct answer
660f9511-...,accuracy,0.0,Fail,Wrong answer
```

**JSONL (JSON Lines):**

```bash
langstar eval export <eval-id> --format jsonl --output results.jsonl
```

**Output:**

```jsonl
{"example_id":"550e8400-...","key":"accuracy","score":1.0,"value":"Pass","comment":"Correct answer"}
{"example_id":"660f9511-...","key":"accuracy","score":0.0,"value":"Fail","comment":"Wrong answer"}
```

**JSON (Full Document):**

```bash
langstar eval export <eval-id> --format json --output results.json --include-metadata
```

### Analyzing Results

**Using CSV with spreadsheet tools:**

```bash
langstar eval export <eval-id> --format csv --output results.csv
# Open in Excel, Google Sheets, etc.
```

**Using jq for JSON analysis:**

```bash
# Average score
langstar eval export <eval-id> --format jsonl | \
  jq -s 'map(.score) | add/length'

# Count by value
langstar eval export <eval-id> --format jsonl | \
  jq -s 'group_by(.value) | map({value: .[0].value, count: length})'

# Filter failing examples
langstar eval export <eval-id> --format jsonl | \
  jq 'select(.score < 0.5)'
```

## Complete Workflow Examples

### Example 1: Exact Match Validation

**Scenario:** Validate Q&A bot answers against expected responses.

```bash
# 1. Create dataset
langstar dataset create --name "qa-test" --data-type kv
DATASET_ID="<returned-id>"

# 2. Import examples
cat > examples.jsonl << EOF
{"input":{"question":"What is 2+2?"},"output":{"answer":"4"}}
{"input":{"question":"Capital of France?"},"output":{"answer":"Paris"}}
{"input":{"question":"Who wrote Hamlet?"},"output":{"answer":"Shakespeare"}}
EOF

langstar dataset import $DATASET_ID --file examples.jsonl

# 3. Create evaluation
langstar eval create \
  --name "qa-exact-match" \
  --dataset $DATASET_ID \
  --evaluator exact-match \
  --json

# 4. Run evaluation
EVAL_ID="<returned-eval-id>"
langstar eval run $EVAL_ID

# 5. View and export results
langstar eval get $EVAL_ID
langstar eval export $EVAL_ID --format csv --output qa_results.csv
```

### Example 2: LLM Judge for Customer Service

**Scenario:** Evaluate customer service response quality.

```bash
# 1. Prepare dataset (already exists)
DATASET_ID="customer-service-qa"

# 2. Create rubric
cat > ./rubrics/cs-quality.txt << EOF
Evaluate this customer service response for quality.

Criteria:
- POOR: Unhelpful, rude, or incorrect
- FAIR: Partially helpful but lacking
- GOOD: Helpful and professional
- EXCELLENT: Exceptionally helpful and empathetic

Return one of: Poor, Fair, Good, Excellent
EOF

# 3. Create evaluation
langstar eval create \
  --name "cs-quality-rating" \
  --dataset $DATASET_ID \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubrics/cs-quality.txt" \
  --score-type categorical \
  --score-choices "Poor,Fair,Good,Excellent" \
  --include-reasoning \
  --json

# 4. Preview first 5 examples
EVAL_ID="<returned-eval-id>"
langstar eval run $EVAL_ID --preview 5

# 5. Run full evaluation
langstar eval run $EVAL_ID

# 6. Export with reasoning
langstar eval export $EVAL_ID \
  --format json \
  --output cs_quality_results.json \
  --include-metadata
```

### Example 3: Continuous Scoring for Relevance

**Scenario:** Score search result relevance on 0-1 scale.

```bash
# 1. Create dataset (search queries + results)
langstar dataset create --name "search-relevance" --data-type kv
DATASET_ID="<returned-id>"

# 2. Create rubric
cat > ./rubrics/relevance.txt << EOF
Rate how relevant this search result is to the query (0.0 to 1.0).

Scoring:
- 0.0-0.3: Not relevant
- 0.4-0.6: Somewhat relevant
- 0.7-0.9: Relevant
- 1.0: Perfectly relevant

Return a decimal number between 0.0 and 1.0.
EOF

# 3. Create evaluation
langstar eval create \
  --name "search-relevance-score" \
  --dataset $DATASET_ID \
  --evaluator llm-judge \
  --judge-model "gpt-4o" \
  --judge-provider "openai" \
  --judge-prompt-file "./rubrics/relevance.txt" \
  --score-type continuous \
  --score-min 0.0 \
  --score-max 1.0 \
  --include-reasoning

# 4. Run and analyze
EVAL_ID="<returned-eval-id>"
langstar eval run $EVAL_ID

# Export and calculate average relevance
langstar eval export $EVAL_ID --format jsonl | \
  jq -s 'map(.score) | add/length'
```

## Best Practices

### Choosing Evaluator Types

**Use Heuristic Evaluators When:**

- ✅ Expected output is well-defined
- ✅ Cost is a concern (zero cost)
- ✅ Speed is critical (instant)
- ✅ Determinism is required

**Use LLM Judge When:**

- ✅ Subjective quality assessment needed
- ✅ Complex reasoning required
- ✅ Natural language understanding needed
- ✅ Human-like judgment desired

### Cost Optimization

**For LLM Judges:**

1. **Use Preview Mode**: Test on subset before full run
   ```bash
   langstar eval run <eval-id> --preview 10
   ```

2. **Choose Efficient Models**: Consider cost vs. quality tradeoffs
   - High quality: `claude-3-5-sonnet-20241022`, `gpt-4o`
   - Cost-efficient: `gpt-4o-mini`, `claude-3-haiku`

3. **Batch Similar Evaluations**: Group similar datasets/prompts

### Rubric Design

**Do:**

- ✅ Be explicit about scoring criteria
- ✅ Provide clear examples
- ✅ Use consistent terminology
- ✅ Test with preview mode
- ✅ Include edge case handling

**Don't:**

- ❌ Use ambiguous language
- ❌ Mix multiple evaluation dimensions without structure
- ❌ Assume judge has external context
- ❌ Over-complicate scoring scales

### Evaluation Management

**Naming Conventions:**

- Use descriptive names: `qa-exact-match-v1`, `cs-quality-claude-sonnet`
- Include version numbers for iterations
- Tag evaluations by purpose: `test`, `prod`, `experiment`

**Version Control:**

- Store judge prompts in version control
- Document rubric changes
- Track evaluation IDs and configurations

## Troubleshooting

### Common Issues

**"API key not found"**

```bash
# Solution: Set required environment variable
export LANGSMITH_API_KEY="<your-key>"
export ANTHROPIC_API_KEY="<your-key>"  # For Anthropic judges
```

**"Dataset not found"**

```bash
# Solution: Verify dataset exists and ID is correct
langstar dataset list
langstar dataset get <dataset-id>
```

**"Judge model rate limit exceeded"**

```bash
# Solution: Use preview mode to reduce calls
langstar eval run <eval-id> --preview 10

# Or use a more available model
--judge-model "gpt-4o-mini"  # Instead of gpt-4
```

**"Rubric file not found"**

```bash
# Solution: Use absolute path or verify file exists
langstar eval create \
  --judge-prompt-file "$(pwd)/rubrics/quality.txt"  # Absolute path
```

**"Invalid score type for choices"**

```bash
# Solution: Categorical requires --score-choices
langstar eval create \
  --score-type categorical \
  --score-choices "Fail,Pass"  # Required for categorical
```

### Debugging Evaluations

**1. Dry Run First:**

```bash
langstar eval run <eval-id> --dry-run
```

**2. Preview with Small Subset:**

```bash
langstar eval run <eval-id> --preview 3
```

**3. Check JSON Output:**

```bash
langstar eval get <eval-id> --json | jq
```

**4. Examine Individual Results:**

```bash
langstar eval export <eval-id> --format jsonl | head -5
```

### Getting Help

- **Documentation**: https://github.com/codekiln/langstar
- **Issues**: https://github.com/codekiln/langstar/issues
- **LangSmith Docs**: https://docs.smith.langchain.com

## See Also

- [LangSmith Datasets Guide](./datasets.md)
- [Configuration Reference](./configuration.md)
- [Evaluation Implementation Plan](./evals-implementation-plan.md)
- [CLI Reference](../README.md)
