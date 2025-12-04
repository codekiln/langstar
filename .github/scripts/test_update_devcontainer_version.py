#!/usr/bin/env python3
"""
Tests for update_devcontainer_version.py
"""

import tempfile
import pytest
from pathlib import Path
from update_devcontainer_version import update_devcontainer_version


# Sample devcontainer.json content with comments
SAMPLE_DEVCONTAINER = """{
  "name": "langstar",
  // Use Docker Compose for container configuration
  "dockerComposeFile": "docker-compose.yml",
  "service": "langstar-dev",
  "workspaceFolder": "/workspace",

  // Dev Container features (installed by VS Code, not Docker Compose)
  "features": {
    "ghcr.io/devcontainers/features/github-cli:1": {},
    "ghcr.io/codekiln/langstar/langstar:1": {
      "version": "latest"
    }
  },

  "remoteUser": "node"
}
"""


def test_update_version_success():
    """Test successful version update."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(SAMPLE_DEVCONTAINER)
        temp_path = f.name

    try:
        # Update version
        result = update_devcontainer_version(temp_path, "0.13.0")
        assert result is True

        # Read updated content
        updated_content = Path(temp_path).read_text()

        # Verify version was updated
        assert '"version": "0.13.0"' in updated_content

        # Verify comments are preserved
        assert "// Use Docker Compose" in updated_content
        assert "// Dev Container features" in updated_content

        # Verify other parts unchanged
        assert '"name": "langstar"' in updated_content
        assert '"remoteUser": "node"' in updated_content
    finally:
        Path(temp_path).unlink()


def test_update_version_with_different_format():
    """Test version update with different version format."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(SAMPLE_DEVCONTAINER)
        temp_path = f.name

    try:
        # Update to different version formats
        for version in ["1.0.0", "0.0.1", "10.20.30"]:
            result = update_devcontainer_version(temp_path, version)
            assert result is True

            updated_content = Path(temp_path).read_text()
            assert f'"version": "{version}"' in updated_content
    finally:
        Path(temp_path).unlink()


def test_file_not_found():
    """Test error handling when file doesn't exist."""
    with pytest.raises(FileNotFoundError):
        update_devcontainer_version("/nonexistent/file.json", "0.13.0")


def test_pattern_not_found():
    """Test error handling when version pattern is not found."""
    # Create file without the expected pattern
    content_without_pattern = """{
  "name": "test",
  "features": {}
}
"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(content_without_pattern)
        temp_path = f.name

    try:
        with pytest.raises(ValueError, match="Pattern not found"):
            update_devcontainer_version(temp_path, "0.13.0")
    finally:
        Path(temp_path).unlink()


def test_preserves_formatting():
    """Test that formatting and whitespace are preserved."""
    content_with_specific_formatting = """{
  "features": {
    "ghcr.io/codekiln/langstar/langstar:1":    {
      "version":    "latest"
    }
  }
}
"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(content_with_specific_formatting)
        temp_path = f.name

    try:
        result = update_devcontainer_version(temp_path, "0.13.0")
        assert result is True

        updated_content = Path(temp_path).read_text()

        # Verify version was updated
        assert '"version":    "0.13.0"' in updated_content

        # Verify extra spaces around the feature name are preserved
        assert '"ghcr.io/codekiln/langstar/langstar:1":    {' in updated_content
    finally:
        Path(temp_path).unlink()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
