# Issue #718 Manual Testing Checklist

**PR**: #721 - Multi-environment devcontainer setup
**Issue**: #718 - Create multi-environment devcontainer configs for VS Code, Codespaces, and JetBrains
**Branch**: `i718-multi-env-devcontainer`

## Overview

This PR adds three separate devcontainer configurations optimized for different development environments:
- **Default** (`.devcontainer/`) - VS Code local development
- **Codespaces** (`.devcontainer/codespaces/`) - GitHub Codespaces
- **JetBrains** (`.devcontainer/jetbrains/`) - RustRover, IntelliJ IDEA

## Pre-Testing Setup

- [ ] Ensure Docker Desktop is running (for local tests)
- [ ] Have GitHub Codespaces access configured (for Codespaces tests)
- [ ] Have JetBrains Gateway installed (for JetBrains tests)
- [ ] Have test credentials ready (API keys, PATs, etc.)

---

## Test Suite 1: VS Code Local Development (Default Config)

### 1.1 Initial Setup from Scratch

- [ ] **Clone fresh repository**
  ```bash
  git clone https://github.com/codekiln/langstar.git test-langstar-vscode
  cd test-langstar-vscode
  git checkout i718-multi-env-devcontainer
  ```

- [ ] **Create `.env` file**
  ```bash
  cd .devcontainer
  cp .env.default .env
  # Edit .env with real credentials
  ```

