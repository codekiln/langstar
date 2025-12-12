# DevContainer Feature Usage Examples

This document provides real-world examples of using the Langstar DevContainer feature in different scenarios.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Version Management](#version-management)
- [Multi-Feature Configurations](#multi-feature-configurations)
- [Environment-Specific Configurations](#environment-specific-configurations)
- [Team Collaboration](#team-collaboration)
- [CI/CD Integration](#cicd-integration)

## Basic Examples

### Minimal Configuration

The simplest possible setup:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    }
}
```

This:

- Uses Ubuntu base image
- Installs latest Langstar version
- Ready to use after container creation

### With Environment Variables

Pass credentials from your local environment:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"
    }
}
```

Set on your host machine:

```bash
# ~/.bashrc or ~/.zshrc
export LANGSMITH_API_KEY="<your-api-key>"
```

### With Verification

Verify installation after container creation:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "postCreateCommand": "langstar --version && langstar config"
}
```

## Version Management

### Latest Version (Development)

Always use the newest version for development:

```jsonc
{
    "name": "Langstar Development",
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

**Use case:** Personal development, exploring new features

### Pinned Version (Production)

Lock to a specific version for stability:

```jsonc
{
    "name": "Langstar Production",
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

**Use case:** Team projects, CI/CD, production deployments

### Version Testing

Test multiple versions using different configs:

```jsonc
// .devcontainer/devcontainer.json (default - latest)
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}

// .devcontainer/devcontainer-stable.json (pinned)
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

Switch between configs in VS Code:

1. Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
2. Run "Dev Containers: Open Container Configuration File"
3. Select desired configuration

## Multi-Feature Configurations

### Rust Development Environment

Complete Rust setup with Langstar:

```jsonc
{
    "name": "Rust + Langstar",
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/devcontainers/features/rust:1": {
            "version": "latest",
            "profile": "default"
        },
        "ghcr.io/devcontainers/features/common-utils:2": {
            "installZsh": true,
            "installOhMyZsh": true,
            "username": "vscode"
        },
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "customizations": {
        "vscode": {
            "extensions": [
                "rust-lang.rust-analyzer",
                "tamasfe.even-better-toml"
            ]
        }
    }
}
```

### Python + LangChain Development

Python environment for LangChain development:

```jsonc
{
    "name": "Python LangChain Development",
    "image": "mcr.microsoft.com/devcontainers/python:3.11",
    "features": {
        "ghcr.io/devcontainers/features/python:1": {
            "version": "3.11",
            "installJupyterlab": true
        },
        "ghcr.io/devcontainers/features/git:1": {},
        "ghcr.io/devcontainers/features/github-cli:1": {},
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "postCreateCommand": "pip install langchain langgraph langsmith",
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"
    }
}
```

### Full-Stack LangGraph Development

Complete environment for LangGraph Cloud projects:

```jsonc
{
    "name": "LangGraph Full Stack",
    "image": "mcr.microsoft.com/devcontainers/typescript-node:20",
    "features": {
        "ghcr.io/devcontainers/features/node:1": {
            "version": "20"
        },
        "ghcr.io/devcontainers/features/python:1": {
            "version": "3.11"
        },
        "ghcr.io/devcontainers/features/docker-in-docker:2": {},
        "ghcr.io/devcontainers/features/github-cli:1": {},
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "postCreateCommand": "npm install && pip install -r requirements.txt",
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}",
        "LANGSMITH_ORGANIZATION_ID": "${localEnv:LANGSMITH_ORGANIZATION_ID}"
    },
    "mounts": [
        "source=/var/run/docker.sock,target=/var/run/docker.sock,type=bind"
    ]
}
```

## Environment-Specific Configurations

### Local Development

For local development with full credentials:

```jsonc
{
    "name": "Local Development",
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}",
        "LANGSMITH_ORGANIZATION_ID": "${localEnv:LANGSMITH_ORGANIZATION_ID}",
        "LANGSMITH_WORKSPACE_ID": "${localEnv:LANGSMITH_WORKSPACE_ID}"
    },
    "postCreateCommand": "langstar config"
}
```

### GitHub Codespaces

Using GitHub secrets for Codespaces:

```jsonc
{
    "name": "GitHub Codespaces",
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${secret:LANGSMITH_API_KEY}"
    }
}
```

**Setup GitHub Secrets:**

1. Repository Settings → Secrets and variables → Codespaces
2. Add `LANGSMITH_API_KEY`

### CI/CD Environment

Minimal config for CI/CD with pinned version:

```jsonc
{
    "name": "CI Environment",
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

Credentials passed via CI environment variables.

## Team Collaboration

### Shared Team Configuration

Standard setup for team consistency:

```jsonc
{
    "name": "Team Development Environment",
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/devcontainers/features/common-utils:2": {
            "installZsh": true,
            "username": "vscode"
        },
        "ghcr.io/devcontainers/features/github-cli:1": {},
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    },
    "customizations": {
        "vscode": {
            "extensions": [
                "rust-lang.rust-analyzer",
                "github.copilot"
            ],
            "settings": {
                "terminal.integrated.defaultProfile.linux": "zsh"
            }
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"
    },
    "postCreateCommand": ".devcontainer/setup.sh"
}
```

With setup script:

```bash
#!/bin/bash
# .devcontainer/setup.sh

set -e

echo "Setting up team environment..."

# Install project dependencies
cargo build

# Verify Langstar installation
langstar --version

# Display configuration help
echo ""
echo "To configure Langstar:"
echo "  1. Set LANGSMITH_API_KEY in your local environment"
echo "  2. Run: langstar config"
echo "  3. Test with: langstar prompt list"
```

### With Organization Scoping

Team configuration with organization scoping:

```jsonc
{
    "name": "Organization Environment",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}",
        "LANGSMITH_ORGANIZATION_ID": "${localEnv:LANGSMITH_ORGANIZATION_ID}",
        "LANGSMITH_ORGANIZATION_NAME": "ACME Corp"
    },
    "postCreateCommand": "langstar prompt list --organization-id ${LANGSMITH_ORGANIZATION_ID}"
}
```

## CI/CD Integration

### GitHub Actions

Use devcontainer in GitHub Actions:

```yaml
name: Test with DevContainer
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node (for devcontainer CLI)
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install DevContainer CLI
        run: npm install -g @devcontainers/cli

      - name: Build DevContainer
        run: devcontainer build --workspace-folder .

      - name: Test Langstar in DevContainer
        run: |
          devcontainer exec --workspace-folder . langstar --version
          devcontainer exec --workspace-folder . langstar prompt list
        env:
          LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
