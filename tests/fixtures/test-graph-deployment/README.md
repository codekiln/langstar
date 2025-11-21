# Test LangGraph Deployment

This directory contains a minimal LangGraph application used for integration testing of the Langstar SDK and CLI assistant functionality.

## Overview

This test deployment provides a simple echo graph that can be deployed to LangGraph Cloud and used to test:

- Assistant CRUD operations (create, list, get, update, delete)
- Assistant search functionality
- CLI commands for assistant management
- Error handling and validation

## Structure

```
test-graph-deployment/
├── langgraph.json          # LangGraph configuration
├── requirements.txt        # Python dependencies
├── .env.example           # Environment variable template
├── README.md              # This file
├── DEPLOYMENT_GUIDE.md    # Detailed deployment instructions
└── test_agent/            # Python module
    ├── __init__.py
    └── agent.py           # Minimal echo graph implementation
```

## Graph Implementation

The test graph is intentionally minimal:

- **Single Node**: An `echo` node that prefixes messages with "Echo: "
- **Simple State**: Contains only a `message` field
- **No Dependencies**: Does not require API keys or external services
- **Fast Execution**: Completes immediately for quick test cycles

### Graph Flow

```
START → echo_node → END
```

**Input:**
```json
{
  "message": "Hello, World!"
}
```

**Output:**
```json
{
  "message": "Echo: Hello, World!"
}
```

## Quick Start

### 1. Review the Code

The graph implementation is in `test_agent/agent.py`:

```python
from langgraph.graph import StateGraph, END

# Define state
class State(TypedDict):
    message: str

# Define node
def echo_node(state: State) -> State:
    return {"message": f"Echo: {state['message']}"}

# Build graph
builder = StateGraph(State)
builder.add_node("echo", echo_node)
builder.set_entry_point("echo")
builder.add_edge("echo", END)

# Compile
graph = builder.compile()
```

### 2. (Optional) Test Locally

If you want to test the graph locally before deploying:

```bash
# Install dependencies
pip install -r requirements.txt

# Install LangGraph CLI
pip install langgraph-cli

# Run local development server
langgraph dev

# In another terminal, test the graph
curl -X POST http://localhost:8000/runs \
  -H "Content-Type: application/json" \
  -d '{"input": {"message": "test"}}'
```

### 3. Deploy to LangGraph Cloud

See **[DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)** for detailed step-by-step instructions.

**Summary:**

1. Navigate to [LangSmith](https://smith.langchain.com/)
2. Go to "Deployments" section
3. Click "New Deployment"
4. Connect this repository or upload files
5. Configure deployment settings
6. Deploy
7. Note the **Graph ID** from the deployment details

### 4. Set Environment Variables

After deployment, set these environment variables for integration tests:

```bash
# Required for all tests
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"

# Required for assistant tests
export TEST_GRAPH_ID="<graph-id-from-deployment>"

# Optional: For read-only tests
export TEST_DEPLOYMENT_ID="<deployment-id>"
```

**Finding these values:**

- **LANGSMITH_API_KEY**: [LangSmith Settings](https://smith.langchain.com/settings)
- **LANGSMITH_WORKSPACE_ID**: LangSmith UI → Settings → Workspace ID
- **TEST_GRAPH_ID**: After deployment, go to deployment details → Copy Graph ID
- **TEST_DEPLOYMENT_ID**: Deployment URL or details page

### 5. Create a Test Assistant (Optional)

Once deployed, you can manually create a test assistant via:

**LangSmith UI:**
1. Go to "Assistants" section
2. Click "New Assistant"
3. Select your deployed graph
4. Name it "Test Assistant"
5. Save

**Or via Langstar CLI (once implemented):**
```bash
langstar assistant create \
  --graph-id "$TEST_GRAPH_ID" \
  --name "Test Assistant" \
  --config '{"temperature": 0.7}'
```

## Integration Tests

This deployment is used by integration tests in:

- `sdk/tests/assistant_integration_test.rs` (Phase 5)
- CLI integration tests (future)

**To run integration tests:**

```bash
# Ensure environment variables are set
export TEST_GRAPH_ID="<your-graph-id>"

# Run assistant integration tests
cd /workspace
cargo test --test assistant_integration_test -- --ignored --nocapture
```

See `sdk/tests/README.md` for more details on running tests.

## Maintenance

### Updating the Graph

To update the graph logic:

1. Modify `test_agent/agent.py`
2. Commit changes
3. Redeploy via LangSmith UI or CI/CD

### Cleanup

To clean up test data:

```bash
# List all test assistants (via Langstar CLI)
langstar assistant list | grep "test-assistant"

# Delete test assistants
langstar assistant delete <assistant-id>

# Or use the LangSmith UI to bulk delete
```

## Troubleshooting

### Graph Fails to Deploy

**Error:** "Cannot find module 'test_agent'"

**Solution:** Ensure `langgraph.json` has `"dependencies": ["./test_agent"]`

---

**Error:** "No module named 'langgraph'"

**Solution:** Check `requirements.txt` includes `langgraph>=0.2.0`

---

### Assistant Creation Fails

**Error:** "Invalid graph_id"

**Solution:**
1. Verify deployment is active in LangSmith UI
2. Check the Graph ID matches exactly
3. Ensure workspace ID is correct

---

**Error:** "Authentication failed"

**Solution:**
1. Verify `LANGSMITH_API_KEY` is set and valid
2. Check API key has correct permissions
3. Verify you're in the correct workspace

---

### Local Testing Issues

**Error:** "Address already in use"

**Solution:** Stop existing `langgraph dev` processes:
```bash
pkill -f "langgraph dev"
```

---

**Error:** "Permission denied" when installing packages

**Solution:** Use virtual environment:
```bash
python -m venv venv
source venv/bin/activate  # or `venv\Scripts\activate` on Windows
pip install -r requirements.txt
```

## References

- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/)
- [LangGraph Cloud Deployment](https://langchain-ai.github.io/langgraph/cloud/)
- [LangSmith Assistants](https://docs.smith.langchain.com/assistants)
- [Langstar SDK Documentation](../../../sdk/README.md)

## Related Issues

- Issue #93: Phase 4 - Test LangGraph Deployment Setup
- Issue #83: Add LangGraph Deployment Assistants support to CLI (Epic)
- Issue #94: Phase 5 - Integration Tests (depends on this deployment)
