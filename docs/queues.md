# Annotation Queues

LangSmith annotation queues organize runs (traces) for human review and annotation. Use `langstar queue` commands to create queues, add runs for review, and manage the annotation workflow.

## Quickstart

### Prerequisites

Set your LangSmith API key:

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

### Create a Queue

```bash
langstar queue create --name "Production Review" --description "Review production errors"
```

Output:

```
Created queue:
  ID: 12345678-1234-1234-1234-123456789012
  Name: Production Review
  Type: Single
  Created: 2025-11-28T10:00:00Z
```

### List Queues

```bash
langstar queue list
```

Output:

```
ID        Name               Type    Description                   Created
12345678  Production Review  single  Review production errors      2025-11-28
```

### Add Runs to a Queue

```bash
# Add single run
langstar queue add-runs <queue-id> <run-id>

# Add multiple runs
langstar queue add-runs <queue-id> <run-id-1> <run-id-2> <run-id-3>
```

### View Queue Items

```bash
langstar queue items <queue-id>
```

---

## Command Reference

### `langstar queue list`

List all annotation queues accessible to your API key.

```bash
langstar queue list [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--name <NAME>` | Filter by exact name match |
| `--name-contains <SUBSTRING>` | Filter by name substring |
| `-l, --limit <N>` | Maximum queues to return (default: 100) |
| `--json` | Output as JSON |

**Examples:**

```bash
# List all queues
langstar queue list

# Filter by name
langstar queue list --name-contains "review"

# JSON output for scripting
langstar queue list --json
```

---

### `langstar queue create`

Create a new annotation queue.

```bash
langstar queue create --name <NAME> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--name <NAME>` | Queue name (required) |
| `--description <DESC>` | Queue description |
| `--rubric <INSTRUCTIONS>` | Rubric instructions for annotators |
| `--queue-type <TYPE>` | Queue type: `single` or `pairwise` (default: single) |
| `--json` | Output as JSON |

**Examples:**

```bash
# Basic queue
langstar queue create --name "Error Triage"

# With description and rubric
langstar queue create \
  --name "Quality Review" \
  --description "Review LLM outputs for accuracy" \
  --rubric "Rate responses on accuracy (1-5) and helpfulness (1-5)"

# Pairwise comparison queue
langstar queue create \
  --name "A/B Comparison" \
  --queue-type pairwise \
  --description "Compare model outputs side-by-side"
```

---

### `langstar queue get`

Get detailed information about a specific queue.

```bash
langstar queue get <QUEUE_ID> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

**Example:**

```bash
langstar queue get 12345678-1234-1234-1234-123456789012
```

Output:

```
Queue: Production Review
  ID: 12345678-1234-1234-1234-123456789012
  Type: Single
  Description: Review production errors
  Rubric: Rate accuracy and helpfulness
  Created: 2025-11-28T10:00:00Z
  Updated: 2025-11-28T10:00:00Z
```

---

### `langstar queue update`

Update an existing annotation queue.

```bash
langstar queue update <QUEUE_ID> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--name <NAME>` | New queue name |
| `--description <DESC>` | New description |
| `--rubric <INSTRUCTIONS>` | New rubric instructions |
| `--json` | Output as JSON |

**Example:**

```bash
langstar queue update 12345678-... --name "Updated Queue Name" --description "New description"
```

---

### `langstar queue delete`

Delete an annotation queue.

```bash
langstar queue delete <QUEUE_ID> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation prompt |

**Example:**

```bash
# With confirmation prompt
langstar queue delete 12345678-1234-1234-1234-123456789012

# Skip confirmation
langstar queue delete 12345678-1234-1234-1234-123456789012 --force
```

---

### `langstar queue add-runs`

Add runs (traces) to an annotation queue for review.

```bash
langstar queue add-runs <QUEUE_ID> <RUN_IDS>... [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--runs-file <FILE>` | File containing run IDs (one per line) |

**Examples:**

```bash
# Add single run
langstar queue add-runs 12345678-... abcdef01-1234-1234-1234-123456789012

# Add multiple runs
langstar queue add-runs 12345678-... run-id-1 run-id-2 run-id-3

# Add runs from file
langstar queue add-runs 12345678-... --runs-file runs.txt
```

**runs.txt format:**

```
# Comments are ignored
abcdef01-1234-1234-1234-123456789012
abcdef02-1234-1234-1234-123456789012
abcdef03-1234-1234-1234-123456789012
```

---

### `langstar queue remove-run`

Remove a run from an annotation queue.