- [ ] **Verify `.env` has actual values** (not placeholders)
  - `GITHUB_PAT` starts with `ghp_`
  - `LANGSMITH_API_KEY` starts with `lsv2_`
  - `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are populated

### 1.2 Container Build and Startup

- [ ] **Open in VS Code**
  - Open the `test-langstar-vscode` folder in VS Code

- [ ] **Reopen in Container**
  - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
  - Select "Dev Containers: Reopen in Container"
  - Wait for container to build (first time takes 3-5 minutes)

- [ ] **Verify build completes without errors**
  - Check VS Code Output panel for any build errors
  - Confirm container starts successfully

### 1.3 Environment Variable Verification

- [ ] **Check environment variables are loaded**
  ```bash
  # In VS Code terminal inside container
  echo "GITHUB_PAT: ${GITHUB_PAT:0:10}..."
  echo "LANGSMITH_API_KEY: ${LANGSMITH_API_KEY:0:10}..."
  echo "AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID:0:10}..."
  printenv | grep -E 'GITHUB|LANGSMITH|AWS' | wc -l  # Should show multiple vars
  ```

- [ ] **Verify all required variables are set**
  - `GITHUB_PAT` is defined
  - `GITHUB_USER` is defined
  - `LANGSMITH_API_KEY` is defined
  - `AWS_ACCESS_KEY_ID` is defined
  - `AWS_SECRET_ACCESS_KEY` is defined

### 1.4 Git Authentication

- [ ] **Test git operations**
  ```bash
  # Should show your GitHub username and email
  git config user.name
  git config user.email

  # Test authenticated fetch (should not prompt for credentials)
  git fetch origin
  ```

- [ ] **Verify git credential helper is configured**
  ```bash
  git config credential.helper  # Should show 'store'
  ```

### 1.5 Development Tools

- [ ] **Verify Rust toolchain**
  ```bash
  rustc --version
  cargo --version
  cargo nextest --version
  ```

- [ ] **Verify GitHub CLI**
  ```bash
  gh --version
  gh auth status  # Should show authenticated
  ```

- [ ] **Verify langstar CLI is installed**
  ```bash
  langstar --version  # Should show v2.1.2 or later
  ```

- [ ] **Run cargo checks**
  ```bash
  cargo fmt --check
  cargo check --workspace --all-features
  cargo clippy --workspace --all-features -- -D warnings
  cargo nextest run --profile ci --all-features --workspace
  ```

### 1.6 File System and Volumes

- [ ] **Verify workspace mount**
  ```bash
  ls -la /workspace  # Should show repository files
  touch /workspace/test-file.txt
  # Check that test-file.txt appears in VS Code file explorer
  rm /workspace/test-file.txt
  ```

- [ ] **Verify persistent volumes**
  ```bash
  # Bash history should persist across container restarts
  echo "test command" >> ~/.bash_history

  # Claude Code config should persist
  ls -la ~/.claude
  ```

- [ ] **Test bidirectional file sync**
  - Edit a file in VS Code
  - Verify changes appear immediately in terminal: `cat <file>`
  - Edit a file in terminal: `echo "test" >> README.md`
  - Verify changes appear immediately in VS Code

### 1.7 Extensions and Settings

- [ ] **Verify VS Code extensions are installed**
  - `anthropic.claude-code` (Claude Code extension)
  - `ms-azuretools.vscode-docker` (Docker extension)
  - `ms-python.python` (Python extension)

- [ ] **Verify VS Code settings**
  - Auto-save is enabled: Check Settings > Files: Auto Save
  - Format on save is enabled: Check Settings > Editor: Format On Save
  - Default terminal is zsh: Open terminal and check shell

### 1.8 Rebuild Test

- [ ] **Test rebuild workflow**
  - Press `Cmd+Shift+P` → "Dev Containers: Rebuild Container"
  - Verify rebuild completes successfully
  - Verify environment variables still work after rebuild
  - Verify git authentication still works after rebuild

---

## Test Suite 2: GitHub Codespaces

### 2.1 Codespaces Secrets Configuration

- [ ] **Configure Codespaces secrets**
  - Go to repository Settings → Secrets and variables → Codespaces
  - Verify the following secrets are set:
    - `GH_PAT` (GitHub PAT)
    - `GH_USER` (GitHub username)
    - `GH_PROJECT_PAT` (GitHub project PAT)
    - `AWS_ACCESS_KEY_ID`
    - `AWS_SECRET_ACCESS_KEY`
    - `LANGSMITH_API_KEY`

### 2.2 Create Codespace

- [ ] **Create a new Codespace**
  - Go to repository on GitHub
  - Click green "Code" button → Codespaces tab
  - Click "Create codespace on i718-multi-env-devcontainer"
  - When prompted for devcontainer configuration, select: `.devcontainer/codespaces/devcontainer.json`

- [ ] **Wait for Codespace to build**
  - First build takes 5-10 minutes
  - Verify build completes without errors

### 2.3 Environment Verification in Codespaces

- [ ] **Verify NO `.env` file exists**
  ```bash
  ls -la /workspace/.devcontainer/.env  # Should NOT exist
  ```

- [ ] **Verify environment variables from secrets**
  ```bash
  echo "GH_PAT: ${GH_PAT:0:10}..."
  echo "LANGSMITH_API_KEY: ${LANGSMITH_API_KEY:0:10}..."
  printenv | grep -E 'GH_|LANGSMITH|AWS' | wc -l  # Should show multiple vars
  ```

- [ ] **Check all required variables**
  - `GH_PAT` is defined
  - `GH_USER` is defined
  - `LANGSMITH_API_KEY` is defined
  - `AWS_ACCESS_KEY_ID` is defined
  - `AWS_SECRET_ACCESS_KEY` is defined

### 2.4 Git and Development Tools in Codespaces

- [ ] **Test git operations**
  ```bash
  git config user.name
  git config user.email
  git fetch origin  # Should work without prompting
  ```

- [ ] **Verify development tools**
  ```bash
  rustc --version
  gh --version
  langstar --version
  ```

- [ ] **Run cargo checks**
  ```bash
  cargo fmt --check
  cargo check --workspace --all-features
  ```

### 2.5 Codespaces-Specific Features

- [ ] **Test port forwarding** (if applicable)
  - Run a local server
  - Verify ports forward automatically in Codespaces

- [ ] **Test VS Code extensions in Codespaces**
  - Verify Claude Code extension is active
  - Verify Docker extension is active

### 2.6 Rebuild in Codespaces

- [ ] **Test rebuild**
  - Codespaces → Rebuild Container
  - Verify rebuild succeeds
  - Verify secrets still work after rebuild

---

## Test Suite 3: JetBrains Gateway (RustRover/IntelliJ)

### 3.1 Prerequisites Setup

- [ ] **Prepare local checkout with `.env`**
  ```bash
  git clone https://github.com/codekiln/langstar.git test-langstar-jetbrains
  cd test-langstar-jetbrains
  git checkout i718-multi-env-devcontainer
  cd .devcontainer
  cp .env.default .env
  # Edit .env with real credentials
  ```

- [ ] **Verify SSH agent is running**
  ```bash
  ssh-add -l  # Should list keys or show "The agent has no identities"
  ```

### 3.2 JetBrains Gateway Setup

- [ ] **Open JetBrains Gateway**
  - Select "Remote Development" → "Dev Containers"

- [ ] **Clone repository through Gateway**
  - Click "Clone Repository"
  - Enter repository URL: `https://github.com/codekiln/langstar.git`
  - When prompted for devcontainer path, select: `.devcontainer/jetbrains/devcontainer.json`
  - Wait for Gateway to clone and build (5-10 minutes first time)

