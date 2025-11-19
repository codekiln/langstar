# Container Registry Options for LangSmith Deployments

**Sources**:
- `langchain-ai/docs`: `src/langsmith/deploy-with-control-plane.mdx`
- `langchain-ai/cicd-pipeline-example`: `.github/workflows/preview-deployment.yml`

## Overview

LangSmith deployments work with **any container registry accessible by your Kubernetes cluster**. The key requirement is that your infrastructure can pull images from the registry.

## Supported Container Registries

LangSmith documentation explicitly mentions:
- ✅ **AWS ECR** (Elastic Container Registry)
- ✅ **Azure ACR** (Azure Container Registry)
- ✅ **GCP Artifact Registry** (Google Cloud)
- ✅ **Docker Hub** (docker.io) - Used in cicd-pipeline-example
- ✅ **Private Docker registries**
- ✅ **GitHub Container Registry (GHCR)** - Mentioned in Flyte integration docs

## Key Principle

> "Push your image to a container registry accessible by your Kubernetes cluster. The specific commands depend on your registry provider."
>
> — deploy-with-control-plane.mdx

**Translation**: As long as your K8s cluster can authenticate and pull from a registry, LangSmith can use it.

## Authentication Requirements

### Public Registries (No Auth Required)
If your images are public, no special configuration is needed.

### Private Registries (Auth Required)
For private registries, you must configure **Kubernetes image pull secrets** as part of infrastructure setup:

**Self-Hosted LangSmith:**
- Configure `imagePullSecrets` in Helm chart's `values.yaml`
- See: [Enable LangSmith Deployment guide](/langsmith/deploy-self-hosted-full-platform#setup)

**Hybrid LangSmith:**
- Configure `imagePullSecrets` in `langgraph-dataplane-values.yaml`

**Reference**: [Kubernetes docs on pulling from private registries](https://kubernetes.io/docs/tasks/configure-pod-container/pull-image-private-registry/)

## GitHub Container Registry (GHCR) Support

### Is GHCR Supported?

**Yes!** GHCR is mentioned explicitly in the Flyte integration documentation:

> "[Docker Hub](https://hub.docker.com/) or [GitHub Container Registry (GHCR)](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry) is a convenient option to begin with."
>
> — src/oss/python/integrations/providers/flyte.mdx (line 72)

### Why Use GHCR?

**Benefits:**
1. **Free for public repositories** - Unlimited bandwidth
2. **Integrated with GitHub Actions** - No separate credentials needed
3. **Native authentication** - Uses `GITHUB_TOKEN` automatically
4. **No Docker Hub rate limits** - Docker Hub has pull rate limits on free tier
5. **Version control integration** - Images tied to GitHub releases
6. **Enterprise-friendly** - GitHub Enterprise includes private registries

### GHCR vs Docker Hub

| Feature | Docker Hub | GitHub Container Registry |
|---------|-----------|---------------------------|
| **Public images** | Free | Free |
| **Private images** | Limited free tier | Free with GitHub account |
| **Pull rate limits** | 200 pulls/6hrs (anonymous), 100/6hrs (free auth) | Unlimited |
| **Integration** | Separate credentials | Built into GitHub |
| **URL format** | `docker.io/username/image` | `ghcr.io/username/image` |
| **GitHub Actions auth** | Requires secrets | Uses `GITHUB_TOKEN` |

## Example: Using Docker Hub (Current cicd-pipeline-example)

**From `.github/workflows/preview-deployment.yml`:**

```yaml
env:
  REGISTRY: docker.io
  IMAGE_NAME: perinim98/text2sql-agent

steps:
  - name: Log in to Docker Hub
    uses: docker/login-action@v3
    with:
      registry: ${{ env.REGISTRY }}
      username: ${{ secrets.DOCKER_USERNAME }}
      password: ${{ secrets.DOCKER_PASSWORD }}

  - name: Build and push preview Docker image
    uses: docker/build-push-action@v5
    with:
      push: true
      tags: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:preview-${{ github.event.pull_request.number }}
      platforms: linux/amd64,linux/arm64
```

**Required GitHub Secrets:**
- `DOCKER_USERNAME`
- `DOCKER_PASSWORD`

## Example: Using GitHub Container Registry (GHCR)

To use GHCR instead of Docker Hub, modify the workflow:

### Changes Required

```yaml
env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}  # Automatically uses owner/repo format

steps:
  - name: Log in to GitHub Container Registry
    uses: docker/login-action@v3
    with:
      registry: ghcr.io
      username: ${{ github.actor }}
      password: ${{ secrets.GITHUB_TOKEN }}  # Built-in, no setup needed!

  - name: Build and push preview Docker image
    uses: docker/build-push-action@v5
    with:
      push: true
      tags: ghcr.io/${{ github.repository }}:preview-${{ github.event.pull_request.number }}
      platforms: linux/amd64,linux/arm64
```

### Key Differences

1. **Registry URL**: `docker.io` → `ghcr.io`
2. **Authentication**: Custom secrets → Built-in `GITHUB_TOKEN`
3. **Username**: Static username → `${{ github.actor }}`
4. **Image name**: Can use `${{ github.repository }}` for automatic naming
5. **No secrets setup required** (for public repos)

### Image Naming Convention

**Docker Hub:**
```
docker.io/username/image-name:tag
```

**GHCR:**
```
ghcr.io/owner/repo-name:tag
```

Example:
- Docker Hub: `docker.io/perinim98/text2sql-agent:latest`
- GHCR: `ghcr.io/langchain-ai/cicd-pipeline-example:latest`

## Deploying GHCR Images to LangSmith

Once you've pushed an image to GHCR, deploy it the same way as any other registry:

### Via Control Plane UI

1. Navigate to **Deployments** in LangSmith UI
2. Click **+ New Deployment**
3. Enter **Image URL**: `ghcr.io/owner/repo:tag`
4. Select Listener/Compute ID
5. Configure environment variables
6. Click **Submit**

### Via Control Plane API

```python
deployment = client.create_deployment(
    deployment_name="my-agent",
    git_repo=None,  # Not using GitHub integration
    git_branch=None,
    config={
        "image_uri": "ghcr.io/owner/repo:tag",  # GHCR image
    },
    secrets=[
        {"name": "OPENAI_API_KEY", "value": "..."},
    ],
)
```

## Authentication for Private GHCR Images

If your GHCR images are private, configure Kubernetes image pull secrets:

### Step 1: Create GitHub Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate token with `read:packages` scope
3. Save the token securely

### Step 2: Create Kubernetes Secret

```bash
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<github-username> \
  --docker-password=<github-pat> \
  --docker-email=<email> \
  --namespace=<namespace>
```

### Step 3: Configure LangSmith

**For Self-Hosted:**

In `values.yaml`:
```yaml
deployment:
  enabled: true
  imagePullSecrets:
    - name: ghcr-secret
```

**For Hybrid:**

In `langgraph-dataplane-values.yaml`:
```yaml
imagePullSecrets:
  - name: ghcr-secret
```

## Best Practices

### 1. Version Tagging

Always tag images with version information for easy rollbacks:

```yaml
# ❌ Bad - overwriting 'latest' makes rollback impossible
tags: ghcr.io/owner/repo:latest

# ✅ Good - semantic versioning allows rollbacks
tags: |
  ghcr.io/owner/repo:v1.2.3
  ghcr.io/owner/repo:latest
```

### 2. Multi-Platform Builds

Build for both AMD and ARM architectures for flexibility:

```yaml
platforms: linux/amd64,linux/arm64
```

### 3. Image Caching

Use GitHub Actions cache to speed up builds:

```yaml
cache-from: type=gha
cache-to: type=gha,mode=max
```

### 4. Public vs Private Images

**Public GHCR images:**
- No authentication needed for pulls
- Free bandwidth
- Suitable for open-source projects

**Private GHCR images:**
- Requires PAT for pulls (via image pull secrets)
- Better for proprietary code
- Same performance as public

### 5. Image Visibility

Make GHCR packages public after first push:
1. Go to GitHub → Packages → Select your package
2. Click "Package settings"
3. Change visibility to "Public"

## Troubleshooting

### Issue: "Failed to pull image from ghcr.io"

**Causes:**
1. Image doesn't exist at that URL
2. Image is private and no image pull secret configured
3. PAT has insufficient permissions

**Solutions:**
- Verify image URL: `docker pull ghcr.io/owner/repo:tag`
- Check image visibility in GitHub Packages
- Ensure PAT has `read:packages` scope
- Verify image pull secret is created and referenced

### Issue: "403 Forbidden" when pulling

**Cause:** PAT lacks `read:packages` permission

**Solution:** Regenerate PAT with correct scopes

### Issue: "Authentication required"

**Cause:** Private image without image pull secrets

**Solution:** Follow [Authentication for Private GHCR Images](#authentication-for-private-ghcr-images)

## Comparison Table

| Registry | URL Format | Auth in GHA | Rate Limits | Cost (Private) |
|----------|-----------|-------------|-------------|----------------|
| **Docker Hub** | `docker.io/user/image` | Secrets required | 200/6h (anon), 100/6h (free) | $5/month (1 private repo) |
| **GHCR** | `ghcr.io/owner/repo` | Built-in `GITHUB_TOKEN` | Unlimited | Free |
| **AWS ECR** | `<account>.dkr.ecr.<region>.amazonaws.com/repo` | AWS credentials | Unlimited | $0.10/GB storage |
| **Azure ACR** | `<registry>.azurecr.io/repo` | Azure credentials | Unlimited | $5/month (Basic) |
| **GCP Artifact Registry** | `<region>-docker.pkg.dev/<project>/<repo>` | GCP credentials | Unlimited | $0.10/GB storage |

## Conclusion

**Yes, you can absolutely use GitHub Container Registry (ghcr.io) with LangSmith deployments.**

The cicd-pipeline-example currently uses Docker Hub, but switching to GHCR is straightforward and offers several advantages:

✅ **No additional secrets needed** (uses built-in `GITHUB_TOKEN`)
✅ **No rate limits** (unlike Docker Hub's 200 pulls/6 hours)
✅ **Free for both public and private images**
✅ **Integrated with GitHub Actions**
✅ **Better for GitHub-based workflows**

The only configuration needed is changing the registry URL and authentication method in your GitHub Actions workflows.