```bash
langstar queue remove-run <QUEUE_ID> <RUN_ID>
```

**Example:**

```bash
langstar queue remove-run 12345678-... abcdef01-...
```

---

### `langstar queue items`

List runs currently in an annotation queue.

```bash
langstar queue items <QUEUE_ID> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `-l, --limit <N>` | Maximum items to return (default: 100) |
| `--json` | Output as JSON |

**Example:**

```bash
langstar queue items 12345678-1234-1234-1234-123456789012 --limit 50
```

Output:

```
Index  Run ID    Name          Status   Added
0      abcdef01  ChatOpenAI    success  2025-11-28
1      abcdef02  RAGChain      error    2025-11-28
2      abcdef03  AgentExecutor success  2025-11-28

Found 3 items in queue
```

---

## CI/CD Integration

### GitHub Actions: Triage Failing Runs

Automatically add failing runs to an annotation queue for human review:

```yaml
name: Triage Failing Traces

on:
  workflow_dispatch:
    inputs:
      queue_name:
        description: 'Annotation queue name'
        required: true
        default: 'CI Review'

jobs:
  triage:
    runs-on: ubuntu-latest
    steps:
      - name: Install langstar
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf \
            https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash

      - name: Find queue ID
        id: queue
        env:
          LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
        run: |
          QUEUE_ID=$(langstar queue list --json | \
            jq -r '.[] | select(.name == "${{ inputs.queue_name }}") | .id')
          echo "queue_id=$QUEUE_ID" >> $GITHUB_OUTPUT

      - name: Query error runs and add to queue
        env:
          LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
        run: |
          # Get recent error runs
          langstar runs query --errors-only --limit 10 --output json | \
            jq -r '.[].id' > error_runs.txt

          # Add to annotation queue
          if [ -s error_runs.txt ]; then
            langstar queue add-runs ${{ steps.queue.outputs.queue_id }} --runs-file error_runs.txt
            echo "Added $(wc -l < error_runs.txt) runs to queue"
          else
            echo "No error runs found"
          fi
```

### GitHub Actions: Create Queue on Deployment

```yaml
- name: Create annotation queue for deployment
  env:
    LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
  run: |
    langstar queue create \
      --name "Deploy Review - ${{ github.sha }}" \
      --description "Review traces from deployment ${{ github.sha }}"
```

### Bulk Import from File

Create a script to import runs from a newline-delimited file:

```bash
#!/bin/bash
# import-runs.sh

QUEUE_ID=$1
RUNS_FILE=$2

if [ -z "$QUEUE_ID" ] || [ -z "$RUNS_FILE" ]; then
    echo "Usage: $0 <queue-id> <runs-file>"
    exit 1
fi

langstar queue add-runs "$QUEUE_ID" --runs-file "$RUNS_FILE"
```

Usage:

```bash
./import-runs.sh 12345678-... runs_to_review.txt
```

---

## Rubrics

Rubrics provide instructions to annotators reviewing runs in a queue. Use the `--rubric` flag when creating or updating a queue:

```bash
langstar queue create --name "Quality Review" \
  --rubric "Rate each response on:
- Accuracy (1-5): Is the information correct?
- Helpfulness (1-5): Does it address the user's question?
- Tone (1-5): Is the response professional and appropriate?"
```

The rubric text is displayed to annotators in the LangSmith UI when they review items in the queue.

### Rubric Best Practices

- **Be specific**: Define clear criteria for each rating dimension
- **Use consistent scales**: Stick to a consistent rating scale (e.g., 1-5)
- **Provide examples**: Include examples of good/poor responses when possible
- **Keep it concise**: Annotators should be able to quickly reference the rubric

### Structured Rubric Items (SDK Only)

The LangSmith API supports structured rubric items with feedback keys and score descriptions. This advanced feature is available through the SDK but not currently exposed in the CLI:

```rust
use langstar_sdk::annotation_queues::{
    CreateAnnotationQueueRequest, AnnotationQueueRubricItem
};

