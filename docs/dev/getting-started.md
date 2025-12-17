# Getting Started

This guide covers setting up the Langstar development environment for first-time contributors. The project uses devcontainers for consistent development environments across different IDEs and platforms.

## Prerequisites

- Docker Desktop installed and running (local development)
- One of:
  - VS Code with Dev Containers extension
  - JetBrains IDE with Gateway/Remote Development
  - GitHub account (for Codespaces)

## Quick Start by Environment

Choose your development environment:

| Environment | Setup Time | Best For |
|-------------|-----------|----------|
| [GitHub Codespaces](#github-codespaces) | ~5 min | Quick start, no local setup |
| [VS Code Devcontainer](#vs-code-devcontainer) | ~10 min | Full-featured local development |
| [JetBrains Devcontainer](#jetbrains-devcontainer) | ~10 min | RustRover, IntelliJ users |

---

## GitHub Codespaces

The fastest way to get started. No local Docker installation required.

### Step 1: Configure Codespaces Secrets

1. Go to the repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Codespaces**
3. Add the following secrets:

| Secret Name | Required | Description |
|-------------|----------|-------------|
| `GH_PAT` | Yes | GitHub Personal Access Token with `repo` scope |
| `GH_USER` | Yes | Your GitHub username |
| `AWS_ACCESS_KEY_ID` | Yes | AWS access key for Bedrock |
| `AWS_SECRET_ACCESS_KEY` | Yes | AWS secret access key |
| `LANGSMITH_API_KEY` | Optional | LangSmith API key for testing |
| `GH_PROJECT_PAT` | Optional | GitHub PAT with project permissions |

### Step 2: Create a Codespace

1. Go to the repository on GitHub
2. Click the green **Code** button
3. Select the **Codespaces** tab
4. Click **Create codespace on main** (or your target branch)

The codespace will automatically:
- Clone the repository
- Build the devcontainer
- Configure git authentication
- Install all development tools

### Step 3: Verify Setup

Once the codespace is ready, verify the setup:

```bash
# Check git authentication
gh auth status

# Verify environment variables
printenv | grep -E 'LANGSMITH|AWS' | cut -d= -f1

# Run a test build
cargo check --workspace
```

---

## VS Code Devcontainer

For local development with full Docker support.

### Step 1: Clone the Repository

```bash
git clone https://github.com/codekiln/langstar.git
cd langstar
```

### Step 2: Configure Secrets

```bash
cd .devcontainer
cp .env.default .env
```

Edit `.env` with your actual credentials:

```bash
# Required
GITHUB_PAT=ghp_YourActualTokenHere
GITHUB_USER=your_github_username
AWS_ACCESS_KEY_ID=your_aws_access_key
AWS_SECRET_ACCESS_KEY=your_aws_secret_key

# Optional
LANGSMITH_API_KEY=lsv2_YourActualKeyHere
GITHUB_PROJECT_PAT=ghp_YourProjectTokenHere
```

### Step 3: Open in Container

1. Open the repository folder in VS Code
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Select **Dev Containers: Reopen in Container**
4. Wait for the container to build (first time takes ~5 minutes)

### Step 4: Initialize Workspace

After the container starts, the `/workspace` directory uses a named Docker volume for optimal file watching performance. Clone the repository into the workspace:

```bash
# Inside the container terminal
cd /workspace
git clone https://github.com/codekiln/langstar.git .
```

> **Note**: The named volume architecture eliminates file watching issues but requires this initial clone step. Your code lives inside the Docker volume, providing native filesystem performance.

### Step 5: Verify Setup

```bash
# Check git authentication
gh auth status

# Verify environment variables
printenv | grep -E 'LANGSMITH|AWS' | cut -d= -f1

# Run a test build
cargo check --workspace
```

### Alternative: Clone Repository in Container Volume

VS Code offers a streamlined workflow that handles cloning automatically:

1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Select **Dev Containers: Clone Repository in Container Volume...**
3. Enter the repository URL: `https://github.com/codekiln/langstar`
4. Wait for clone and container build

> **Note**: With this method, you'll need to configure secrets inside the container or use VS Code's Remote Containers settings to pass environment variables.

---

## JetBrains Devcontainer

For RustRover, IntelliJ IDEA, and other JetBrains IDEs.

### Step 1: Clone the Repository

```bash
git clone https://github.com/codekiln/langstar.git
cd langstar
```

### Step 2: Configure Secrets

```bash
cd .devcontainer
cp .env.default .env
```

Edit `.env` with your actual credentials (same as VS Code setup above).

### Step 3: Open with Gateway

1. Open JetBrains Gateway
2. Select **Dev Containers** connection type
3. Choose **Open Folder in Container**
4. Navigate to the cloned repository folder
5. Wait for the container to build

### Step 4: Initialize Workspace

After the container starts, clone the repository into the workspace volume:

```bash
# Inside the container terminal
cd /workspace
git clone https://github.com/codekiln/langstar.git .
```

### Step 5: Verify Setup

```bash
# Check git authentication
gh auth status

# Verify environment variables
printenv | grep -E 'LANGSMITH|AWS' | cut -d= -f1

# Run a test build
cargo check --workspace
```

### Why Named Volumes for JetBrains

JetBrains IDEs rely on `inotify` for file watching. With bind mounts on macOS, Docker uses gRPC FUSE which doesn't provide full inotify support, causing:
- "External file changes sync might be slow" warning
- Unreliable file change detection
- Potential IDE performance issues

The named volume approach keeps files in the container's native filesystem, providing proper inotify support and eliminating these issues.

---

## Understanding the Architecture

### Secrets Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         HOST MACHINE                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  .devcontainer/                                          │   │
│  │  ├── .env           ← Your secrets (gitignored)         │   │
│  │  ├── docker-compose.yml  ← Reads .env for substitution  │   │
│  │  └── devcontainer.json                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                    Docker Compose                               │
│                    substitutes ${VARS}                          │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    CONTAINER                             │   │
│  │  Environment variables set via docker-compose.yml        │   │
│  │  ├── GITHUB_PAT=<from .env>                             │   │
│  │  ├── AWS_ACCESS_KEY_ID=<from .env>                      │   │
│  │  └── LANGSMITH_API_KEY=<from .env>                      │   │
│  │                                                          │   │
│  │  /workspace  ← Named Docker volume (code lives here)    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

**Key insight**: Docker Compose reads `.env` from the **host** filesystem before starting the container. Environment variables are passed into the container via the `environment:` section in `docker-compose.yml`. The `.env` file itself doesn't need to be inside the container.

### Workspace Volume Architecture

The devcontainer uses a **named Docker volume** for `/workspace` instead of a bind mount:

| Approach | File Location | File Watching | Performance |
|----------|---------------|---------------|-------------|
| Bind mount | Host filesystem | Limited (gRPC FUSE) | Variable |
| Named volume | Container filesystem | Full inotify | Optimal |

**Benefits of named volumes:**
- Proper `inotify` support for all IDEs
- No gRPC FUSE layer on macOS
- Consistent performance across platforms
- Eliminates "External file changes sync might be slow" warning

**Trade-off:**
- Code lives only inside the container volume
- Requires initial clone into the container
- Can't edit files with host-based editors outside the container

### Volume Persistence

The named volume persists across container rebuilds:

```bash
# Inspect volume
docker volume inspect langstar-workspace

# Remove volume (deletes all workspace data!)
docker volume rm langstar-workspace
```

---

## Troubleshooting

### Container Build Fails

1. Ensure Docker Desktop is running
2. Check `.env` file exists and has actual values (not placeholders)
3. Try rebuilding without cache:
   ```bash
   docker-compose -f .devcontainer/docker-compose.yml build --no-cache
   ```

### Environment Variables Not Available

1. Verify `.devcontainer/.env` exists with actual values
2. Check variables are declared in `docker-compose.yml` environment section
3. Rebuild the container: **Dev Containers: Rebuild Container**
4. Test substitution:
   ```bash
   cd .devcontainer
   docker-compose config | grep -A 20 environment:
   ```

### Git Authentication Fails

1. Verify `GITHUB_PAT` (local) or `GH_PAT` (Codespaces) is set correctly
2. Check the token has `repo` scope
3. Run setup manually: `bash .devcontainer/setup-github-auth.sh`
4. Check credential store:
   ```bash
   git credential-store --file ~/.git-credentials get <<< "protocol=https
   host=github.com"
   ```

### Workspace is Empty

If `/workspace` is empty after container starts:

```bash
# Clone the repository into the workspace
cd /workspace
git clone https://github.com/codekiln/langstar.git .
```

This is expected behavior with named volumes. The volume starts empty and you clone the repo into it.

---

## Next Steps

After setup is complete:

1. Review [GitHub Workflow](./github-workflow.md) for development practices
2. Check [Testing Standards](./testing/README.md) for test requirements
3. Read [Git SCM Conventions](./git-scm-conventions.md) for commit formatting
4. See [Environment Variables](./environment-variables.md) for API authentication details

---

## Environment Variables Reference

For complete environment variable documentation, see [environment-variables.md](./environment-variables.md).

### Quick Reference

| Variable | Local Name | Codespaces Name | Purpose |
|----------|------------|-----------------|---------|
| GitHub PAT | `GITHUB_PAT` | `GH_PAT` | Git operations |
| GitHub User | `GITHUB_USER` | `GH_USER` | Git config |
| AWS Access Key | `AWS_ACCESS_KEY_ID` | Same | Bedrock auth |
| AWS Secret Key | `AWS_SECRET_ACCESS_KEY` | Same | Bedrock auth |
| LangSmith API Key | `LANGSMITH_API_KEY` | Same | LangSmith API |
