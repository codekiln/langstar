# Installer Script Testing Checklist

This document provides a comprehensive testing checklist for the langstar installer script (`scripts/install.sh`).

## Overview

Before merging the installer script, it must be tested across all supported platforms and scenarios to ensure reliability.

## Prerequisites

- Access to test environments (Linux, macOS Intel, macOS ARM)
- GitHub release with published binaries (at least one version)
- sudo access on test systems (optional but recommended for full testing)

---

## Platform Testing

### Linux (Debian/Ubuntu) - x86_64

**Environment:**
- [ ] Ubuntu 22.04 LTS
- [ ] Ubuntu 20.04 LTS
- [ ] Debian 12 (Bookworm)
- [ ] Debian 11 (Bullseye)

**Test cases:**
- [ ] Fresh installation (no existing langstar)
- [ ] Update existing installation (older version → latest)
- [ ] Re-install same version (idempotency check)
- [ ] Install with sudo access (system-wide)
- [ ] Install without sudo access (user-local)
- [ ] Install specific version (`--version 0.2.0`)
- [ ] Install to custom prefix (`--prefix ~/bin`)
- [ ] Verify SHA256 checksum passes
- [ ] Verify binary works after installation
- [ ] Check PATH warnings (for `~/.local/bin`)

**Commands:**
```bash
# Fresh install
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
langstar --version

# Update
./install.sh
langstar --version

# Specific version
./install.sh --version 0.2.0
langstar --version

# Custom prefix
./install.sh --prefix ~/test-bin
~/test-bin/langstar --version

# Without sudo (as non-root user without sudo)
./install.sh
~/.local/bin/langstar --version
```

---

### macOS (Intel) - x86_64

**Environment:**
- [ ] macOS Ventura (13.x)
- [ ] macOS Monterey (12.x)
- [ ] macOS Big Sur (11.x)

**Test cases:**
- [ ] Fresh installation
- [ ] Update existing installation
- [ ] Re-install same version
- [ ] Install with admin rights (system-wide)
- [ ] Install without admin rights (user-local)
- [ ] Install specific version
- [ ] Install to custom prefix
- [ ] Verify checksum passes
- [ ] Verify binary works
- [ ] Check PATH warnings

**Commands:**
```bash
# Same as Linux testing commands above
```

---

### macOS (Apple Silicon) - aarch64

**Environment:**
- [ ] macOS Sonoma (14.x) - M1/M2/M3
- [ ] macOS Ventura (13.x) - M1/M2/M3
- [ ] macOS Monterey (12.x) - M1/M2

**Test cases:**
- [ ] Fresh installation (native ARM64 binary)
- [ ] Update existing installation
- [ ] Re-install same version
- [ ] Install with admin rights
- [ ] Install without admin rights
- [ ] Install specific version
- [ ] Install to custom prefix
- [ ] Verify checksum passes
- [ ] Verify binary works (native, not Rosetta)
- [ ] Check `file` output shows ARM64
- [ ] Check PATH warnings

**Commands:**
```bash
# Same as Linux testing commands above

# Verify ARM64 (not x86_64 via Rosetta)
file $(which langstar)
# Should show: Mach-O 64-bit executable arm64
```

---

## Functional Testing

### Download & Verification

- [ ] **Valid checksum:** Installation succeeds with matching checksum
- [ ] **Invalid checksum:** Installation fails with corrupted archive
  ```bash
  # Test by tampering with downloaded file
  ```
- [ ] **Missing checksum file:** Installation continues with warning
- [ ] **Network failure:** Installation fails gracefully with clear error

### Installation Paths

- [ ] **System-wide (`/usr/local/bin`):**
  - With sudo: succeeds
  - Without sudo: fails gracefully, suggests user-local
- [ ] **User-local (`~/.local/bin`):**
  - Creates directory if missing
  - Sets correct permissions (755)
  - Adds to PATH or warns about PATH
- [ ] **Custom prefix:**
  - Respects `--prefix` flag
  - Creates directory if missing
  - Installs to specified location
  - Warns about PATH if needed
- [ ] **Environment variable (`LANGSTAR_INSTALL_DIR`):**
  - Respects environment variable
  - Overrides default but not `--prefix`

### Version Selection

- [ ] **Latest version (default):**
  - Queries GitHub API successfully
  - Downloads latest release
  - Shows correct version after install
- [ ] **Specific version (`--version`):**
  - Downloads requested version
  - Fails if version doesn't exist
  - Shows correct version after install
- [ ] **Invalid version:**
  - Clear error message
  - Suggests checking GitHub releases

### Update & Idempotency

- [ ] **Update from older version:**
  - Detects existing installation
  - Reports version change (X.Y.Z → A.B.C)
  - Successfully replaces binary
  - New version runs correctly
- [ ] **Re-install same version:**
  - Detects existing version
  - Reports "already installed"
  - Re-installs cleanly
  - Binary still works
- [ ] **Downgrade to older version:**
  - Downloads and installs older version
  - Binary works correctly
  - Reports version change

