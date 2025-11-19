# cicd-pipeline-example

## Repository Information

- **Repository**: [langchain-ai/cicd-pipeline-example](https://github.com/langchain-ai/cicd-pipeline-example.git)
- **Date Created**: 2025-11-19
- **Cloned to**: `../reference/repo/langchain-ai/cicd-pipeline-example/code`

## Purpose

Studying the LangChain CI/CD pipeline example to understand how to deploy LangGraph agents using the Control Plane API with automated testing and evaluation workflows.

## Key Findings

### GitHub Container Registry (GHCR) Support

**Yes, you can use GitHub Container Registry instead of Docker Hub!**

Key advantages:
- ✅ Uses built-in `GITHUB_TOKEN` (no separate credentials)
- ✅ No rate limits (Docker Hub limits to 200 pulls/6 hours)
- ✅ Free for both public and private images
- ✅ Better integration with GitHub Actions workflows

See [Container Registry Options](./container-registry-options.md) for complete details.

### Current Implementation Uses Docker Hub

The example repository uses Docker Hub (`docker.io`) with manual credentials:
- Registry: `docker.io`
- Requires: `DOCKER_USERNAME` and `DOCKER_PASSWORD` secrets
- Image: `perinim98/text2sql-agent`

### Control Plane API Helper Script

Location: `.github/scripts/langgraph_api.py`
- Manages deployments via Control Plane API
- Creates preview deployments for PRs
- Creates production deployments on merge
- Handles deployment revisions

## Architecture

### Repository Structure

```
.
├── agents/                    # LangGraph agent implementations
│   ├── simple_text2sql.py    # Main agent
│   ├── prompts.py            # Prompt templates
│   └── utils.py              # Utility functions
├── .github/
│   ├── scripts/              # API helper scripts
│   │   └── langgraph_api.py # Control Plane API wrapper
│   └── workflows/            # GitHub Actions
│       ├── preview-deployment.yml
│       ├── new-lgp-revision.yml
│       └── test-with-results.yml
├── tests/                    # Test suites
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   ├── e2e/                 # End-to-end tests
│   └── offline_evals/       # Offline evaluations
└── langgraph.json           # LangGraph configuration
```

### CI/CD Workflow

1. **Tests run** (unit, integration, e2e, offline evals)
2. **Docker image built** (using Docker Hub or GHCR)
3. **Image pushed** to container registry
4. **Preview deployment created** via Control Plane API (for PRs)
5. **Production deployment created** via Control Plane API (on merge)
6. **Online evaluations** run on deployed agents

## Notes

### Detailed Documentation

- [**Container Registry Options**](./container-registry-options.md) - Comprehensive guide on using different container registries with LangSmith:
  - Docker Hub (current implementation)
  - GitHub Container Registry (GHCR) - Recommended alternative
  - AWS ECR, Azure ACR, GCP Artifact Registry
  - Authentication setup for private registries
  - Step-by-step migration guide from Docker Hub to GHCR
  - Troubleshooting common issues
  - Cost and rate limit comparisons

