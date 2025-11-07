# LangGraph Cloud Deployment Guide

This guide provides detailed step-by-step instructions for deploying the test LangGraph application to LangGraph Cloud via the LangSmith UI.

## Prerequisites

Before you begin, ensure you have:

- [ ] A LangSmith account (sign up at https://smith.langchain.com/)
- [ ] LangSmith API key (from https://smith.langchain.com/settings)
- [ ] Access to this repository (if deploying via GitHub integration)
- [ ] Or the ability to upload files manually

## Deployment Methods

There are three primary methods to deploy to LangGraph Cloud:

1. **GitHub Integration** (Recommended) - Automatic deployment from GitHub
2. **Manual File Upload** - Upload files directly via UI
3. **LangGraph CLI** - Deploy from command line (requires langgraph-cli)

This guide covers **Method 1 (GitHub)** and **Method 2 (Manual Upload)**.

---

## Method 1: Deploy via GitHub Integration (Recommended)

### Step 1: Connect GitHub Repository

1. Navigate to [LangSmith](https://smith.langchain.com/)
2. Log in to your account
3. In the left sidebar, click **"Deployments"**
4. Click the **"New Deployment"** button (top right)
5. Select **"Connect GitHub"**
6. Authorize LangSmith to access your GitHub account
7. Select the repository: `codekiln/langstar`
8. Select the branch: `claude/93-phase-4-test-langgraph-deployment` (or current branch)

### Step 2: Configure Deployment Settings

1. **Deployment Name:**
   - Enter: `langstar-test-graph` (or any descriptive name)

2. **Path to langgraph.json:**
   - Enter: `tests/fixtures/test-graph-deployment/langgraph.json`
   - This tells LangGraph Cloud where to find your configuration

3. **Environment Variables:** (Optional)
   - Click **"Add Environment Variable"**
   - Add any keys from `.env.example` if needed:
     - `LANGSMITH_API_KEY` - Your LangSmith API key (for tracing)
   - For this minimal test graph, environment variables are **optional**

4. **Python Version:**
   - Select: **Python 3.11** (or latest stable)

5. **Build Settings:** (Usually auto-detected)
   - **Requirements File:** `tests/fixtures/test-graph-deployment/requirements.txt`
   - Should be auto-detected from `langgraph.json`

### Step 3: Review and Deploy

1. Review all settings
2. Click **"Create Deployment"**
3. LangGraph Cloud will:
   - Clone your repository
   - Install dependencies from `requirements.txt`
   - Build the graph from `test_agent/agent.py`
   - Deploy to cloud infrastructure

### Step 4: Monitor Deployment

1. You'll be redirected to the deployment details page
2. Monitor the build logs:
   - Look for "Installing dependencies..."
   - Look for "Building graph..."
   - Look for "Deployment successful"
3. Wait for status to change to **"Active"** (usually 2-5 minutes)

### Step 5: Get Deployment Details

Once deployed:

1. On the deployment details page, note:
   - **Deployment ID**: A UUID like `d7e8c9a1-2b3c-4d5e-6f7a-8b9c0d1e2f3a`
   - **Graph ID**: A string like `test_graph` (matches `langgraph.json`)
   - **Deployment URL**: API endpoint for the graph

2. Copy the **Graph ID** - you'll need this for tests:
   ```bash
   export TEST_GRAPH_ID="<graph-id-here>"
   ```

3. Verify deployment:
   - Look for "Status: Active" with green indicator
   - Check "Last Deployed" timestamp is recent

### Step 6: Test Deployment

**Via LangSmith UI:**

1. On the deployment page, click **"Playground"** or **"Test"**
2. Enter test input:
   ```json
   {
     "message": "Hello, World!"
   }
   ```
3. Click **"Run"**
4. Verify output:
   ```json
   {
     "message": "Echo: Hello, World!"
   }
   ```

**Via curl:**

```bash
# Get the deployment URL from the UI
DEPLOYMENT_URL="<your-deployment-url>"
LANGSMITH_API_KEY="<your-api-key>"

curl -X POST "${DEPLOYMENT_URL}/runs" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${LANGSMITH_API_KEY}" \
  -d '{
    "input": {
      "message": "Hello from curl!"
    }
  }'
```

### Step 7: Create Test Assistant (Optional)

Now that your graph is deployed, you can create assistants:

**Via LangSmith UI:**

1. In LangSmith, go to **"Assistants"** (left sidebar)
2. Click **"New Assistant"**
3. Fill in details:
   - **Name:** `Test Assistant`
   - **Graph:** Select your deployed `langstar-test-graph`
   - **Configuration:** (Optional) Add any config like:
     ```json
     {
       "temperature": 0.7
     }
     ```
4. Click **"Create"**
5. Note the **Assistant ID** for tests

**Via Langstar CLI (once implemented):**

```bash
langstar assistant create \
  --graph-id "$TEST_GRAPH_ID" \
  --name "Test Assistant" \
  --config '{"temperature": 0.7}'
```

---

## Method 2: Deploy via Manual File Upload

If you prefer not to connect GitHub, you can upload files manually.

### Step 1: Prepare Files

1. Ensure all files are in the fixture directory:
   ```
   test-graph-deployment/
   ├── langgraph.json
   ├── requirements.txt
   └── test_agent/
       ├── __init__.py
       └── agent.py
   ```

2. (Optional) Create a ZIP file:
   ```bash
   cd tests/fixtures
   zip -r test-graph-deployment.zip test-graph-deployment/
   ```

### Step 2: Upload to LangSmith

1. Navigate to [LangSmith Deployments](https://smith.langchain.com/deployments)
2. Click **"New Deployment"**
3. Select **"Upload Files"**
4. Either:
   - Drag and drop the `test-graph-deployment.zip` file
   - Or click "Browse" and select individual files

### Step 3: Configure and Deploy

1. Follow steps 2-7 from Method 1 above
2. The process is identical after file upload

---

## Post-Deployment Setup

### Set Environment Variables for Tests

After successful deployment, set these environment variables on your local machine or in CI:

```bash
# Required for authentication
export LANGSMITH_API_KEY="<your-api-key>"
export LANGCHAIN_WORKSPACE_ID="<your-workspace-id>"

# Required for assistant tests
export TEST_GRAPH_ID="<graph-id-from-deployment>"

# Optional: For deployment-related tests
export TEST_DEPLOYMENT_ID="<deployment-id>"
```

**Where to find these:**

- **LANGSMITH_API_KEY**: https://smith.langchain.com/settings → "API Keys" → "Create API Key"
- **LANGCHAIN_WORKSPACE_ID**:
  - LangSmith UI → "Settings" → "Workspace"
  - Or via API: `curl -H "X-API-Key: $LANGSMITH_API_KEY" https://api.smith.langchain.com/api/v1/workspaces`
- **TEST_GRAPH_ID**: Deployment details page → "Graph ID" field
- **TEST_DEPLOYMENT_ID**: Deployment details page → URL or "Deployment ID" field

### Add to Shell Profile (Optional)

To persist these variables:

```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export LANGSMITH_API_KEY="<your-key>"' >> ~/.bashrc
echo 'export LANGCHAIN_WORKSPACE_ID="<your-workspace-id>"' >> ~/.bashrc
echo 'export TEST_GRAPH_ID="<your-graph-id>"' >> ~/.bashrc

# Reload shell
source ~/.bashrc
```

### Configure CI/CD (Optional)

If running tests in CI/CD, add these as secrets:

**GitHub Actions:**

1. Go to repository → Settings → Secrets and variables → Actions
2. Add new repository secrets:
   - `LANGSMITH_API_KEY`
   - `LANGCHAIN_WORKSPACE_ID`
   - `TEST_GRAPH_ID`

**In workflow file:**

```yaml
- name: Run integration tests
  env:
    LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
    LANGCHAIN_WORKSPACE_ID: ${{ secrets.LANGCHAIN_WORKSPACE_ID }}
    TEST_GRAPH_ID: ${{ secrets.TEST_GRAPH_ID }}
  run: |
    cargo test --test assistant_integration_test -- --ignored --nocapture
```

---

## Troubleshooting

### Deployment Fails

**Error:** "Cannot find langgraph.json"

**Solution:**
- Verify the path in deployment settings: `tests/fixtures/test-graph-deployment/langgraph.json`
- Ensure the file exists in the repository
- Check branch selection is correct

---

**Error:** "Failed to install dependencies"

**Solution:**
- Check `requirements.txt` has valid package names
- Verify Python version compatibility
- Look at build logs for specific error messages

---

**Error:** "Cannot find module 'test_agent'"

**Solution:**
- Verify `langgraph.json` has `"dependencies": ["./test_agent"]`
- Ensure directory structure matches:
  ```
  test-graph-deployment/
  └── test_agent/
      ├── __init__.py
      └── agent.py
  ```

---

### Deployment Succeeds but Graph Fails

**Error:** "AttributeError: module 'test_agent.agent' has no attribute 'graph'"

**Solution:**
- Verify `agent.py` has `graph = builder.compile()` at module level
- Check variable name matches `langgraph.json`: `"./test_agent/agent.py:graph"`
- Ensure `graph` is not inside a function or class

---

### Cannot Create Assistant

**Error:** "Invalid graph_id"

**Solution:**
1. Go to deployment details page
2. Copy the exact **Graph ID** (case-sensitive)
3. Verify deployment status is "Active"
4. Check workspace ID is correct

---

**Error:** "403 Forbidden" or "Authentication failed"

**Solution:**
1. Verify `LANGSMITH_API_KEY` is set correctly
2. Check API key has correct permissions:
   - Go to https://smith.langchain.com/settings
   - View API key permissions
   - Ensure "Deployments" and "Assistants" are enabled
3. Verify workspace ID matches your current workspace

---

## Updating the Deployment

To update the deployed graph:

### Via GitHub Integration:

1. Push changes to the connected branch
2. LangSmith will auto-detect changes
3. Click "Redeploy" in the deployment details page
4. Monitor build logs

### Via Manual Upload:

1. Upload new files
2. LangSmith will create a new deployment version
3. Activate the new version

---

## Cleanup

### Deactivate Deployment

1. Go to Deployments → Your deployment
2. Click **"Settings"** or **"..."** menu
3. Select **"Deactivate"**
4. Confirm

### Delete Deployment

1. Deactivate first (see above)
2. Click **"Delete"**
3. Confirm deletion
4. Note: This does NOT delete assistants using this graph

### Delete Test Assistants

**Via LangSmith UI:**

1. Go to "Assistants"
2. Select test assistants
3. Click "Delete"
4. Confirm

**Via Langstar CLI:**

```bash
# List test assistants
langstar assistant list | grep "test-assistant"

# Delete each
langstar assistant delete <assistant-id>
```

---

## Next Steps

Once deployment is successful:

1. ✅ Set environment variables (`TEST_GRAPH_ID`, etc.)
2. ✅ Run integration tests: `cargo test --test assistant_integration_test -- --ignored`
3. ✅ Create test assistants for manual validation
4. ✅ Proceed to Phase 5 (Integration Tests)

---

## References

- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
- [LangSmith Deployments](https://docs.smith.langchain.com/deployments)
- [LangSmith Assistants API](https://docs.smith.langchain.com/api/assistants)
- [Langstar Repository](https://github.com/codekiln/langstar)

## Support

If you encounter issues:

1. Check deployment logs in LangSmith UI
2. Review this troubleshooting section
3. See [fixture README.md](./README.md) for common issues
4. File an issue at https://github.com/codekiln/langstar/issues
