# Implementation Plan: Automated CI Testing for Langstar DevContainer Feature

**Issue**: #240 - Implement Automated CI Testing
**Research Phase**: #241 (completed)
**Analysis Phase**: #246 (this document)
**Date**: 2025-11-22

## Executive Summary

Based on analysis of production devcontainer repositories (devcontainers/features and devcontainers/templates), this document provides actionable recommendations for implementing automated CI testing for the langstar devcontainer feature.

**Key Finding**: Use the standard `devcontainer features test` command with matrix testing across multiple base images, following the exact patterns used by Microsoft's official devcontainers/features repository.

## Research Sources

1. **High-level research** (#241): `/workspace/reference/research/241-devcontainer-feature-ci-testing/devcontainer-feature-ci-testing-best-practices-2025-11-22.md`
2. **Detailed analysis** (#246):
   - `/workspace/reference/repo/devcontainers/features/notes/README.md`
   - `/workspace/reference/repo/devcontainers/templates/notes/README.md`
3. **Actual workflow files** (cloned repositories):
   - `/workspace/reference/repo/devcontainers/features/code/.github/workflows/`
   - `/workspace/reference/repo/devcontainers/templates/code/.github/workflows/`

## Implementation Phases

### Phase 1: Basic Test Infrastructure (MVP)

**Goal**: Create minimal test infrastructure and verify it works locally.

**Tasks**:
1. Create `test/langstar/` directory structure
2. Create `test/langstar/test.sh` with basic smoke tests
3. Test locally with `devcontainer features test`

**Deliverables**:

**File**: `test/langstar/test.sh`
```bash
#!/bin/bash

set -e

# Optional: Import test library
source dev-container-features-test-lib

# Test mise installation
check "mise version" mise --version

# Test rustup installation
check "rustup version" rustup --version

# Test cargo installation
check "cargo version" cargo --version

# Test rust compiler
check "rustc version" rustc --version

# Test git installation
check "git version" git --version

# Test gh CLI installation
check "gh version" gh --version

# Test mise.toml is configured
check "mise config exists" test -f ~/.config/mise/config.toml

# Test rust toolchain is active
check "rust toolchain active" rustup show active-toolchain

# Report result
reportResults
```

**Testing Command**:
```bash
# From project root
npm install -g @devcontainers/cli
devcontainer features test -f langstar -i ubuntu:noble .
```

**Success Criteria**:
- All checks pass on ubuntu:noble
- Test completes in < 5 minutes

---

### Phase 2: CI Integration

**Goal**: Add GitHub Actions workflow for automated testing on PRs.

**Tasks**:
1. Create `.github/workflows/test-feature-pr.yaml`
2. Add ShellCheck linting workflow
3. Test against 3 base images

**Deliverables**:

**File**: `.github/workflows/test-feature-pr.yaml`
```yaml
name: "PR - Test Langstar Feature"

on:
  pull_request:
    paths:
      - 'src/langstar/**'
      - 'test/langstar/**'
      - '.github/workflows/test-feature-pr.yaml'

jobs:
  test:
    runs-on: ubuntu-latest
    continue-on-error: true
    strategy:
      matrix:
        baseImage:
          - "ubuntu:noble"      # Ubuntu 24.04 LTS
          - "ubuntu:jammy"      # Ubuntu 22.04 LTS
          - "debian:12"         # Debian Bookworm
    steps:
      - uses: actions/checkout@v4

      - name: "Install latest devcontainer CLI"
        run: npm install -g @devcontainers/cli

      - name: "Test langstar feature on ${{ matrix.baseImage }}"
        run: devcontainer features test --skip-scenarios -f langstar -i ${{ matrix.baseImage }} .
```

**File**: `.github/workflows/lint-shell.yaml`
```yaml
name: "Lint Shell Scripts"

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  shellcheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Shell Linter
        uses: azohra/shell-linter@v0.6.0
        with:
          path: "src/**/*.sh"
          severity: "error"
```

**Success Criteria**:
- CI runs on every PR that touches langstar feature
- Tests pass on all 3 base images
- ShellCheck catches shell script issues
- Total CI time < 10 minutes

---

### Phase 3: Comprehensive Testing

**Goal**: Add scenario testing and expand base image matrix.

**Tasks**:
1. Create `test/langstar/scenarios.json`
2. Add scenario-specific test scripts
3. Expand base image matrix to 7 images
4. Add `test-feature-all.yaml` for main branch

**Deliverables**:

**File**: `test/langstar/scenarios.json`
```json
{
  "langstar_default": {
    "image": "ubuntu:noble",
    "features": {
      "langstar": {}
    }
  },
  "langstar_stable_rust": {
    "image": "ubuntu:jammy",
    "features": {
      "langstar": {
        "rustChannel": "stable"
      }
    }
  },
  "langstar_on_debian": {
    "image": "debian:12",
    "features": {
      "langstar": {}
    }
  },
  "langstar_with_specific_rust_version": {
    "image": "ubuntu:noble",
    "features": {
      "langstar": {
        "rustVersion": "1.75.0"
      }
    }
  }
}
```

**File**: `.github/workflows/test-feature-all.yaml`
```yaml
name: "CI - Test Langstar Feature (All Images)"

on:
  push:
    branches:
      - main
  workflow_dispatch:

jobs:
  test:
    runs-on: ubuntu-latest
    continue-on-error: true
    strategy:
      matrix:
        baseImage:
          - "ubuntu:focal"       # Ubuntu 20.04
          - "ubuntu:jammy"       # Ubuntu 22.04
          - "ubuntu:noble"       # Ubuntu 24.04
          - "debian:11"          # Debian Bullseye
          - "debian:12"          # Debian Bookworm
          - "mcr.microsoft.com/devcontainers/base:ubuntu"
          - "mcr.microsoft.com/devcontainers/base:debian"
    steps:
      - uses: actions/checkout@v4

      - name: "Install latest devcontainer CLI"
        run: npm install -g @devcontainers/cli

      - name: "Test langstar feature on ${{ matrix.baseImage }}"
        run: devcontainer features test --skip-scenarios -f langstar -i ${{ matrix.baseImage }} .

  test-scenarios:
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4

      - name: "Install latest devcontainer CLI"
        run: npm install -g @devcontainers/cli

      - name: "Test langstar scenarios"
        run: devcontainer features test -f langstar --skip-autogenerated .
```

**Success Criteria**:
- Tests pass on 7+ base images
- Scenario testing validates different configurations
- Main branch has comprehensive coverage

---

## Detailed Technical Specifications

### Test Library Usage

The `dev-container-features-test-lib` provides:
- `check "<description>" <command>` - Run command and verify exit code 0
- `reportResults` - Output pass/fail summary

### Base Image Selection Rationale

| Image | Why Test |
|-------|----------|
| ubuntu:noble | Ubuntu 24.04 LTS (latest) |
| ubuntu:jammy | Ubuntu 22.04 LTS (widely used) |
| ubuntu:focal | Ubuntu 20.04 LTS (legacy support) |
| debian:12 | Debian Bookworm (stable) |
| debian:11 | Debian Bullseye (oldstable) |
| mcr.microsoft.com/devcontainers/base:ubuntu | Official devcontainer base |
| mcr.microsoft.com/devcontainers/base:debian | Official devcontainer base |

### CI Performance Optimization

**PR Workflow** (fast):
- Test only changed features (3 images × 1 feature = 3 jobs)
- Run in parallel
- Target: < 10 minutes total

**Main Branch Workflow** (comprehensive):
- Test all scenarios (7 images + scenarios)
- Can take longer (15-20 minutes)
- Ensures nothing breaks in integration

### Testing Best Practices (from upstream)

✅ **DO**:
1. Use `devcontainer features test` (not manual docker commands)
2. Test across multiple OS distributions
3. Use `continue-on-error: true` to see all failures
4. Keep test.sh simple (version checks, smoke tests)
5. Use scenarios.json for configuration variations
6. Run ShellCheck on all shell scripts
7. Use matrix strategy for parallel testing

❌ **DON'T**:
1. Manually build containers (CLI does it)
2. Test in VS Code (headless only)
3. Create complex test infrastructure
4. Skip linting
5. Test everything on every PR (use path filters)
6. Block on single test failure (use continue-on-error)

## Implementation Checklist

### Phase 1 (MVP) - 2-3 hours
- [ ] Create `test/langstar/` directory
- [ ] Write `test/langstar/test.sh`
- [ ] Make test.sh executable (`chmod +x`)
- [ ] Install devcontainer CLI locally
- [ ] Run test locally: `devcontainer features test -f langstar -i ubuntu:noble .`
- [ ] Verify all checks pass
- [ ] Document test results

### Phase 2 (CI Integration) - 2-3 hours
- [ ] Create `.github/workflows/test-feature-pr.yaml`
- [ ] Create `.github/workflows/lint-shell.yaml`
- [ ] Test workflow on a PR
- [ ] Verify CI passes on all base images
- [ ] Verify ShellCheck catches issues
- [ ] Update README with CI badge

### Phase 3 (Comprehensive) - 3-4 hours
- [ ] Create `test/langstar/scenarios.json`
- [ ] Add scenario-specific test scripts (if needed)
- [ ] Create `.github/workflows/test-feature-all.yaml`
- [ ] Expand base image matrix to 7 images
- [ ] Test scenarios locally
- [ ] Verify main branch CI passes
- [ ] Document testing strategy

## Risks and Mitigations

### Risk 1: devcontainer CLI not available in CI

**Mitigation**: Install via npm in workflow:
```yaml
- name: "Install latest devcontainer CLI"
  run: npm install -g @devcontainers/cli
```

### Risk 2: Tests take too long

**Mitigation**:
- Use path filters to only test on changes
- Run matrix jobs in parallel
- Keep test.sh simple (no complex builds)

### Risk 3: Tests fail on some base images

**Mitigation**:
- Use `continue-on-error: true` to see all results
- Add image-specific exclusions if needed (like upstream does)
- Document known issues

### Risk 4: Docker-in-Docker issues in CI

**Mitigation**:
- Use standard GitHub runners (they support Docker)
- Follow exact pattern from devcontainers/features
- No special runner needed for basic tests

## Success Metrics

### Phase 1
- Test runs locally without errors
- All checks pass on ubuntu:noble
- Test completes in < 5 minutes

### Phase 2
- CI runs automatically on PRs
- Tests pass on 3 base images
- ShellCheck catches script issues
- PR feedback within 10 minutes

### Phase 3
- Comprehensive testing on main branch
- 7+ base images tested
- Scenarios validate different configurations
- Zero manual testing required for releases

## References

### Documentation
- Dev Container CLI: https://github.com/devcontainers/cli
- Feature test docs: https://containers.dev/implementors/features/#test
- Test library: https://github.com/devcontainers/cli/blob/main/scripts/test-lib.sh

### Example Workflows
- devcontainers/features: `/workspace/reference/repo/devcontainers/features/code/.github/workflows/test-pr.yaml`
- Rust feature test: `/workspace/reference/repo/devcontainers/features/code/test/rust/`

### Research Documents
- Best practices: `/workspace/reference/research/241-devcontainer-feature-ci-testing/`
- Detailed analysis: `/workspace/reference/repo/devcontainers/features/notes/README.md`

## Next Steps

1. **Immediate**: Implement Phase 1 (MVP) - Create test.sh and run locally
2. **Short-term**: Implement Phase 2 (CI) - Add GitHub Actions workflows
3. **Medium-term**: Implement Phase 3 (Comprehensive) - Add scenarios and expand coverage
4. **Ongoing**: Monitor CI performance and adjust as needed

## Appendix: Commands Reference

### Local Testing
```bash
# Install CLI
npm install -g @devcontainers/cli

# Test feature on specific image
devcontainer features test -f langstar -i ubuntu:noble .

# Test with scenarios
devcontainer features test -f langstar .

# Test specific scenario
devcontainer features test -f langstar --skip-autogenerated .
```

### Debugging
```bash
# See verbose output
devcontainer features test -f langstar -i ubuntu:noble . --log-level debug

# Test in interactive container
devcontainer features test -f langstar -i ubuntu:noble . --preserve-test-containers
```

### CI Commands
```bash
# What CI runs (PR)
devcontainer features test --skip-scenarios -f langstar -i ubuntu:noble .

# What CI runs (scenarios)
devcontainer features test -f langstar --skip-autogenerated .
```