### 3.3 Verify Hybrid Mount Architecture

- [ ] **Check workspace location**
  ```bash
  # Should be in named volume, not bind mount
  df -h /workspace | grep -E 'workspace|volume'
  ```

- [ ] **Verify `.devcontainer` bind mount**
  ```bash
  ls -la /workspace/.devcontainer/.env  # Should exist (bind-mounted from host)
  cat /workspace/.devcontainer/.env | head -n 3  # Should show actual values
  ```

- [ ] **Confirm environment variables loaded from bind-mounted `.env`**
  ```bash
  echo "GITHUB_PAT: ${GITHUB_PAT:0:10}..."
  echo "LANGSMITH_API_KEY: ${LANGSMITH_API_KEY:0:10}..."
  ```

### 3.4 File Watching and inotify

- [ ] **Test file watching**
  - Edit a file in JetBrains IDE
  - Verify changes appear immediately in terminal: `cat <file>`
  - Check for NO warning: "External file changes sync might be slow"

- [ ] **Verify native inotify support**
  ```bash
  # Should NOT show gRPC FUSE warnings
  dmesg | grep -i fuse  # Should be minimal or empty
  ```

- [ ] **Test rapid file changes**
  - Use Find & Replace across multiple files
  - Verify IDE remains responsive
  - Check for no OOM crashes or performance issues

### 3.5 Development Workflow in JetBrains

- [ ] **Run Rust build**
  - Open Cargo.toml
  - Click "Build" or use IDE build tools
  - Verify build succeeds

- [ ] **Run tests in IDE**
  - Right-click on `tests/` directory
  - Select "Run Tests"
  - Verify tests execute successfully

- [ ] **Use integrated terminal**
  - Open terminal in IDE
  - Run: `cargo check --workspace`
  - Verify command completes successfully

### 3.6 Git Integration in JetBrains

- [ ] **Test git operations in IDE**
  - VCS → Git → Fetch
  - Should complete without authentication prompts

- [ ] **Check git configuration**
  - Terminal: `git config user.name`
  - Should show your GitHub username

### 3.7 JetBrains Gateway Reconnection

- [ ] **Test disconnect/reconnect**
  - Disconnect from the remote environment
  - Reconnect to the same container
  - Verify state is preserved (open files, terminal sessions, etc.)
  - Verify environment variables still work

---

## Test Suite 4: Cross-Configuration Validation

### 4.1 Configuration Isolation

