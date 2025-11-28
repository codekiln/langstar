# LangSmith Dataset Management

This document provides complete documentation for managing LangSmith datasets using the `langstar dataset` CLI commands.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [CLI Command Reference](#cli-command-reference)
- [Import/Export Formats](#importexport-formats)
- [Common Workflows](#common-workflows)
- [SDK API Reference](#sdk-api-reference)
- [Best Practices](#best-practices)

## Overview

Datasets in LangSmith are collections of input/output examples used for testing, evaluation, and fine-tuning. The `langstar dataset` commands provide a complete toolkit for managing datasets and their examples through the command line.

### Key Features

- **CRUD Operations**: Create, read, update, and delete datasets
- **Example Management**: Import, export, and list examples within datasets
- **Multiple Formats**: Support for JSONL and CSV formats
- **Bulk Operations**: Efficient bulk import/export of examples
- **Flexible Filtering**: Filter datasets by name, type, and other attributes
- **Output Formats**: Table or JSON output for all commands

### Dataset Types

LangSmith supports three dataset types:

| Type | Description | Use Case |
|------|-------------|----------|
| `kv` | Key-value pairs (default) | Generic input/output mapping |
| `llm` | LLM completion format | Prompt/completion pairs for language models |
| `chat` | Chat message format | Conversational AI and chat applications |

## Prerequisites

Before using dataset commands, ensure you have:

1. **LangSmith API Key**: Set your API key in the environment:
   ```bash
   export LANGSMITH_API_KEY=<your-api-key>
   ```

2. **Langstar CLI**: Install langstar CLI (see main README for installation)

3. **Authentication**: Verify authentication works:
   ```bash
   langstar dataset list
   ```

## CLI Command Reference

### Create a Dataset

Create a new empty dataset.

**Usage:**
```bash
langstar dataset create --name <NAME> [OPTIONS]
```

**Options:**
- `--name <NAME>` - Dataset name (required)
- `--data-type <TYPE>` - Data type: `kv`, `llm`, or `chat` (default: `kv`)
- `--description <DESC>` - Optional description
- `--json` - Output as JSON instead of human-readable format

**Examples:**
```bash
# Create a basic key-value dataset for testing Q&A pairs
langstar dataset create --name "my-qa-dataset" --data-type kv

# Create a chat dataset for training a customer support bot
langstar dataset create --name "customer-support-chats" \
  --data-type chat \
  --description "Training data for customer support bot"

# Create and capture output as JSON
langstar dataset create --name "test-dataset" --data-type llm --json
```

**Output:**
```
Created dataset:
  ID: 12345678-1234-1234-1234-123456789012
  Name: my-qa-dataset
  Type: kv
  Modified: 2025-11-28T12:00:00Z
```

### List Datasets

List all datasets with optional filtering.

**Usage:**
```bash
langstar dataset list [OPTIONS]
```

**Options:**
- `--name <NAME>` - Filter by exact name match
- `--name-contains <SUBSTRING>` - Filter by name substring
- `--data-type <TYPE>` - Filter by data type (`kv`, `llm`, `chat`)
- `-l, --limit <N>` - Maximum number of datasets to return (default: 100)
- `--json` - Output as JSON

**Examples:**
```bash
# List all datasets (default table format)
langstar dataset list

# List datasets with name containing "test"
langstar dataset list --name-contains test

# List only chat datasets
langstar dataset list --data-type chat

# List first 10 datasets as JSON
langstar dataset list --limit 10 --json
```

**Output (table format):**
```
ID       Name              Type  Examples  Description              Modified
12345678 my-qa-dataset     kv    100       Question-answer pairs... 2025-11-28
87654321 customer-chats    chat  50        Customer support data... 2025-11-27

Found 2 datasets
```

### Get Dataset Details

Retrieve detailed information about a specific dataset.

**Usage:**
```bash
langstar dataset get <DATASET_ID> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset

**Options:**
- `--json` - Output as JSON

**Examples:**
```bash
# Get dataset details
langstar dataset get 12345678-1234-1234-1234-123456789012

# Get dataset as JSON
langstar dataset get 12345678-1234-1234-1234-123456789012 --json
```

**Output:**
```
Dataset: my-qa-dataset
  ID: 12345678-1234-1234-1234-123456789012
  Type: kv
  Description: Question-answer pairs for testing
  Examples: 100
  Sessions: 3
  Created: 2025-11-20T10:00:00Z
  Modified: 2025-11-28T12:00:00Z
```

### Update a Dataset

Update dataset name or description.

**Usage:**
```bash
langstar dataset update <DATASET_ID> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset

**Options:**
- `--name <NEW_NAME>` - New name for the dataset
- `--description <NEW_DESC>` - New description
- `--json` - Output as JSON

**Note:** At least one of `--name` or `--description` must be provided.

**Examples:**
```bash
# Update dataset name
langstar dataset update 12345678-1234-1234-1234-123456789012 \
  --name "updated-qa-dataset"

# Update description only
langstar dataset update 12345678-1234-1234-1234-123456789012 \
  --description "Updated: comprehensive Q&A pairs"

# Update both name and description
langstar dataset update 12345678-1234-1234-1234-123456789012 \
  --name "production-qa-dataset" \
  --description "Production-ready Q&A examples"
```

**Output:**
```
Dataset 12345678-1234-1234-1234-123456789012 updated successfully
  Name: updated-qa-dataset
  Description: Updated: comprehensive Q&A pairs
```

### Delete a Dataset

Permanently delete a dataset and all its examples.

**Usage:**
```bash
langstar dataset delete <DATASET_ID> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset

**Options:**
- `-y, --yes` - Skip confirmation prompt (required for actual deletion)

**Examples:**
```bash
# Attempt deletion (shows confirmation message)
langstar dataset delete 12345678-1234-1234-1234-123456789012

# Delete without confirmation
langstar dataset delete 12345678-1234-1234-1234-123456789012 --yes
```

**Output:**
```
Deleted dataset 12345678-1234-1234-1234-123456789012
```

### Import Examples

Import examples from a JSONL or CSV file into an existing dataset.

**Usage:**
```bash
langstar dataset import <DATASET_ID> --file <FILE_PATH> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset to import into

**Options:**
- `--file <PATH>` - Path to the file to import (required)
- `--format <FORMAT>` - File format: `jsonl` or `csv` (optional; inferred from file extension if not specified)

**Examples:**
```bash
# Import from JSONL file (format inferred from .jsonl extension)
langstar dataset import 12345678-1234-1234-1234-123456789012 \
  --file examples.jsonl

# Import from file with ambiguous extension (explicit format required)
langstar dataset import 12345678-1234-1234-1234-123456789012 \
  --file data.txt --format jsonl
```

**Output:**
```
Imported 100 examples to dataset 12345678-1234-1234-1234-123456789012
```

**Notes:**
- Empty lines and lines starting with `#` are skipped in JSONL files
- Invalid records generate warnings but don't stop the import
- Uses bulk create for efficiency

### List Examples

List examples within a dataset.

**Usage:**
```bash
langstar dataset list-examples <DATASET_ID> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset

**Options:**
- `-l, --limit <N>` - Maximum number of examples to return (default: 100)
- `--json` - Output as JSON

**Examples:**
```bash
# List examples in table format
langstar dataset list-examples 12345678-1234-1234-1234-123456789012

# List first 20 examples
langstar dataset list-examples 12345678-1234-1234-1234-123456789012 --limit 20

# Get examples as JSON
langstar dataset list-examples 12345678-1234-1234-1234-123456789012 --json
```

**Output (table format):**
```
ID       Name         Inputs                               Outputs                             Created
abcdef12 Example 1    {"question":"What is 2+2?"}          {"answer":"4"}                      2025-11-28
fedcba21 Example 2    {"question":"What is the capital...  {"answer":"Paris"}                  2025-11-28

Found 2 examples
```

### Export Examples

Export examples from a dataset to a JSONL or CSV file.

**Usage:**
```bash
langstar dataset export <DATASET_ID> --format <FORMAT> [OPTIONS]
```

**Arguments:**
- `<DATASET_ID>` - UUID of the dataset

**Options:**
- `--format <FORMAT>` - Export format: `jsonl` or `csv` (required)
- `-o, --out <PATH>` - Output file path (prints to stdout if not specified)
- `-l, --limit <N>` - Maximum number of examples to export (default: 100)

**Examples:**
```bash
# Export to JSONL file
langstar dataset export 12345678-1234-1234-1234-123456789012 \
  --format jsonl --out backup.jsonl

# Export to CSV
langstar dataset export 12345678-1234-1234-1234-123456789012 \
  --format csv --out data.csv

# Export to stdout (pipe to other commands)
langstar dataset export 12345678-1234-1234-1234-123456789012 \
  --format jsonl | jq '.inputs'

# Export limited number of examples
langstar dataset export 12345678-1234-1234-1234-123456789012 \
  --format jsonl --out sample.jsonl --limit 10
```

**Output:**
```
Exported 100 examples to "backup.jsonl"
```

## Import/Export Formats

### JSONL Format

JSONL (JSON Lines) format stores one JSON object per line. This is the recommended format for programmatic access.

**Structure:**
```json
{"inputs": {"field1": "value1"}, "outputs": {"result": "value"}, "metadata": {"key": "value"}}
{"inputs": {"field1": "value2"}, "outputs": {"result": "value2"}}
```

**Fields:**
- `inputs` (required): JSON object containing input data
- `outputs` (optional): JSON object containing expected output
- `metadata` (optional): JSON object with additional metadata
- `id` (optional): UUID for the example (generated if not provided)

**Example:**
```jsonl
{"inputs": {"question": "What is 2+2?"}, "outputs": {"answer": "4"}}
{"inputs": {"question": "What is the capital of France?"}, "outputs": {"answer": "Paris"}, "metadata": {"category": "geography"}}
```

**Best Practices:**
- One JSON object per line (no pretty-printing)
- Empty lines are ignored
- Lines starting with `#` are treated as comments
- Invalid JSON lines generate warnings but don't stop import

### CSV Format

CSV format is convenient for spreadsheet tools and simple datasets.

**Structure:**
```csv
inputs,outputs,metadata
"{""question"":""What is 2+2?""}","{""answer"":""4""}","{""category"":""math""}"
"{""question"":""What is Paris?""}","{""answer"":""Capital of France""}",""
```

**Columns:**
- `inputs` (required): JSON-encoded string or auto-mapped from columns
- `outputs` (optional): JSON-encoded string
- `metadata` (optional): JSON-encoded string
- `id` (optional): UUID string
- Other columns: Automatically mapped to inputs if no `inputs` column exists

**Auto-Mapping Example:**
```csv
question,category,answer
"What is 2+2?","math",4
"What is Paris?","geography","Capital of France"
```

When no explicit `inputs` or `outputs` columns are present, all non-reserved columns are mapped to `inputs`:
- Row 1 → `inputs`: `{"question": "What is 2+2?", "category": "math", "answer": "4"}`
- Row 2 → `inputs`: `{"question": "What is Paris?", "category": "geography", "answer": "Capital of France"}`

**Best Practices:**
- Use explicit `inputs` and `outputs` columns with JSON strings for complex data
- Use auto-mapping for simple tabular data
- Escape JSON properly when embedding in CSV
- Empty metadata fields are ignored

## Common Workflows

### Create and Populate a Dataset

Complete workflow for creating a new dataset and adding examples:

```bash
# Step 1: Create the dataset
langstar dataset create --name "qa-eval-dataset" --data-type kv --json > dataset.json

# Step 2: Extract dataset ID
DATASET_ID=$(jq -r '.id' dataset.json)
echo "Created dataset: $DATASET_ID"

# Step 3: Prepare examples in JSONL format
cat > examples.jsonl <<'EOF'
{"inputs": {"question": "What is 2+2?"}, "outputs": {"answer": "4"}}
{"inputs": {"question": "What is the capital of France?"}, "outputs": {"answer": "Paris"}}
{"inputs": {"question": "Who wrote Hamlet?"}, "outputs": {"answer": "William Shakespeare"}}
EOF

# Step 4: Import examples
langstar dataset import $DATASET_ID --file examples.jsonl

# Step 5: Verify import
langstar dataset get $DATASET_ID
langstar dataset list-examples $DATASET_ID
```

### Backup and Restore a Dataset

Export a dataset for backup and restore it later:

```bash
# Backup dataset metadata and examples
DATASET_ID="12345678-1234-1234-1234-123456789012"

# Export examples
langstar dataset export $DATASET_ID --format jsonl --out backup-examples.jsonl

# Export metadata (via get command)
langstar dataset get $DATASET_ID --json > backup-metadata.json

# Later: Restore to new dataset
NEW_NAME=$(jq -r '.name' backup-metadata.json)
NEW_TYPE=$(jq -r '.data_type // "kv"' backup-metadata.json)

langstar dataset create --name "$NEW_NAME-restored" \
  --data-type $NEW_TYPE --json > new-dataset.json

NEW_DATASET_ID=$(jq -r '.id' new-dataset.json)
langstar dataset import $NEW_DATASET_ID --file backup-examples.jsonl
```

### Migrate Data Between Formats

Convert between JSONL and CSV formats:

```bash
DATASET_ID="12345678-1234-1234-1234-123456789012"

# Export as JSONL, import to another dataset
langstar dataset export $DATASET_ID --format jsonl --out temp.jsonl
langstar dataset import <other-dataset-id> --file temp.jsonl

# Export as CSV for spreadsheet analysis
langstar dataset export $DATASET_ID --format csv --out analysis.csv
# Edit in spreadsheet...
langstar dataset import $DATASET_ID --file analysis.csv --format csv
```

### Filter and Sample Datasets

Find and work with specific datasets:

```bash
# Find datasets by name pattern
langstar dataset list --name-contains "prod" --json | \
  jq -r '.[] | "\(.id) \(.name) \(.example_count)"'

# Export samples from multiple datasets
for ds_id in $(langstar dataset list --data-type chat --json | jq -r '.[].id'); do
  langstar dataset export $ds_id --format jsonl --out "sample-${ds_id}.jsonl" --limit 10
done

# List all chat datasets with examples
langstar dataset list --data-type chat
```

### Bulk Dataset Operations

Process multiple datasets programmatically:

```bash
# Update descriptions for all datasets matching a pattern
langstar dataset list --name-contains "test" --json | \
  jq -r '.[].id' | \
  while read ds_id; do
    langstar dataset update $ds_id --description "Test dataset - archived 2025-11-28"
  done

# Export all datasets
langstar dataset list --json | \
  jq -r '.[] | "\(.id) \(.name)"' | \
  while read ds_id ds_name; do
    langstar dataset export $ds_id --format jsonl --out "${ds_name}.jsonl"
  done
```

## SDK API Reference

For programmatic access from Rust code, use the SDK client methods.

### Dataset Client Methods

```rust
use langstar_sdk::{LangchainClient, DatasetCreate, DatasetUpdate, DataType};
use uuid::Uuid;

// Initialize client
let client = LangchainClient::new(auth_config)?;

// Create dataset
let request = DatasetCreate {
    name: "my-dataset".to_string(),
    description: Some("Test dataset".to_string()),
    data_type: Some(DataType::Kv),
    ..Default::default()
};
let dataset = client.create_dataset(request).await?;

// List datasets
let params = ListDatasetsParams {
    name_contains: Some("test".to_string()),
    limit: Some(50),
    ..Default::default()
};
let datasets = client.list_datasets(params).await?;

// Get dataset by ID
let dataset_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012")?;
let dataset = client.get_dataset(dataset_id).await?;

// Update dataset
let update = DatasetUpdate {
    name: Some("updated-name".to_string()),
    ..Default::default()
};
let updated = client.update_dataset(dataset_id, update).await?;

// Delete dataset
client.delete_dataset(dataset_id).await?;
```

### Example Client Methods

```rust
use langstar_sdk::{ExampleCreate, ListExamplesParams};
use serde_json::json;

// Create single example
let example = ExampleCreate {
    dataset_id,
    inputs: Some(json!({"question": "What is 2+2?"})),
    outputs: Some(json!({"answer": "4"})),
    ..Default::default()
};
let created = client.create_example(example).await?;

// Bulk create examples
let examples = vec![
    ExampleCreate {
        dataset_id,
        inputs: Some(json!({"question": "Q1"})),
        outputs: Some(json!({"answer": "A1"})),
        ..Default::default()
    },
    // ... more examples
];
let created_examples = client.bulk_create_examples(examples).await?;

// List examples
let params = ListExamplesParams {
    dataset: Some(dataset_id),
    limit: Some(100),
    ..Default::default()
};
let examples = client.list_examples(params).await?;

// Get single example
let example_id = Uuid::parse_str("abcd1234-..."))?;
let example = client.get_example(example_id).await?;

// Delete example
client.delete_example(example_id).await?;
```

### SDK Types

**Dataset Types:**
- `Dataset` - Response type with all fields
- `DatasetCreate` - Request type for creating datasets
- `DatasetUpdate` - Request type for updating datasets
- `DataType` - Enum: `Kv`, `Llm`, `Chat`

**Example Types:**
- `Example` - Response type with all fields
- `ExampleCreate` - Request type for creating examples
- `ExampleUpdate` - Request type for updating examples

See `sdk/src/datasets.rs` for complete type definitions and field documentation.

## Best Practices

### Dataset Organization

- **Naming Convention**: Use descriptive, consistent names (e.g., `project-feature-version`)
- **Descriptions**: Always add descriptions explaining the dataset's purpose
- **Data Types**: Choose the appropriate data type (`kv`, `llm`, `chat`) at creation
- **Versioning**: Include version info in names or metadata for tracking

### Import/Export

- **JSONL for Automation**: Use JSONL format for scripts and programmatic access
- **CSV for Humans**: Use CSV for manual review and spreadsheet editing
- **Validate First**: Test import/export on small samples before bulk operations
- **Backup Regularly**: Export datasets periodically for backup
- **Atomic Operations**: Use bulk operations for efficiency when possible

### Error Handling

- **Check Limits**: Be aware of pagination limits (default 100 items)
- **Handle Failures**: Import operations skip invalid records with warnings
- **Verify Results**: Always verify example counts after import
- **Use --json**: Use JSON output for error handling in scripts

### Security and Privacy

- **API Keys**: Never commit API keys to version control
- **Environment Variables**: Always use environment variables for credentials:
  ```bash
  export LANGSMITH_API_KEY=<your-api-key>
  ```
- **Sensitive Data**: Be cautious with PII in datasets
- **Access Control**: Use LangSmith workspace permissions to control access

### Performance

- **Bulk Operations**: Use bulk create for importing many examples
- **Pagination**: Use `--limit` to control memory usage for large datasets
- **Filtering**: Filter datasets server-side rather than fetching all then filtering locally
- **Streaming**: Export to files rather than stdout for large datasets

---

For more information on LangSmith datasets, see:
- [LangSmith API Documentation](https://api.smith.langchain.com/openapi.json)
- [LangSmith Documentation](https://docs.langchain.com/langsmith/manage-datasets-in-application)
- [Langstar SDK Reference](../sdk/src/datasets.rs)
