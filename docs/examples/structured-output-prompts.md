# Structured Output Prompts Usage Guide

**Issue**: [#409](https://github.com/codekiln/langstar/issues/409) - Documentation for structured output prompt support
**Feature**: [#402](https://github.com/codekiln/langstar/issues/402) - Add structured output prompt support to langstar CLI

## Overview

Structured output prompts combine a prompt template with a JSON Schema to constrain LLM outputs to match a predefined structure. This enables reliable data extraction, API response formatting, and typed output handling.

### Benefits

- **Type Safety**: Outputs conform to defined schemas
- **Validation**: Invalid outputs are rejected
- **Consistency**: Same structure every time
- **Integration**: Direct mapping to typed objects in your code

### Use Cases

1. **Data Extraction** - Extract structured information from unstructured text
2. **API Responses** - Format LLM outputs to match API schemas
3. **Form Filling** - Generate structured data for forms
4. **Classification** - Categorize inputs with structured labels

## Quick Start

### 1. Create a JSON Schema

JSON Schema defines the structure of your expected output:

```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "The movie title"
    },
    "rating": {
      "type": "integer",
      "minimum": 1,
      "maximum": 10,
      "description": "Rating from 1-10"
    },
    "summary": {
      "type": "string",
      "description": "Brief summary"
    }
  },
  "required": ["title", "rating", "summary"]
}
```

Save this as `movie-review.json`.

### 2. Push the Prompt

```bash
langstar prompt push \
  -o team -r movie-reviewer \
  -t "Review the movie: {movie_name}" \
  -i "movie_name" \
  --schema movie-review.json
```

### 3. Use in Your Application

Pull the prompt from LangSmith and use it with an LLM:

```bash
langstar prompt pull team/movie-reviewer
```

## Detailed Examples

### Example 1: Invoice Data Extraction

Extract structured invoice data from documents.

**Schema** (`invoice-schema.json`):

```json
{
  "type": "object",
  "title": "Invoice",
  "properties": {
    "invoice_number": {
      "type": "string",
      "description": "Unique invoice identifier"
    },
    "date": {
      "type": "string",
      "format": "date",
      "description": "Invoice date (YYYY-MM-DD)"
    },
    "vendor": {
      "type": "string",
      "description": "Vendor/seller name"
    },
    "amount": {
      "type": "number",
      "minimum": 0,
      "description": "Total amount"
    },
    "currency": {
      "type": "string",
      "enum": ["USD", "EUR", "GBP"],
      "default": "USD"
    }
  },
  "required": ["invoice_number", "amount"]
}
```

**Push Command**:

```bash
langstar prompt push \
  -o acme -r invoice-extractor \
  -t "Extract invoice data from the following document:\n\n{document}" \
  -i "document" \
  --schema invoice-schema.json
```

### Example 2: Sentiment Analysis

Analyze sentiment with categorical classification.

**Schema** (`sentiment-schema.json`):

```json
{
  "type": "object",
  "title": "SentimentAnalysis",
  "properties": {
    "sentiment": {
      "type": "string",
      "enum": ["positive", "negative", "neutral"],
      "description": "Overall sentiment"
    },
    "confidence": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Confidence score"
    },
    "aspects": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "aspect": {"type": "string"},
          "sentiment": {"type": "string", "enum": ["positive", "negative", "neutral"]}
        }
      },
      "description": "Aspect-level sentiments"
    }
  },
  "required": ["sentiment", "confidence"]
}
```

**Push Command**:

```bash
langstar prompt push \
  -o team -r sentiment-analyzer \
  -t "Analyze the sentiment of: {text}" \
  -i "text" \
  --schema sentiment-schema.json
```

### Example 3: Contact Information Extraction

Extract contact details from business cards or emails.

**Schema** (`contact-schema.json`):

```json
{
  "type": "object",
  "title": "ContactInfo",
  "properties": {
    "name": {
      "type": "string",
      "description": "Full name"
    },
    "email": {
      "type": "string",
      "format": "email",
      "description": "Email address"
    },
    "phone": {
      "type": "string",
      "pattern": "^\\+?[1-9]\\d{1,14}$",
      "description": "Phone number (E.164 format)"
    },
    "company": {
      "type": "string",
      "description": "Company name"
    },
    "title": {
      "type": "string",
      "description": "Job title"
    }
  },
  "required": ["name"]
}
```

**Push Command**:

```bash
langstar prompt push \
  -o team -r contact-extractor \
  -t "Extract contact information from: {source}" \
  -i "source" \
  --schema contact-schema.json
```

## Structured Output Methods

Langstar supports two methods for applying schemas:

### json_schema (Default)

Uses JSON Schema mode where the LLM is instructed to output valid JSON matching the schema.

```bash
langstar prompt push \
  -o team -r my-prompt \
  -t "Extract: {input}" \
  --schema schema.json \
  --schema-method json_schema
```

**When to use:**

- General-purpose structured output
- Most common use case
- Supported by most modern LLMs

### function_calling

Uses function calling mode where the schema represents a function's parameters.

```bash
langstar prompt push \
  -o team -r my-prompt \
  -t "Process: {input}" \
  --schema schema.json \
  --schema-method function_calling
```

**When to use:**

- When using models optimized for function calling
- Tool/agent workflows
- OpenAI models with function calling support

## Command Reference

### Push Structured Prompt

```bash
langstar prompt push \
  -o <owner> \
  -r <repo> \
  -t <template> \
  [--schema <FILE>] \
  [--schema-method <METHOD>]
```

**Required Arguments:**

- `-o, --owner` - Prompt owner (username or organization)
- `-r, --repo` - Prompt repository name
- `-t, --template` - Prompt template text

**Structured Output Arguments:**

- `--schema <FILE>` - Path to JSON Schema file
- `--schema-method <METHOD>` - Method: `json_schema` or `function_calling` (default: `json_schema`)

**Optional Arguments:**

- `-i, --input-variables` - Comma-separated input variables
- `--template-format` - Template format (default: `f-string`)
- `--organization-id` - Organization scope
- `--workspace-id` - Workspace scope

### Pull Structured Prompt

```bash
langstar prompt pull <handle> [--commit <COMMIT>]
```

**Arguments:**

- `<handle>` - Prompt handle (e.g., `owner/prompt-name`)
- `--commit` - Commit hash or tag (default: `latest`)

**Output**: Shows the prompt including schema if structured.

### Get Structured Prompt Details

```bash
langstar prompt get <handle>
```

Shows full prompt details including schema and metadata.

## JSON Schema Tips

### Basic Structure

Every JSON Schema must have:

- `"type": "object"` at the top level
- `properties` object defining fields
- `required` array (optional but recommended)

```json
{
  "type": "object",
  "properties": {
    "field_name": {"type": "string"}
  },
  "required": ["field_name"]
}
```

### Common Field Types

```json
{
  "string_field": {"type": "string"},
  "number_field": {"type": "number"},
  "integer_field": {"type": "integer"},
  "boolean_field": {"type": "boolean"},
  "array_field": {
    "type": "array",
    "items": {"type": "string"}
  },
  "object_field": {
    "type": "object",
    "properties": {
      "nested": {"type": "string"}
    }
  }
}
```

### Validation Constraints

**Strings:**

```json
{
  "type": "string",
  "minLength": 1,
  "maxLength": 100,
  "pattern": "^[A-Z]",
  "enum": ["option1", "option2"],
  "format": "email"  // email, date, uri, etc.
}
```

**Numbers:**

```json
{
  "type": "number",
  "minimum": 0,
  "maximum": 100,
  "multipleOf": 0.01
}
```

**Arrays:**

```json
{
  "type": "array",
  "items": {"type": "string"},
  "minItems": 1,
  "maxItems": 10,
  "uniqueItems": true
}
```

### Adding Descriptions

Always add descriptions to help the LLM understand field meanings:

```json
{
  "type": "object",
  "title": "ProductReview",
  "description": "A review of a product",
  "properties": {
    "rating": {
      "type": "integer",
      "description": "Product rating from 1-5 stars",
      "minimum": 1,
      "maximum": 5
    },
    "pros": {
      "type": "array",
      "description": "List of positive aspects",
      "items": {"type": "string"}
    }
  }
}
```

## Common Patterns

### Pattern 1: Multiple Choice Classification

```json
{
  "type": "object",
  "properties": {
    "category": {
      "type": "string",
      "enum": ["bug", "feature", "question", "documentation"]
    },
    "confidence": {"type": "number", "minimum": 0, "maximum": 1}
  },
  "required": ["category"]
}
```

### Pattern 2: Named Entity Recognition

```json
{
  "type": "object",
  "properties": {
    "entities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "text": {"type": "string"},
          "type": {"type": "string", "enum": ["person", "organization", "location"]},
          "start": {"type": "integer"},
          "end": {"type": "integer"}
        },
        "required": ["text", "type"]
      }
    }
  },
  "required": ["entities"]
}
```

### Pattern 3: Hierarchical Classification

```json
{
  "type": "object",
  "properties": {
    "primary_category": {"type": "string"},
    "subcategories": {
      "type": "array",
      "items": {"type": "string"}
    },
    "tags": {
      "type": "array",
      "items": {"type": "string"}
    }
  },
  "required": ["primary_category"]
}
```

## Troubleshooting

### Schema Validation Errors

**Error**: `Schema file contains invalid JSON`

- **Cause**: Syntax error in JSON file
- **Fix**: Validate JSON with a linter or `jq < schema.json`

**Error**: `Schema file is not a valid JSON Schema`

- **Cause**: Missing required fields or invalid schema structure
- **Fix**: Ensure schema has `"type": "object"` at root level

### Method Errors

**Error**: `Invalid schema method`

- **Cause**: Typo in `--schema-method` value
- **Fix**: Use `json_schema` or `function_calling`

### File Not Found

**Error**: `Schema file not found`

- **Cause**: Incorrect path to schema file
- **Fix**: Verify file path is correct and file exists

## Advanced Usage

### From Pydantic Models

If you have a Pydantic model in Python, export its schema:

```python
from pydantic import BaseModel

class MovieReview(BaseModel):
    title: str
    rating: int
    summary: str

# Export schema
import json
schema = MovieReview.model_json_schema()
with open('movie-schema.json', 'w') as f:
    json.dump(schema, f, indent=2)
```

Then use with langstar:

```bash
langstar prompt push -o team -r reviewer \
  -t "Review: {movie}" \
  --schema movie-schema.json
```

### CI/CD Integration

Store schemas in version control and deploy prompts automatically:

```bash
# .github/workflows/deploy-prompts.yml
- name: Deploy structured prompts
  run: |
    for schema in schemas/*.json; do
      name=$(basename "$schema" .json)
      langstar prompt push \
        -o team -r "$name" \
        -t "$(cat templates/$name.txt)" \
        --schema "$schema"
    done
```

### Schema Reuse

Share schemas across multiple prompts:

```bash
# Use same schema for different tasks
langstar prompt push -o team -r extractor-v1 \
  -t "Extract from: {text}" --schema shared/extraction.json

langstar prompt push -o team -r extractor-v2 \
  -t "Parse the following: {input}" --schema shared/extraction.json
```

## Best Practices

1. **Start Simple** - Begin with basic schemas and add complexity as needed
2. **Add Descriptions** - Always describe fields to guide the LLM
3. **Version Control** - Keep schemas in git alongside code
4. **Validate Locally** - Test schemas with sample data before pushing
5. **Use Enums** - Constrain categorical fields with `enum`
6. **Required Fields** - Mark essential fields in `required` array
7. **Reasonable Limits** - Set min/max constraints to prevent invalid outputs

## Related Documentation

- **Research**: [398-structured-output-prompts-scout.md](../research/398-structured-output-prompts-scout.md)
- **Design**: [Research document Section 11](../research/398-structured-output-prompts-scout.md#11-design-decisions)
- **Implementation Plan**: [402-structured-prompts-implementation.md](../implementation/402-structured-prompts-implementation.md)
- **SDK Reference**: [prompts.rs SDK documentation](../../sdk/src/prompts.rs)
- **CLI Reference**: [prompt.rs CLI documentation](../../cli/src/commands/prompt.rs)

## See Also

- [LangChain Structured Output Guide](https://python.langchain.com/docs/how_to/structured_output/)
- [JSON Schema Reference](https://json-schema.org/understanding-json-schema/)
- [LangSmith Prompts Documentation](https://docs.smith.langchain.com/prompt-engineering)
