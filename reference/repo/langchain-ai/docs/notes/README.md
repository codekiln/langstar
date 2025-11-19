# docs

## Repository Information

- **Repository**: [langchain-ai/docs](https://github.com/langchain-ai/docs)
- **Date Created**: 2025-11-19
- **Cloned to**: `../reference/repo/langchain-ai/docs/code`

## Purpose

Studying the LangChain documentation repository to understand deployment patterns for LangSmith and LangGraph applications, particularly CI/CD pipeline implementations.

## Key Findings

### Control Plane API vs UI Deployment

The documentation distinguishes between two deployment methods:
- **Method 1 (UI)**: Manual deployment through web interface
- **Method 2 (Control Plane API)**: Programmatic deployment for CI/CD automation

**Critical distinction**: Method 2 is essential for production-grade automated deployments.

### API Endpoint Confusion

Common mistake: There are TWO different APIs with different endpoints:
- **LangSmith API** (traces, evals): `api.smith.langchain.com`
- **Control Plane API** (deployments): `api.host.langchain.com`

Using the wrong endpoint causes deployment failures.

## Architecture

### CI/CD Pipeline Structure

The langchain-ai/cicd-pipeline-example repository demonstrates:
- Automated testing (unit, integration, e2e, offline evals)
- Preview deployments on PR creation
- Production deployments on merge
- Online evaluation monitoring
- Quality gates and alerting

## Notes

### Detailed Documentation

- [**Control Plane API Deployment**](./control-plane-api-deployment.md) - Comprehensive notes on Method 2: Control Plane API, including:
  - Cloud vs Self-Hosted deployment approaches
  - Docker image building for self-hosted
  - Container registry options (ECR, ACR, GCR, Docker Hub)
  - Preview vs Production deployment types
  - API endpoints and common pitfalls
  - CI/CD integration patterns

