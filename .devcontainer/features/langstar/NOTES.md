# Langstar CLI Feature

This devcontainer feature installs the [Langstar CLI](https://github.com/codekiln/langstar), a tool for managing LangGraph Cloud deployments via the LangSmith API.

## Usage

Add this feature to your `devcontainer.json`:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {}
    }
}
```

## Options

### `version` (string)

Specify which version of Langstar CLI to install.

- **Default**: `"latest"`
- **Examples**: `"latest"`, `"v0.4.0"`, `"v0.3.0"`

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

- **Langstar CLI binary** installed to `/usr/local/bin/langstar`
- Supports both **x86_64** and **ARM64** architectures

## After Installation

Verify the installation:

```bash
langstar --version
```

## Examples

### Install Latest Version

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

### Install Specific Version

```jsonc
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "v0.4.0"
        }
    }
}
```

### Combined with Other Features

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/rust:1",
    "features": {
        "ghcr.io/devcontainers/features/common-utils:2": {},
        "ghcr.io/devcontainers/features/github-cli:1": {},
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

## Configuration

After installation, you'll need to configure Langstar with your LangSmith API credentials.

### Method 1: Environment Variables (Recommended for Devcontainers)

```bash
export LANGSMITH_API_KEY="your-api-key"
export LANGSMITH_ORGANIZATION_ID="your-org-id"  # Optional
export LANGSMITH_WORKSPACE_ID="your-workspace-id"  # Optional
```

### Method 2: Configuration File

Create or edit `~/.config/langstar/config.toml`:

```toml
[langstar]
langsmith_api_key = "your-api-key"
organization_id = "your-org-id"  # Optional
workspace_id = "your-workspace-id"  # Optional
```

### Verify Configuration

Check your configuration with:

```bash
langstar config
```

## Documentation

For complete documentation, visit:
- [Langstar GitHub Repository](https://github.com/codekiln/langstar)
- [Langstar CLI Documentation](https://github.com/codekiln/langstar/blob/main/README.md)

## Support

If you encounter issues:
1. Check the [GitHub Issues](https://github.com/codekiln/langstar/issues)
2. Review the [installation script](https://github.com/codekiln/langstar/blob/main/scripts/install.sh)
3. Open a new issue with details about your environment

## Architecture Support

This feature currently supports:
- **Linux x86_64** (amd64) ✅
- **macOS ARM64** (Apple Silicon) ✅

**Planned but not yet available:**
- **Linux ARM64** (aarch64) - Planned for future release

The installer automatically detects your architecture and downloads the appropriate binary.

**Note:** There is currently a known issue with release asset uploads (#214) that prevents binary downloads. This is being addressed and will be fixed in an upcoming release.