### Error Handling

- [ ] **Missing dependencies:**
  - curl/wget: clear error, suggests installation
  - tar: clear error, suggests installation
  - shasum/sha256sum: warning, continues without verification
- [ ] **Invalid platform:**
  - Unsupported OS: clear error message
  - Unsupported architecture: clear error message
  - Suggests building from source or opening issue
- [ ] **Permission denied:**
  - System directory without sudo: suggests user-local
  - User directory without write: clear error
  - Installation fails gracefully
- [ ] **Download failures:**
  - 404 (version not found): clear error
  - Network timeout: clear error
  - GitHub rate limit: clear error
- [ ] **Extraction failures:**
  - Corrupted archive: clear error
  - Wrong archive format: clear error
  - Cleanup of temp files

### Help & Documentation

- [ ] **`--help` flag:**
  - Shows usage information
  - Lists all options
  - Provides examples
  - Exits with code 0
- [ ] **Invalid flag:**
  - Shows error
  - Suggests `--help`
  - Exits with code 1

---

## Integration Testing

### CI/CD Integration (Future)

- [ ] Runs in GitHub Actions workflow
- [ ] Tests installation on CI runners
- [ ] Verifies installed binary works

### Real-World Scenarios

- [ ] **New user setup:**
  - User has never used langstar
  - Follows quick install from README
  - Binary works immediately
  - Can run `langstar --help`
- [ ] **Developer setup:**
  - Developer clones repo
  - Runs installer to test
  - Can build from source afterward
- [ ] **System administrator:**
  - Installs system-wide for all users
  - Users can access without sudo
  - Binary in standard PATH location

---

## Regression Testing

Before each release:

- [ ] Test installer with new release artifacts
- [ ] Verify checksums match
- [ ] Test on all supported platforms
- [ ] Update this checklist if needed

---

## Manual Testing Template

Use this template for manual testing:

```bash
#!/bin/bash
# Manual testing script for installer

set -e

echo "=== Langstar Installer Testing ==="
echo ""

# 1. Test fresh install
echo "1. Testing fresh install..."
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
langstar --version
echo "✓ Fresh install successful"
echo ""

# 2. Test re-install (idempotency)
echo "2. Testing re-install (idempotency)..."
curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh
chmod +x install.sh
./install.sh
langstar --version
echo "✓ Re-install successful"
echo ""

# 3. Test specific version
echo "3. Testing specific version install..."
./install.sh --version 0.2.0
langstar --version | grep "0.2.0"
echo "✓ Specific version install successful"
echo ""

# 4. Test custom prefix
echo "4. Testing custom prefix install..."
mkdir -p ~/test-langstar-bin
./install.sh --prefix ~/test-langstar-bin
~/test-langstar-bin/langstar --version
rm -rf ~/test-langstar-bin
echo "✓ Custom prefix install successful"
echo ""

# 5. Test help
echo "5. Testing --help flag..."
./install.sh --help
echo "✓ Help flag successful"
echo ""

echo "=== All tests passed ==="
```

---

## Automated Testing (Future)

Consider implementing automated tests:

```bash
# scripts/test-installer-automated.sh
#!/bin/bash
# Automated testing framework

# Test in Docker containers for different platforms
docker run --rm -v "$(pwd):/workspace" ubuntu:22.04 /workspace/scripts/install.sh
docker run --rm -v "$(pwd):/workspace" debian:12 /workspace/scripts/install.sh

# Test with different configurations
LANGSTAR_INSTALL_DIR=/tmp/test-install ./install.sh
./install.sh --prefix /tmp/test-prefix

# Verify binaries work
/tmp/test-install/langstar --version
/tmp/test-prefix/langstar --version

# Cleanup
rm -rf /tmp/test-install /tmp/test-prefix
```

---

## Sign-Off Checklist

Before marking the installer as production-ready:

- [ ] All platform tests pass
- [ ] All functional tests pass
- [ ] All error cases handled gracefully
- [ ] Documentation is complete and accurate
- [ ] README updated with installation instructions
- [ ] At least two team members have tested on different platforms
- [ ] CI/CD integration planned (if applicable)
- [ ] Release notes mention the installer script

---

## Reporting Issues

If tests fail, report with:

1. **Platform details:**
   - OS and version (`uname -a`, `sw_vers` on macOS)
   - Architecture (`uname -m`)
   - Shell (`echo $SHELL`)

2. **Test case:**
   - Which test case failed
   - Command executed
   - Expected vs actual behavior

3. **Error output:**
   - Full error messages
   - Installer script output
   - Any relevant logs

4. **Environment:**
   - Sudo access available? (yes/no)
   - Network restrictions? (firewall, proxy)
   - Other relevant details

---

## Next Steps

After testing:

1. Mark completed test cases with ✅
2. Document any issues found
3. Fix issues and re-test
4. Update documentation if needed
5. Get approval from maintainers
6. Merge PR and celebrate! 🎉
