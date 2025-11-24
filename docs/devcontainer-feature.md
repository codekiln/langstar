# Langstar DevContainer Feature

Official DevContainer feature for installing the [Langstar CLI](https://github.com/codekiln/langstar) in development containers.

## Overview

The Langstar DevContainer feature provides automatic installation of the Langstar CLI in any devcontainer environment, including:

- **VS Code Dev Containers** - Local development with consistent environments
- **GitHub Codespaces** - Cloud-based development workspaces
- **devpod** - Client-only development container runtime
- **Any OCI-compliant devcontainer implementation**

## Quick Start

Add to your `.devcontainer/devcontainer.json`:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

## Feature Options

### `version` (string)

Specify which version of Langstar CLI to install.

| Value | Description | Example |
|-------|-------------|---------|
| `"latest"` | Most recent release (default) | `"latest"` |
| Specific version | Pin to exact version | `"v0.4.0"`, `"v0.3.0"` |

**Default:** `"latest"`

**Example:**
```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

## What Gets Installed

- **Binary:** `/usr/local/bin/langstar`
- **Architecture Support:**
  - Linux x86_64 (amd64) ✅
  - Linux ARM64 (aarch64) ✅
  - macOS ARM64 (Apple Silicon) ✅

## Compatibility

### Base Images

The feature has been tested and verified on:

| Distribution | Versions | Status |
|--------------|----------|--------|
| Ubuntu | 22.04, 24.04 | ✅ Fully tested |
| Debian | 11 (bullseye), 12 (bookworm) | ✅ Fully tested |
| Alpine | 3.18, 3.19 | ✅ Fully tested |

The feature should work on any Linux-based devcontainer image.

### Devcontainer Implementations

| Implementation | Support |
|----------------|---------|
| VS Code Dev Containers | ✅ Supported |
| GitHub Codespaces | ✅ Supported |
| devpod | ✅ Supported |
| Docker | ✅ Supported |
| Podman | ✅ Supported (with devcontainer CLI) |

## Installation Examples

### Basic Usage

Minimal configuration using defaults:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    }
}
```

### With Version Pinning

Pin to a specific version for reproducibility:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

### Multi-Feature Configuration

Combine with other devcontainer features:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/devcontainers/features/common-utils:2": {
            "installZsh": true,
            "installOhMyZsh": true
        },
        "ghcr.io/devcontainers/features/github-cli:1": {
            "version": "latest"
        },
        "ghcr.io/devcontainers/features/rust:1": {
            "version": "latest"
        },
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

### Real-World Example

Complete devcontainer configuration with environment variables:

```jsonc
{
    "name": "LangGraph Development",
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/devcontainers/features/common-utils:2": {
            "installZsh": true
        },
        "ghcr.io/devcontainers/features/github-cli:1": {},
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"
    },
    "postCreateCommand": "langstar --version && langstar config"
}
```

## Configuration After Installation

After the feature installs Langstar, you'll need to configure it with your LangSmith credentials.

### Method 1: Environment Variables (Recommended)

Use devcontainer `remoteEnv` to pass credentials from your local environment:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}",
        "LANGSMITH_ORGANIZATION_ID": "${localEnv:LANGSMITH_ORGANIZATION_ID}",
        "LANGSMITH_WORKSPACE_ID": "${localEnv:LANGSMITH_WORKSPACE_ID}"
    }
}
```

Set local environment variables in your shell:

```bash
# ~/.bashrc or ~/.zshrc
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"  # Optional
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"  # Optional
```

### Method 2: Secrets (for GitHub Codespaces)

For GitHub Codespaces, use repository or organization secrets:

1. Go to your repository settings → Secrets and variables → Codespaces
2. Add secrets:
   - `LANGSMITH_API_KEY`
   - `LANGSMITH_ORGANIZATION_ID` (optional)
   - `LANGSMITH_WORKSPACE_ID` (optional)

Reference in `devcontainer.json`:

```jsonc
{
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${secret:LANGSMITH_API_KEY}"
    }
}
```

### Method 3: Configuration File

Create a configuration file inside the container:

```bash
# Inside the devcontainer
mkdir -p ~/.langstar
cat > ~/.langstar/config.toml << 'EOF'
[langstar]
langsmith_api_key = "<your-api-key>"
organization_id = "<your-org-id>"  # Optional
workspace_id = "<your-workspace-id>"  # Optional
EOF
```

### Verify Configuration

After starting your devcontainer:

```bash
# Check Langstar is installed
langstar --version