- [ ] **Verify configs don't interfere**
  - Default config uses `.devcontainer/docker-compose.yml`
  - Codespaces config uses `.devcontainer/codespaces/docker-compose.yml`
  - JetBrains config uses `.devcontainer/jetbrains/docker-compose.yml`
  - Each should work independently

### 4.2 Documentation Accuracy

- [ ] **Verify README.md matches actual behavior**
  - Re-read `.devcontainer/README.md`
  - Check all commands work as documented
  - Verify all troubleshooting steps are accurate

- [ ] **Check getting-started.md (if exists)**
  - Verify instructions match the new multi-config setup

### 4.3 Template Files

- [ ] **Verify `.env.default` is complete**
  - Compare `.env.default` with actual `.env` used in testing
  - Ensure all required variables are documented

- [ ] **Check `docker-compose.override.yml.template`**
  - Verify template syntax is valid
  - Confirm deprecation notice is clear

---

## Test Suite 5: Edge Cases and Error Scenarios

### 5.1 Missing Credentials

- [ ] **Test VS Code without `.env` file**
  - Remove `.devcontainer/.env`
  - Try to build container
  - Verify clear error message about missing `.env`

- [ ] **Test Codespaces without secrets**
  - Remove one Codespaces secret (e.g., `GH_PAT`)
  - Try to create Codespace
  - Verify container starts but authentication fails gracefully

### 5.2 Invalid Credentials

- [ ] **Test with invalid API keys**
  - Use fake/expired `LANGSMITH_API_KEY`
  - Verify langstar CLI fails gracefully with clear error
  - Verify container still starts

- [ ] **Test with invalid GitHub PAT**
  - Use expired/invalid `GITHUB_PAT`
  - Verify git operations fail with clear error message
  - Verify `setup-github-auth.sh` handles this gracefully

### 5.3 Volume Cleanup

- [ ] **Test volume persistence after container deletion**
  - Note a value in bash history: `echo "test123" >> ~/.bash_history`
  - Delete container: `Dev Containers: Rebuild Container`
  - Verify bash history persists: `history | grep test123`

- [ ] **Test clean slate rebuild**
  - Delete all related volumes: `docker volume prune`
  - Rebuild container
  - Verify fresh environment

---

## Test Suite 6: Regression Testing

### 6.1 Existing Workflows Still Work

- [ ] **Test legacy local setup** (if user had old config)
  - Users with existing `.devcontainer/.env` should still work
  - No breaking changes to existing local development workflow

- [ ] **Test CI/CD integration**
  - Verify GitHub Actions still work (check PR #721 status)
  - Ensure CI doesn't depend on local `.env` files

### 6.2 Feature Parity

- [ ] **Verify all configs have same dev tools**
  - Rust toolchain (rustc, cargo, nextest)
  - GitHub CLI (gh)
  - langstar CLI (v2.1.2+)
  - Python (if needed)

- [ ] **Verify all configs have same features**
  - All devcontainer features are applied correctly
  - No missing features in any configuration

---

## Completion Checklist

- [ ] All VS Code local tests pass (Suite 1)
- [ ] All Codespaces tests pass (Suite 2)
- [ ] All JetBrains tests pass (Suite 3)
- [ ] Cross-configuration tests pass (Suite 4)
- [ ] Edge cases handled correctly (Suite 5)
- [ ] No regressions detected (Suite 6)

## Issues Found

Document any issues discovered during testing:

1. **Issue**: [Description]
   - **Severity**: Critical / High / Medium / Low
   - **Config Affected**: VS Code / Codespaces / JetBrains / All
   - **Steps to Reproduce**: [Steps]
   - **Expected**: [What should happen]
   - **Actual**: [What actually happens]

2. [Add more as needed]

---

## Sign-off

- [ ] I have completed all applicable test suites
- [ ] I have documented any issues found
- [ ] I recommend this PR for merge (or outline blocking issues)

**Tester**: ________________________
**Date**: ________________________
**Notes**: ________________________
