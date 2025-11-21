# Local Feature Testing

This directory contains a test devcontainer configuration for locally testing the Langstar feature.

## Testing Locally

To test the feature locally before publishing:

1. Open this directory (`test-feature/`) in VS Code
2. Run "Dev Containers: Reopen in Container" from the command palette
3. The feature should install automatically during container build
4. Verify installation with: `langstar --version`

## Test Configuration

The `devcontainer.json` references the local feature using a relative path:

```jsonc
{
    "features": {
        "../features/langstar": {
            "version": "latest"
        }
    }
}
```

## Expected Behavior

When the container builds:
1. The base Ubuntu image is pulled
2. The feature's `install.sh` script executes
3. Langstar CLI is downloaded and installed to `/usr/local/bin`
4. The `postCreateCommand` runs to verify installation

## Troubleshooting

If the feature fails to install:

1. Check the container build logs for errors
2. Verify `install.sh` is executable: `ls -l ../.devcontainer/features/langstar/install.sh`
3. Test the install script manually: `bash ../.devcontainer/features/langstar/install.sh`
4. Ensure the feature JSON is valid: `jq . ../.devcontainer/features/langstar/devcontainer-feature.json`

## Manual Testing

You can also test the install script directly:

```bash
cd ../.devcontainer/features/langstar
bash install.sh
langstar --version
```