let request = CreateAnnotationQueueRequest {
    name: "Structured Review".to_string(),
    rubric_instructions: Some("General guidelines here".to_string()),
    rubric_items: Some(vec![
        AnnotationQueueRubricItem {
            feedback_key: "accuracy".to_string(),
            description: Some("How accurate is the response?".to_string()),
            score_descriptions: Some(serde_json::json!({
                "1": "Completely incorrect",
                "3": "Partially correct",
                "5": "Fully accurate"
            })),
            value_descriptions: None,
        },
    ]),
    ..Default::default()
};
```

---

## Common Workflows

### Error Triage Pipeline

1. **Create a queue for error review:**
   ```bash
   langstar queue create --name "Error Triage" \
     --description "Production errors requiring investigation" \
     --rubric "Investigate root cause. Tag as: bug, data_issue, or expected"
   ```

2. **Query recent errors and add to queue:**
   ```bash
   # Save error run IDs to file
   langstar runs query --errors-only --limit 100 --output json | \
     jq -r '.[].id' > errors.txt

   # Add to queue
   langstar queue add-runs <queue-id> --runs-file errors.txt
   ```

3. **Review in LangSmith UI:**
   Navigate to the Annotation Queues section in LangSmith to review and annotate runs.

### A/B Testing Review

1. **Create pairwise comparison queue:**
   ```bash
   langstar queue create --name "Model Comparison" \
     --queue-type pairwise \
     --description "Compare GPT-4 vs Claude outputs" \
     --rubric "Select the better response based on accuracy and helpfulness"
   ```

2. **Add run pairs for comparison:**
   ```bash
   # Add runs from both model variants
   langstar queue add-runs <queue-id> <gpt4-run-id> <claude-run-id>
   ```

---

## SDK Usage (Rust)

For programmatic access to annotation queues, use the `langstar-sdk` crate:

```rust
use langstar_sdk::{
    LangchainClient, AuthConfig,
    CreateAnnotationQueueRequest, ListAnnotationQueuesParams, QueueType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let auth = AuthConfig::from_env()?;
    let client = LangchainClient::new(auth)?;

    // Create a queue
    let request = CreateAnnotationQueueRequest {
        name: "SDK Review Queue".to_string(),
        description: Some("Created via SDK".to_string()),
        queue_type: Some(QueueType::Single),
        rubric_instructions: Some("Rate accuracy 1-5".to_string()),
        ..Default::default()
    };
    let queue = client.create_annotation_queue(request).await?;
    println!("Created queue: {}", queue.base.id);

    // List queues
    let params = ListAnnotationQueuesParams {
        name_contains: Some("Review".to_string()),
        limit: Some(10),
        ..Default::default()
    };
    let queues = client.list_annotation_queues(params).await?;
    println!("Found {} queues", queues.len());

    // Add runs to queue
    let run_ids = vec![
        "abcdef01-1234-1234-1234-123456789012".parse()?,
        "abcdef02-1234-1234-1234-123456789012".parse()?,
    ];
    client.add_runs_to_annotation_queue(queue.base.id, run_ids).await?;

    // List queue items
    for index in 0..10 {
        match client.get_run_from_annotation_queue(queue.base.id, index).await {
            Ok(item) => println!("Item {}: {}", index, item.run.name),
            Err(_) => break, // No more items
        }
    }

    // Clean up
    client.delete_annotation_queue(queue.base.id).await?;

    Ok(())
}
```

---

## Troubleshooting

### "Authentication failed"

Ensure `LANGSMITH_API_KEY` is set:

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

Verify with:

```bash
langstar config
```

### "Queue not found"

- Verify the queue ID is correct: `langstar queue list --json`
- Ensure your API key has access to the workspace containing the queue

### "Run not found"

- Verify the run ID exists: `langstar runs query --filter 'eq(id, "<run-id>")'`
- Ensure the run belongs to a project accessible by your API key

### Empty queue items

The `items` command fetches runs sequentially by index. If you see fewer items than expected:

- Some runs may have been reviewed and removed
- The queue may be empty

---

## API Reference

The annotation queue commands use the LangSmith REST API:

| CLI Command        | HTTP Method | Endpoint                                       |
| ------------------ | ----------- | ---------------------------------------------- |
| `queue list`       | GET         | `/api/v1/annotation-queues`                    |
| `queue create`     | POST        | `/api/v1/annotation-queues`                    |
| `queue get`        | GET         | `/api/v1/annotation-queues/{id}`               |
| `queue update`     | PATCH       | `/api/v1/annotation-queues/{id}`               |
| `queue delete`     | DELETE      | `/api/v1/annotation-queues/{id}`               |
| `queue add-runs`   | POST        | `/api/v1/annotation-queues/{id}/runs`          |
| `queue remove-run` | DELETE      | `/api/v1/annotation-queues/{id}/runs/{run_id}` |
| `queue items`      | GET         | `/api/v1/annotation-queues/{id}/run/{index}`   |

For complete API documentation, see the [LangSmith OpenAPI spec](https://api.smith.langchain.com/redoc).
