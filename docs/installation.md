# Installation Guide

This guide provides detailed instructions for installing langstar on various platforms.

## Table of Contents

- [Quick Install](#quick-install)
- [Supported Platforms](#supported-platforms)
- [Installation Methods](#installation-methods)
  - [Using the Installer Script](#using-the-installer-script)
  - [Manual Installation](#manual-installation)
  - [Building from Source](#building-from-source)
- [Installation Options](#installation-options)
- [Verifying Installation](#verifying-installation)
- [Updating langstar](#updating-langstar)
- [Uninstalling langstar](#uninstalling-langstar)
- [Troubleshooting](#troubleshooting)

---

## Quick Install

For most users, the installer script is the fastest and easiest method:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
```

This will:
1. Detect your platform (Linux or macOS) and architecture (x86_64 or ARM64)
2. Download the appropriate pre-built binary from GitHub releases
3. Verify the SHA256 checksum
4. Install to `/usr/local/bin` (or `~/.local/bin` if you don't have sudo access)

## Supported Platforms

langstar currently supports the following platforms:

| Operating System | Architecture | Binary Type | Notes |
|-----------------|--------------|-------------|-------|
| **Linux** | x86_64 (64-bit) | Static (musl) | No runtime dependencies |
| **macOS** | x86_64 (Intel) | Dynamic | macOS 10.13+ |
| **macOS** | aarch64 (Apple Silicon) | Dynamic | M1/M2/M3 Macs |

### Requirements

**All platforms:**
- `curl` or `wget` (for downloading)
- `tar` (for extracting archives)
- `shasum` or `sha256sum` (for checksum verification)

These tools are pre-installed on most systems.

**Linux:**
- No additional runtime dependencies (static binary)
- Works on Debian, Ubuntu, Fedora, RHEL, Alpine, and other distributions

**macOS:**
- macOS 10.13 (High Sierra) or later
- No additional runtime dependencies

---

## Installation Methods

### Using the Installer Script

The official installer script is the recommended method for most users.

#### Option 1: Direct Install (Recommended)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
```

**What it does:**
- Uses HTTPS with TLS 1.2+ for security
- Downloads and runs the installer script
- Installs the latest version

#### Option 2: Download and Inspect

If you prefer to inspect the script before running:

```bash
# Download the installer
curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh

# Inspect the script (optional but recommended)
less install.sh

# Make it executable
chmod +x install.sh

# Run the installer
./install.sh
```

#### Option 3: With Options

Install with specific options:

```bash
# Install specific version
curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh
chmod +x install.sh
./install.sh --version 0.2.0

# Install to custom directory
./install.sh --prefix ~/bin

# Combine options
./install.sh --version 0.2.0 --prefix ~/.local/bin
```

### Manual Installation

If you prefer not to use the installer script, you can install manually:

#### Step 1: Determine Your Platform

**Linux (x86_64):**
```bash
PLATFORM="x86_64-linux-musl"
```

**macOS (Intel):**
```bash
PLATFORM="x86_64-macos"
```

**macOS (Apple Silicon):**
```bash
PLATFORM="aarch64-macos"
```

#### Step 2: Set Version

```bash
VERSION="0.2.0"  # or check latest at https://github.com/codekiln/langstar/releases
```

#### Step 3: Download and Extract

```bash
# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Download archive
curl -LO "https://github.com/codekiln/langstar/releases/download/v${VERSION}/langstar-${VERSION}-${PLATFORM}.tar.gz"

# Download checksum
curl -LO "https://github.com/codekiln/langstar/releases/download/v${VERSION}/langstar-${VERSION}-${PLATFORM}.tar.gz.sha256"

# Verify checksum
shasum -a 256 -c "langstar-${VERSION}-${PLATFORM}.tar.gz.sha256"

# Extract
tar -xzf "langstar-${VERSION}-${PLATFORM}.tar.gz"
```

#### Step 4: Install

**System-wide (requires sudo):**
```bash
sudo install -m 755 langstar /usr/local/bin/langstar
```

**User-local (no sudo required):**
```bash
mkdir -p ~/.local/bin
install -m 755 langstar ~/.local/bin/langstar

# Add to PATH if not already present
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc for zsh
source ~/.bashrc  # or source ~/.zshrc
```

#### Step 5: Verify

```bash
langstar --version
```

### Building from Source

For developers or if pre-built binaries aren't available for your platform:

#### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.70+)
- Git

#### Steps

```bash
# Clone the repository
git clone https://github.com/codekiln/langstar.git
cd langstar

# Build and install
cargo install --path cli

# Verify
langstar --version
```

**Build options:**

```bash
# Development build (faster compilation, slower runtime)
cargo build --bin langstar

# Release build (optimized)
cargo build --release --bin langstar

# Install specific features
cargo install --path cli --features experimental

# Install to custom directory
cargo install --path cli --root ~/.local
```

---

## Installation Options

### Installer Script Options

The `install.sh` script supports the following options:

| Option | Description | Example |
|--------|-------------|---------|
| `--version VERSION` | Install specific version | `--version 0.2.0` |
| `--prefix DIR` | Install to custom directory | `--prefix ~/bin` |
| `--help` | Show help message | `--help` |

### Installation Directories

The installer uses the following logic to choose the installation directory:

1. **`--prefix` flag** (highest priority) - User-specified directory
2. **`LANGSTAR_INSTALL_DIR` environment variable** - Override default directory
3. **`/usr/local/bin`** - System-wide (if writable or sudo available)
4. **`~/.local/bin`** - User-local fallback (no sudo required)

**Examples:**

```bash
# System-wide (requires sudo)
./install.sh

# User-local (no sudo)
LANGSTAR_INSTALL_DIR=~/.local/bin ./install.sh

# Custom directory
./install.sh --prefix ~/my-tools/bin
```

### PATH Configuration

If langstar is installed to a directory not in your PATH, you'll need to add it:

**For ~/.local/bin:**
```bash
# Bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Fish
fish_add_path ~/.local/bin
```

**For custom directory:**
```bash
echo 'export PATH="/path/to/custom/dir:$PATH"' >> ~/.bashrc  # or ~/.zshrc
source ~/.bashrc  # or source ~/.zshrc
```

---

## Verifying Installation

After installation, verify that langstar is working:

```bash
# Check version
langstar --version

# Show help
langstar --help

# Test configuration (requires LANGSMITH_API_KEY)
export LANGSMITH_API_KEY="your-api-key"
langstar config
```

**Expected output:**
```
langstar 0.2.0
```

---

## Updating langstar

### Using the Installer Script

The installer is idempotent and can be used to update:

```bash
# Update to latest version
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh

# Update to specific version
curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh
chmod +x install.sh
./install.sh --version 0.3.0
```

The installer will:
1. Detect existing installation
2. Compare versions
3. Download and install the new version
4. Verify installation

### Manual Update

```bash
# Check current version
langstar --version

# Download and install new version manually (see Manual Installation section)
```

### Update from Source

```bash
cd langstar
git pull origin main
cargo install --path cli --force
```

---

## Uninstalling langstar

### If Installed via Installer Script

**System-wide installation:**
```bash
sudo rm /usr/local/bin/langstar
```

**User-local installation:**
```bash
rm ~/.local/bin/langstar
```

**Custom installation:**
```bash
rm /path/to/custom/dir/langstar
```

### If Installed via Cargo

```bash
cargo uninstall langstar
```

### Clean Up Configuration (Optional)

To remove configuration files:

```bash
# Remove config directory
rm -rf ~/.langstar

# Remove from shell profile (if added to PATH manually)
# Edit ~/.bashrc or ~/.zshrc and remove the export PATH line
```

---

## Troubleshooting

### Installer Script Issues

#### "curl: command not found"

Install curl:

**Debian/Ubuntu:**
```bash
sudo apt-get update && sudo apt-get install curl
```

**macOS:**
```bash
# curl is pre-installed on macOS
# If missing, install via Homebrew:
brew install curl
```

#### "Failed to download"

Check your internet connection and GitHub accessibility:

```bash
# Test GitHub connectivity
curl -I https://github.com

# Try with wget instead of curl
wget https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh -O install.sh
chmod +x install.sh
./install.sh
```

#### "Checksum verification failed"

This indicates the downloaded file is corrupted or tampered with:

1. Try downloading again (network issue)
2. Check GitHub releases page manually
3. Report the issue if it persists

#### "Permission denied"

**For system-wide installation:**
```bash
# Use sudo
sudo ./install.sh

# Or install to user directory
./install.sh --prefix ~/.local/bin
```

**For user-local installation:**
```bash
# Ensure directory exists and is writable
mkdir -p ~/.local/bin
chmod 755 ~/.local/bin
./install.sh --prefix ~/.local/bin
```

### Command Not Found After Installation

#### Check if binary exists
```bash
# System-wide
ls -l /usr/local/bin/langstar

# User-local
ls -l ~/.local/bin/langstar
```

#### Check PATH
```bash
echo $PATH
```

If the installation directory isn't in PATH, add it (see [PATH Configuration](#path-configuration)).

#### Reload shell
```bash
# Reload shell configuration
source ~/.bashrc  # or source ~/.zshrc for zsh

# Or open a new terminal
```

### Platform Not Supported

If you see "Unsupported operating system" or "Unsupported architecture":

1. Check if your platform is listed in [Supported Platforms](#supported-platforms)
2. Try building from source (see [Building from Source](#building-from-source))
3. Open an issue on GitHub requesting support for your platform

### Build from Source Issues

#### Rust not installed
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Compilation errors
```bash
# Update Rust to latest stable
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

#### Tests failing
```bash
# Run tests to diagnose
cargo test

# Check for required environment variables
export LANGSMITH_API_KEY="your-test-api-key"
cargo test
```

---

## Getting Help

If you encounter issues not covered here:

1. **Check existing issues:** https://github.com/codekiln/langstar/issues
2. **Open a new issue:** https://github.com/codekiln/langstar/issues/new
3. **Provide details:**
   - Operating system and version
   - Architecture (x86_64 or ARM64)
   - Installation method used
   - Full error messages
   - Output of `langstar --version` (if installed)

---

## Next Steps

After installation:

1. **Configure langstar:** See [Configuration Guide](../README.md#configuration)
2. **Get started:** See [Usage Examples](../README.md#usage-examples)
3. **Read documentation:** Check the [README](../README.md) for full documentation

---

Built with ❤️ using Rust