# Verify configuration
langstar config

# Test with a simple command
langstar prompt list
```

## Version Management

### Using Latest Version

Always use the most recent release:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

**Pros:**
- Always up-to-date with latest features and fixes
- No manual version management

**Cons:**
- May introduce breaking changes (follow semantic versioning)
- Less reproducible across team members

### Using Pinned Versions

Lock to a specific version for reproducibility:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

**Pros:**
- Reproducible builds across team
- No surprise breaking changes
- Better for CI/CD pipelines

**Cons:**
- Manual updates required
- May miss bug fixes and new features

### Recommended Strategy

For most projects, use pinned versions in production and `latest` for development:

```jsonc
// devcontainer.json (development)
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}

// devcontainer.prod.json (production/CI)
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

## How It Works

### Installation Process

When the feature runs:

1. **Version Resolution**
   - Resolves `"latest"` to current GitHub release
   - Validates specific version exists

2. **Architecture Detection**
   - Detects container architecture (x86_64, ARM64)
   - Selects appropriate binary

3. **Binary Download**
   - Downloads from GitHub Releases
   - Verifies SHA256 checksums

4. **Installation**
   - Installs to `/usr/local/bin/langstar`
   - Makes binary executable
   - Verifies installation with `langstar --version`

### Implementation Details

The feature uses the official Langstar install script:

```bash
curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash -s -- --prefix /usr/local
```

Source code:
- Feature definition: `.devcontainer/features/langstar/devcontainer-feature.json`
- Install script: `.devcontainer/features/langstar/install.sh`
- Published to: `ghcr.io/codekiln/langstar/langstar:1`

## Troubleshooting

### Feature Fails to Install

**Problem:** Installation fails during container build

**Solutions:**

1. Check network connectivity:
   ```bash
   curl -I https://github.com
   ```

2. Verify feature version exists:
   ```bash
   # Check available releases
   gh release list --repo codekiln/langstar
   ```

3. Check container logs:
   ```bash
   # In VS Code: View → Output → Dev Containers
   # Or rebuild with verbose logging:
   docker build --progress=plain .
   ```

### Binary Not Found After Installation

**Problem:** `langstar: command not found`

**Solutions:**

1. Verify installation location:
   ```bash
   ls -la /usr/local/bin/langstar
   ```

2. Check PATH includes `/usr/local/bin`:
   ```bash
   echo $PATH
   ```

3. Restart the terminal or source your shell config:
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   ```

### Wrong Architecture Binary

**Problem:** Binary doesn't match container architecture

**Solutions:**

1. Check container architecture:
   ```bash
   uname -m  # Should be x86_64 or aarch64
   ```

2. Check binary type:
   ```bash
   file /usr/local/bin/langstar
   ```

3. If mismatch, rebuild container:
   ```bash
   # In VS Code: Dev Containers: Rebuild Container
   # Or via CLI:
   devcontainer build --workspace-folder .
   ```

### Configuration Not Working

**Problem:** `langstar config` shows no credentials

**Solutions:**

1. Verify environment variables are set:
   ```bash
   env | grep LANGSMITH
   ```

2. Check `remoteEnv` in `devcontainer.json`:
   ```jsonc
   {
       "remoteEnv": {
           "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"
       }
   }
   ```

3. Verify local environment has the variables:
   ```bash
   # On host machine
   echo $LANGSMITH_API_KEY
   ```

4. Rebuild container to pick up new environment:
   ```bash
   # In VS Code: Dev Containers: Rebuild Container
   ```

## CI/CD Integration

### GitHub Actions Example

Use the feature in GitHub Actions workflows:

```yaml
name: Test with Langstar
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup DevContainer CLI
        run: npm install -g @devcontainers/cli

      - name: Build and Run DevContainer
        run: |
          devcontainer up --workspace-folder .
          devcontainer exec --workspace-folder . langstar --version
          devcontainer exec --workspace-folder . langstar prompt list
        env:
          LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
