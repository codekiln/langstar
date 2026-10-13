# GitHub Project Configuration

This document contains the GitHub Projects V2 configuration for the langstar project.

## Project Identifiers

These IDs are used by the `update-issue-status.sh` script to interact with the GitHub Projects V2 API.

### Project ID

```
PVT_kwHOAAImgs4BGe4B
```

This is the unique identifier for the langstar GitHub Project.

### Status Field ID

```
PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ
```

This is the field ID for the "Status" field in the project.

### Status Option IDs

The Status field has three possible values:

| Status | Option ID | Description |
|--------|-----------|-------------|
| Todo | `f75ad846` | Initial status for new issues |
| In Progress | `47fc9ee4` | Status for issues being actively worked on |
| Done | `98236657` | Status for completed issues |

## How to Update These IDs

If the project is recreated or the configuration changes, you'll need to update the IDs in the script.

### Finding Project ID

```bash
# List all projects for the repository
gh api graphql -f query='
  query {
    repository(owner: "codekiln", name: "langstar") {
      projectsV2(first: 10) {
        nodes {
          id
          title
          number
        }
      }
    }
  }
' --jq '.data.repository.projectsV2.nodes[]'
```

### Finding Field IDs

```bash
# Get field IDs for a project
PROJECT_ID="PVT_kwHOAAImgs4BGe4B"

gh api graphql -f query="
  query {
    node(id: \"$PROJECT_ID\") {
      ... on ProjectV2 {
        fields(first: 20) {
          nodes {
            ... on ProjectV2FieldCommon {
              id
              name
            }
            ... on ProjectV2SingleSelectField {
              id
              name
              options {
                id
                name
              }
            }
          }
        }
      }
    }
  }
" --jq '.data.node.fields.nodes[]'
```

This will show all field IDs and, for single-select fields like Status, the option IDs for each value.

## Authentication

### Two-Token Approach

This skill uses **two separate tokens** because GitHub doesn't provide a single token type that supports both repository and project operations:

1. **GITHUB_PROJECT_PAT** - Classic Personal Access Token with `project` scope
2. **GITHUB_PAT** - Fine-grained Personal Access Token with `repo` scope

### Token Scopes

**GITHUB_PROJECT_PAT (required):**
- Type: Classic Personal Access Token
- Required scope: `project`
- Used for: All project operations (querying, adding issues, updating status)

**GITHUB_PAT (optional):**
- Type: Fine-grained Personal Access Token
- Required scope: `repo` (or `public_repo` for public repos)
- Used for: Issue assignment only
- If not set, the script skips assignment but still updates project status

### Token Environment Variables

**For Project Operations:**
1. `GITHUB_PROJECT_PAT` - Primary choice
2. `GH_PROJECT_PAT` - Alternative name

**For Issue Operations:**
- `GITHUB_PAT` - Required only for issue assignment

### Setting Up Local Tokens

Create or update `.devcontainer/.env` (gitignored):

```bash
# Classic PAT with project scope (required)
GITHUB_PROJECT_PAT=ghp_xxxxxxxxxxxxxxxxxxxx

# Fine-grained PAT with repo scope (optional, for assignment)
GITHUB_PAT=github_pat_xxxxxxxxxxxxxxxxxxxx
```

Load the environment:

```bash
source .devcontainer/.env
```

### GitHub Actions

In GitHub Actions, the default `GITHUB_TOKEN` works when the workflow has appropriate permissions:

```yaml
permissions:
  issues: write
  contents: read
```

The Actions environment has special privileges that allow project updates through the issue's GraphQL connection.

## Repository Configuration

The script requires the repository to be set:

```bash
export GITHUB_REPOSITORY="codekiln/langstar"
```

Or set it in `.devcontainer/.env`.

## Related Documentation

- [GitHub Projects V2 documentation](../../docs/dev/github-projects.md)
- [Issue automation documentation](../../docs/dev/issue-automation.md)