```

### GitLab CI

```yaml
test:
  image: docker:latest
  services:
    - docker:dind
  before_script:
    - apk add --no-cache nodejs npm
    - npm install -g @devcontainers/cli
  script:
    - devcontainer build --workspace-folder .
    - devcontainer exec --workspace-folder . langstar --version
    - devcontainer exec --workspace-folder . langstar assistant list
  variables:
    LANGSMITH_API_KEY: ${LANGSMITH_API_KEY}
```

### CircleCI

```yaml
version: 2.1

jobs:
  test:
    docker:
      - image: cimg/node:20.0
    steps:
      - checkout
      - setup_remote_docker
      - run:
          name: Install DevContainer CLI
          command: npm install -g @devcontainers/cli
      - run:
          name: Build DevContainer
          command: devcontainer build --workspace-folder .
      - run:
          name: Test Langstar
          command: |
            devcontainer exec --workspace-folder . langstar --version
            devcontainer exec --workspace-folder . langstar config
```

## Advanced Scenarios

### Custom Dockerfile with Feature

Use custom base image with feature:

```dockerfile
# Dockerfile
FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Install additional packages
RUN apt-get update && apt-get install -y \
    curl \
    git \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Set up custom user
RUN useradd -m -s /bin/bash developer
USER developer
```

```jsonc
// devcontainer.json
{
    "build": {
        "dockerfile": "Dockerfile"
    },
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "remoteUser": "developer"
}
```

### Multi-Stage Development

Different configs for different development stages:

```jsonc
// .devcontainer/devcontainer-develop.json
{
    "name": "Development",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}

// .devcontainer/devcontainer-staging.json
{
    "name": "Staging",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    },
    "remoteEnv": {
        "LANGSMITH_WORKSPACE_ID": "${localEnv:LANGSMITH_STAGING_WORKSPACE_ID}"
    }
}

// .devcontainer/devcontainer-production.json
{
    "name": "Production",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    },
    "remoteEnv": {
        "LANGSMITH_WORKSPACE_ID": "${localEnv:LANGSMITH_PROD_WORKSPACE_ID}"
    }
}
```

### With Volume Mounts

Persist configuration across rebuilds:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "mounts": [
        "source=${localEnv:HOME}/.langstar,target=/home/vscode/.langstar,type=bind,consistency=cached"
    ]
}
```

This mounts your local `~/.langstar` directory into the container, preserving configuration across rebuilds.

## Troubleshooting Examples

### Debug Installation

Add verbose logging:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    },
    "postCreateCommand": "set -x && langstar --version && langstar config && set +x"
}
```

### Test Network Access

Verify network connectivity:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "postCreateCommand": "curl -I https://github.com && langstar --version"
}
```

### Fallback Installation

If feature fails, fall back to manual install:

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    },
    "postCreateCommand": "command -v langstar || curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash"
}
```

## Best Practices

### 1. Pin Versions for Team Projects

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"  // Not "latest"
        }
    }
}
```

### 2. Use Environment Variables for Credentials

```jsonc
{
    "remoteEnv": {
        "LANGSMITH_API_KEY": "${localEnv:LANGSMITH_API_KEY}"  // Not hardcoded
    }
}
```

### 3. Document Setup Steps

```jsonc
{
    "postCreateCommand": "cat .devcontainer/README.md"
}
```

### 4. Verify Installation

```jsonc
{
    "postCreateCommand": "langstar --version && echo 'Langstar installed successfully!'"
}
```

### 5. Use Lifecycle Hooks

```jsonc
{
    "postCreateCommand": "cargo build",
    "postStartCommand": "git fetch",
    "postAttachCommand": "langstar config"
}
```

## Additional Resources

- [DevContainer Feature Documentation](../devcontainer-feature.md)
- [Langstar Configuration Guide](../configuration.md)
- [Troubleshooting Guide](../troubleshooting.md)
- [DevContainers Specification](https://containers.dev/implementors/json_reference/)
