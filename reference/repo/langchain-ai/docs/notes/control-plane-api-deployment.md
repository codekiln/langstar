# Method 2: Control Plane API Deployment

**Source**: `src/langsmith/cicd-pipeline-example.mdx` (lines 250-273)

## Overview

Method 2 uses the **Control Plane API** to programmatically deploy LangGraph agents to LangSmith Deployments. This method is essential for CI/CD automation and provides more control than the UI-based approach.

## Two Deployment Approaches

### Cloud LangSmith Deployment

**Characteristics:**
- Uses Control Plane API to create deployments
- Points directly to GitHub repository
- **No Docker image building required**
- Simplest programmatic deployment option

**Workflow:**
```
GitHub Repository → Control Plane API → LangSmith Deployment
```

**Key Benefit**: LangSmith handles the build process automatically from your GitHub repo.

### Self-Hosted/Hybrid LangSmith Deployment

**Characteristics:**
- Requires manual Docker image building
- Uses container registry for image distribution
- Control Plane API pulls from registry

**Workflow:**
```bash
# 1. Build Docker image locally
langgraph build -t my-agent:latest

# 2. Push to container registry
docker push my-agent:latest

# 3. Use Control Plane API to deploy from registry
# (API call specifies image URI)
```

**Supported Container Registries:**
- Docker Hub
- AWS ECR (Elastic Container Registry)
- Azure ACR (Azure Container Registry)
- Google GCR (Google Container Registry)
- Any registry accessible to deployment environment

## Control Plane API

**API Documentation**: [/langsmith/api-ref-control-plane](/langsmith/api-ref-control-plane)

**Helper Script Example**:
- Repository: https://github.com/langchain-ai/cicd-pipeline-example
- Location: `.github/scripts/langgraph_api.py`
- Purpose: Manages API interactions and deployments in GitHub Actions

## Deployment Types

The Control Plane API supports two deployment types:

1. **Preview/Development Deployments**
   - Created when PR is opened and tests pass
   - Allows testing in staging environment
   - Can be deleted when PR is merged

2. **Production Deployments**
   - Created when PR is merged to main
   - Replaces preview deployment
   - Integrated into production infrastructure

## Deployment Flow

### New Agent Deployment
```mermaid
graph LR
    A[PR Opened] --> B[Tests Pass]
    B --> C[Control Plane API]
    C --> D[Preview Deployment Created]
```

### Agent Revision
```mermaid
graph LR
    A[Existing Deployment ID Found] --> B[Control Plane API]
    C[PR Merged to Main] --> B
    B --> D[Revision Created]
    C --> E[Preview Deleted]
    E --> F[Production Deployment Created]
```

## Key Differences from Method 1 (UI Deployment)

| Aspect | Method 1: UI | Method 2: Control Plane API |
|--------|-------------|----------------------------|
| **Interface** | Web UI | Programmatic API |
| **Automation** | Manual | Fully automated |
| **CI/CD Integration** | No | Yes |
| **GitHub Actions** | Not supported | Fully supported |
| **Flexibility** | Limited | High |
| **Use Case** | Quick manual deploys | Production CI/CD pipelines |

## Environment Configuration

After deployment via Control Plane API, you can set environment variables:

```bash
# External PostgreSQL database
export POSTGRES_URI_CUSTOM="postgresql://user:pass@host:5432/db"

# External Redis cache
export REDIS_URI_CUSTOM="redis://host:6379/0"
```

**Note**: By default, LangSmith creates PostgreSQL and Redis instances for you.

## API Endpoints

### Control Plane API (Deployments)

| Region | Endpoint |
|--------|----------|
| US | `https://api.host.langchain.com` |
| EU | `https://eu.api.host.langchain.com` |
| Self-Hosted | `http(s)://<langsmith-url>/api-host` |

**Critical**: This is different from the LangSmith API (traces, evaluations):
- LangSmith API: `https://api.smith.langchain.com`
- Control Plane API: `https://api.host.langchain.com`

## CI/CD Pipeline Integration

**Purpose**: Method 2 is specifically designed for automated CI/CD workflows.

**Typical Workflow in GitHub Actions:**

1. Code pushed to branch
2. Tests run (unit, integration, e2e, offline evals)
3. **Control Plane API creates preview deployment**
4. Online evaluations run on staging
5. PR merged to main
6. **Control Plane API creates production deployment**

**Benefits:**
- Zero manual intervention
- Consistent deployment process
- Quality gates enforced
- Rollback capability

## Prerequisites

Before using Control Plane API:

1. **LangSmith API Key** - Needed for authentication
2. **Organization ID** - Target organization for deployment
3. **langgraph.json** - Configuration file in repository
4. **Dependencies file** - requirements.txt or pyproject.toml

## Common Pitfalls

### Wrong Endpoint
- ❌ Using `api.smith.langchain.com` for deployments
- ✅ Using `api.host.langchain.com` for deployments

### Missing API Key
- Control Plane API requires proper authentication
- Set `LANGSMITH_API_KEY` in GitHub Secrets

### Self-Hosted: Registry Access
- Deployment environment must have access to container registry
- Configure registry credentials in deployment environment

## Additional Resources

- [Control Plane API Reference](/langsmith/api-ref-control-plane)
- [LangSmith API Reference](https://api.smith.langchain.com/redoc)
- [Example Helper Script](https://github.com/langchain-ai/cicd-pipeline-example/blob/main/.github/scripts/langgraph_api.py)
- [Application Structure Documentation](/langsmith/application-structure)
- [LangGraph CLI Build Documentation](/langsmith/cli#build)
