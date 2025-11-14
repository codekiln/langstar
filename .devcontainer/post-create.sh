#!/bin/bash
set -e

echo "🔧 Running post-create setup..."

# Ensure PATH includes common tool locations
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# Check if mise is available (either from feature or Dockerfile)
if command -v mise &> /dev/null; then
    echo "✓ mise found"

    # Add mise activation to zshrc if not already present
    if ! grep -q "mise activate" ~/.zshrc 2>/dev/null; then
        echo 'eval "$(mise activate zsh)"' >> ~/.zshrc
        echo "✓ Added mise activation to ~/.zshrc"
    fi

    # Trust and install mise tools
    echo "→ Running mise trust..."
    mise trust

    echo "→ Running mise install..."
    mise install
else
    echo "⚠️  mise not found - skipping mise setup"
fi

# Check if uv is available (either from feature or Dockerfile)
if command -v uv &> /dev/null; then
    echo "✓ uv found"

    echo "→ Installing specify-cli with uv..."
    uv tool install specify-cli --from git+https://github.com/github/spec-kit.git
else
    echo "⚠️  uv not found - skipping specify-cli installation"
fi

# Check if cargo is available
if command -v cargo &> /dev/null; then
    echo "✓ cargo found"

    echo "→ Installing cargo-release and git-cliff..."
    cargo install cargo-release git-cliff
else
    echo "⚠️  cargo not found - skipping cargo tools installation"
fi

echo "✅ Post-create setup complete!"
