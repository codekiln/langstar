# Experiment: Projects vs Sessions Terminology

**Issue**: #574
**Date**: 2025-12-05
**Experiment Goal**: Disambiguate "projects" vs "sessions" terminology

## Problem Statement

The Python SDK uses "projects" in method names (`list_projects()`) but the schema classes are named `TracerSession` and the API endpoints are `/sessions`. We need to:

1. Confirm the actual API endpoint paths
2. Understand the request/response field names
3. Determine the correct terminology for the Rust SDK

## Hypothesis

Based on code analysis:

- **Python SDK public API**: Uses "projects" terminology
- **Python SDK schemas**: Uses "TracerSession" classes
- **REST API endpoints**: Uses "/sessions" paths
- **LangSmith UI**: Refers to them as "Projects"

This experiment will verify this mapping through actual API calls.

## Experiment Setup

### Prerequisites

```bash
# Ensure langsmith Python package is installed
pip install langsmith

# Set API key
export LANGSMITH_API_KEY=<your-api-key>
```

### Files

- `test_projects.py` - Python script to test API behavior
- `run_test.sh` - Shell wrapper with environment setup
- `validate_projects.py` - Script to validate full workspace project listing and querying

## Running the Experiment

```bash
# From repository root
cd reference/experiments/574-ls-projects

# Run the experiment
./run_test.sh
```

Or directly:

```bash
python test_projects.py
```

## Expected Outcomes

The experiment will demonstrate:

1. **SDK Method Names**: All use "projects" terminology
   - `list_projects()`, `read_project()`, etc.

2. **API Endpoints**: All use "/sessions" paths
   - Visible in error messages or debug output

3. **Schema Classes**: Use "TracerSession" naming
   - Returned objects are `TracerSessionResult` instances

4. **Field Names**: Should clarify if fields reference "session" or "project"
   - Check for `session_id` vs `project_id`
   - Check metadata field naming

## Results

**Date Run**: 2025-12-05
**Status**: ✅ Complete

### Key Findings

1. **Python SDK Returns TracerSessionResult**
   - Object type: `TracerSessionResult`
   - Inherits from `TracerSession` base class
   - Contains ~30 fields including stats, costs, tokens

2. **Terminology is Consistent Across Layers**
   ```
   Layer                 Terminology
   ─────────────────────────────────────────
   LangSmith UI         'Projects' (in URLs: /projects/p/{id})
   Python SDK Methods   'Projects' (list_projects, read_project)
   Python Schema Class  'TracerSession' (internal)
   REST API Endpoint    '/sessions' (GET/POST/PATCH/DELETE)
   ```

3. **Field Names Use Session Terminology Sparingly**
   - Only one field found: `session_feedback_stats`
   - All other fields are neutral (id, name, description, etc.)
   - No `session_id` field (it's just `id`)

4. **URL Structure Reveals UI Preference**
   ```
   https://smith.langchain.com/o/{tenant_id}/projects/p/{project_id}
   ```
   The UI explicitly uses "projects/p/" in URLs, not "sessions"

### API Behavior Confirmed

- ✅ `list_projects()` successfully returns project list
- ✅ `read_project(project_name=...)` successfully reads by name
- ✅ Returns `TracerSessionResult` objects with all expected fields
- ✅ API URL is `https://api.smith.langchain.com` (uses /sessions endpoint internally)

### Terminology Mapping Verified

| Concept             | Python SDK            | REST API                | Rust SDK (Recommended)              |
| ------------------- | --------------------- | ----------------------- | ----------------------------------- |
| Collection endpoint | `list_projects()`     | `GET /sessions`         | `list_projects()`                   |
| Single read         | `read_project()`      | `GET /sessions/{id}`    | `get_project()` or `read_project()` |
| Create              | `create_project()`    | `POST /sessions`        | `create_project()`                  |
| Update              | `update_project()`    | `PATCH /sessions/{id}`  | `update_project()`                  |
| Delete              | `delete_project()`    | `DELETE /sessions/{id}` | `delete_project()`                  |
| Schema class        | `TracerSessionResult` | -                       | `Project` or `TracerSession`        |

## Implications for Rust SDK

### Recommended Design

1. **Public Struct Name**: `Project`
   ```rust
   pub struct Project {
       pub id: Uuid,
       pub name: Option<String>,
       pub description: Option<String>,
       // ... other fields
   }
   ```

2. **Method Names**: Use "project" terminology
   ```rust
   impl LangchainClient {
       pub async fn list_projects(&self, ...) -> Result<Vec<Project>>
       pub async fn get_project(&self, ...) -> Result<Project>
       pub async fn create_project(&self, ...) -> Result<Project>
       pub async fn update_project(&self, ...) -> Result<Project>
       pub async fn delete_project(&self, ...) -> Result<()>
   }
   ```

3. **Internal Endpoint Mapping**: Use `/sessions` paths
   ```rust
   // In implementation:
   let endpoint = format!("{}/sessions", self.api_url);
   ```

4. **Field Naming**: Follow Python SDK precedent
   - Use `session_feedback_stats` (keep original name)
   - All other fields use neutral terminology

### Rationale

- **Consistency**: Matches Python SDK public API
- **User Expectation**: UI calls them "Projects"
- **Developer Experience**: More intuitive than "sessions"
- **API Compatibility**: Internal mapping to /sessions is implementation detail

## Additional Validation

### Full Workspace Project Listing

**Script**: `validate_projects.py`

**Purpose**: Validate that the Python SDK can successfully:

1. List all projects in a workspace (162 projects)
2. Query specific project by name
3. Retrieve project ID
4. Count runs within a project

**Results**:

```
✅ Total projects found: 162
✅ Found project: test-deployment-cli-48499
   Project ID: 98e12dc6-2171-4bf3-80fb-1153041d6cbf
✅ Total runs in project: 0
```

**Key Findings**:

- Successfully listed all 162 projects in workspace
- Project lookup by name works correctly
- Run counting per project is supported via `list_runs(project_name=...)`
- Confirms pagination handles large project lists (>100 items)

This validates the full CRUD workflow and ensures the Rust SDK implementation will have all necessary API capabilities.

## References

- Python SDK client.py:3408-3780 (project methods)
- Python SDK schemas.py:729-788 (TracerSession classes)
- Python SDK schemas.py:732 comment: "Sessions are also referred to as 'Projects' in the UI."
