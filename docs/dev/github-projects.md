# GitHub Projects Configuration

This document describes the GitHub Projects V2 setup for the Langstar repository.

## Project Overview

**Project Name:** langstar
**Project Number:** 4
**Project ID:** `PVT_kwHOAAImgs4BGe4B`
**Description:** A project for governing LangStar issues
**Visibility:** Public
**URL:** https://github.com/users/codekiln/projects/4

## Authentication

To access GitHub Projects V2 via the API, use the `GITHUB_PROJECT_PAT` environment variable (defined in `.devcontainer/.env`). This PAT has the necessary permissions to read and modify project data. This is not available in a github actions setting; it's only available locally.

### Using the PAT with GitHub CLI

```bash
export GH_TOKEN=$GITHUB_PROJECT_PAT
gh api graphql -f query='...'
```

## Project Fields

The langstar project includes the following fields:

| Field Name | Data Type | Field ID | Description |
|------------|-----------|----------|-------------|
| Title | TITLE | `PVTF_lAHOAAImgs4BGe4Bzg3g-NI` | Issue/PR title |
| Assignees | ASSIGNEES | `PVTF_lAHOAAImgs4BGe4Bzg3g-NM` | Who is working on this |
| **Status** | SINGLE_SELECT | `PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ` | Current work status |
| Labels | LABELS | `PVTF_lAHOAAImgs4BGe4Bzg3g-NU` | Issue labels |
| Linked pull requests | LINKED_PULL_REQUESTS | `PVTF_lAHOAAImgs4BGe4Bzg3g-NY` | Associated PRs |
| Milestone | MILESTONE | `PVTF_lAHOAAImgs4BGe4Bzg3g-Nc` | Release milestone |
| Repository | REPOSITORY | `PVTF_lAHOAAImgs4BGe4Bzg3g-Ng` | Source repository |
| Reviewers | REVIEWERS | `PVTF_lAHOAAImgs4BGe4Bzg3g-Nk` | PR reviewers |
| Parent issue | PARENT_ISSUE | `PVTF_lAHOAAImgs4BGe4Bzg3g-No` | Parent issue link |
| Sub-issues progress | SUB_ISSUES_PROGRESS | `PVTF_lAHOAAImgs4BGe4Bzg3g-Ns` | Child issue tracking |

## Status Field Configuration

The **Status** field is the primary workflow field with three states:

| Status Name | Option ID | Description |
|-------------|-----------|-------------|
| **Todo** | `f75ad846` | This item hasn't been started |
| **In Progress** | `47fc9ee4` | This is actively being worked on |
| **Done** | `98236657` | This has been completed |

### Status Workflow

Issues should follow this progression:

1. **Todo** - New issues start here
2. **In Progress** - Move here when work begins
3. **Done** - Move here when work is complete

## Querying Project Data

### Get All Projects for User

```bash
gh api graphql -f query='
query {
  user(login: "codekiln") {
    projectsV2(first: 10) {
      nodes {
        id
        number
        title
        shortDescription
        url
      }
    }
  }
}'
```

### Get Project Fields and Status Options

```bash
gh api graphql -f query='
query {
  node(id: "PVT_kwHOAAImgs4BGe4B") {
    ... on ProjectV2 {
      id
      number
      title
      fields(first: 20) {
        nodes {
          ... on ProjectV2SingleSelectField {
            id
            name
            options {
              id
              name
              description
            }
          }
        }
      }
    }
  }
}'
```

### Get Issue Project Status

```bash
gh issue view <issue-number> --json projectItems
```

Example output:
```json
{
  "projectItems": [
    {
      "status": {
        "optionId": "f75ad846",
        "name": "Todo"
      },
      "title": "langstar"
    }
  ]
}
```

## Modifying Project Items

### Add Issue to Project

```bash
gh api graphql -f query='
mutation {
  addProjectV2ItemById(input: {
    projectId: "PVT_kwHOAAImgs4BGe4B"
    contentId: "<issue-node-id>"
  }) {
    item {
      id
    }
  }
}'
```

### Update Item Status

```bash
gh api graphql -f query='
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "PVT_kwHOAAImgs4BGe4B"
    itemId: "<item-id>"
    fieldId: "PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ"
    value: {
      singleSelectOptionId: "47fc9ee4"
    }
  }) {
    projectV2Item {
      id
    }
  }
}'
```

## Integration with Development Workflow

The GitHub project integrates with the issue-driven development workflow described in [github-workflow.md](./github-workflow.md):

1. **Issue Created** → Automatically added to project with status "Todo"
2. **Work Starts** → Status updated to "In Progress"
3. **PR Merged** → Status updated to "Done"

### Automation Opportunities

Consider implementing GitHub Actions to:
- Auto-add new issues to the project
- Update status to "In Progress" when a branch is created
- Update status to "Done" when a PR is merged
- Auto-assign issues based on branch ownership

## Troubleshooting

### PAT Permissions

If you encounter "Resource not accessible" errors, ensure the PAT has these scopes:
- `project` - Read/write access to projects
- `repo` - Access to repository issues and PRs

### GraphQL Node IDs

GitHub uses global node IDs for GraphQL queries. To get an issue's node ID:

```bash
gh issue view <issue-number> --json id
```

## Related Documentation

- [GitHub Workflow](./github-workflow.md) - Issue-driven development process
- [Git SCM Conventions](./git-scm-conventions.md) - Commit message standards
- [GitHub Projects V2 API Documentation](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)