```

### GitLab CI Example

```yaml
test:
  image: docker:latest
  services:
    - docker:dind
  before_script:
    - npm install -g @devcontainers/cli
  script:
    - devcontainer build --workspace-folder .
    - devcontainer exec --workspace-folder . langstar --version
  variables:
    LANGSMITH_API_KEY: ${LANGSMITH_API_KEY}
```

## Release Information

### Current Version

- **Feature Version:** `1.0.0`
- **Feature ID:** `langstar`
- **Registry:** `ghcr.io/codekiln/langstar/langstar:1`

### Version Compatibility

| Feature Version | Langstar CLI Versions | Status |
|----------------|----------------------|---------|
| `1.x.x` | `v0.3.0` and above | Active |

### Update Notifications

The feature automatically uses the latest available version when `"version": "latest"` is specified. No manual updates required.

To check for updates:

```bash
# Inside devcontainer
langstar --version

# Check latest release
curl -s https://api.github.com/repos/codekiln/langstar/releases/latest | grep '"tag_name"'
```

## Advanced Usage

### Custom Base Images

The feature works with custom base images:

```dockerfile
# Dockerfile
FROM mcr.microsoft.com/devcontainers/base:ubuntu
RUN apt-get update && apt-get install -y curl git
```

```jsonc
// devcontainer.json
{
    "build": {
        "dockerfile": "Dockerfile"
    },
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    }
}
```

### Post-Install Commands

Run commands after feature installation:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "postCreateCommand": "langstar --version && langstar config"
}
```

### Lifecycle Hooks

Execute custom scripts at different container lifecycle stages:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "postCreateCommand": "echo 'Container created'",
    "postStartCommand": "echo 'Container started'",
    "postAttachCommand": "langstar config"
}
```

## Security Considerations

### Credential Management

**Do NOT:**
- Hard-code API keys in `devcontainer.json`
- Commit secrets to version control
- Share API keys in plain text

**Do:**
- Use environment variables with `${localEnv:VAR}`
- Use secrets for Codespaces (`${secret:VAR}`)
- Use vault solutions for team environments
- Rotate API keys regularly

### Binary Verification

The install script automatically verifies SHA256 checksums of downloaded binaries. This ensures:

- Binary integrity (not corrupted)
- Binary authenticity (from official releases)
- Protection against MITM attacks

## Support

### Getting Help

If you encounter issues:

1. **Check this documentation** for common solutions
2. **Search existing issues:** [GitHub Issues](https://github.com/codekiln/langstar/issues)
3. **Open a new issue** with:
   - DevContainer configuration
   - Base image and architecture
   - Error messages and logs
   - Steps to reproduce

### Useful Resources

- **Main Repository:** https://github.com/codekiln/langstar
- **Feature Source:** `.devcontainer/features/langstar/`
- **Install Script:** `scripts/install.sh`
- **CI Testing:** See `.github/workflows/test-devcontainer-feature.yml`

### Contributing

Contributions welcome! See:
- [GitHub Workflow](./dev/github-workflow.md)
- [Development Guidelines](./dev/README.md)

## Related Documentation

- [Installation Guide](./installation.md) - All installation methods
- [Configuration Guide](./configuration.md) - Detailed configuration options
- [Architecture Overview](./architecture.md) - System design and principles
- [Troubleshooting Guide](./troubleshooting.md) - Common issues and solutions
