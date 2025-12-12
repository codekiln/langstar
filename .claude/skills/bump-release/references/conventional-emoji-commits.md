# Conventional Emoji Commits Reference

This document provides a quick reference for Conventional Emoji Commits format and how they map to semantic versioning.

## Commit Message Format

```
<emoji> <type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Semantic Versioning Rules

| Commit Type            | Example                                 | Version Bump              | Description                                 |
| ---------------------- | --------------------------------------- | ------------------------- | ------------------------------------------- |
| **BREAKING CHANGE**    | `🚨 BREAKING CHANGE: remove legacy API` | **MAJOR** (1.0.0 → 2.0.0) | API-breaking modifications                  |
| **Breaking footer**    | Commit body contains `BREAKING CHANGE:` | **MAJOR** (1.0.0 → 2.0.0) | Breaking change in footer                   |
| **Breaking indicator** | `feat!:` or `fix!:` with `!` before `:` | **MAJOR** (1.0.0 → 2.0.0) | Breaking change indicator                   |
| **✨ feat**            | `✨ feat: add user authentication`      | **MINOR** (1.0.0 → 1.1.0) | New feature                                 |
| **🩹 fix**             | `🩹 fix: resolve memory leak`           | **PATCH** (1.0.0 → 1.0.1) | Bug fix                                     |
| **⚡️ perf**            | `⚡️ perf: optimize query performance`   | **PATCH** (1.0.0 → 1.0.1) | Performance improvement                     |
| **🔄 revert**          | `🔄 revert: revert previous commit`     | **PATCH** (1.0.0 → 1.0.1) | Revert previous changes                     |
| **📚 docs**            | `📚 docs: update README`                | **NONE**                  | Documentation only                          |
| **🎨 style**           | `🎨 style: fix formatting`              | **NONE**                  | Code formatting only                        |
| **♻️ refactor**         | `♻️ refactor: simplify function`         | **NONE**                  | Code refactoring without functional changes |
| **🧪 test**            | `🧪 test: add unit tests`               | **NONE**                  | Adding or modifying tests                   |
| **🔧 build**           | `🔧 build: update dependencies`         | **NONE**                  | Build system or dependency changes          |
| **🤖 ci**              | `🤖 ci: update GitHub Actions`          | **NONE**                  | CI/CD configuration changes                 |
| **📦 chore**           | `📦 chore: update gitignore`            | **NONE**                  | Maintenance tasks                           |

## Commit Type Details

### Breaking Changes (MAJOR bump)

Breaking changes **always** trigger a MAJOR version bump, regardless of the commit type. There are three ways to indicate a breaking change:

1. **Breaking change emoji in subject:**
   ```
   🚨 BREAKING CHANGE: remove deprecated API
   ```

2. **Breaking change in footer:**
   ```
   ✨ feat: add new authentication system

   BREAKING CHANGE: Old auth endpoints have been removed.
   Use the new /api/v2/auth endpoints instead.
   ```

3. **Breaking indicator (!) in subject:**
   ```
   feat!: redesign user API
   ```

### Features (MINOR bump)

Features add new functionality without breaking existing APIs:

```
✨ feat: add email verification
✨ feat(auth): add OAuth2 support
✨ feature: implement dark mode
```

### Bug Fixes (PATCH bump)

Bug fixes resolve issues without adding new functionality:

```
🩹 fix: resolve null pointer exception
🩹 fix(ui): correct button alignment
🩹 bugfix: handle empty input correctly
```

### Performance Improvements (PATCH bump)

Performance improvements that don't change functionality:

```
⚡️ perf: optimize database queries
⚡️ perf(api): reduce response time
```

### Non-Releasable Types (No bump)

These commits do not trigger a version bump:

- `📚 docs` - Documentation changes
- `🎨 style` - Formatting, whitespace, missing semicolons
- `♻️ refactor` - Code changes without functional impact
- `🧪 test` - Adding or updating tests
- `🔧 build` - Build system or dependency updates
- `🤖 ci` - CI/CD configuration
- `📦 chore` - Maintenance tasks

## Scopes (Optional)

Scopes provide additional context about which part of the codebase is affected:

```
✨ feat(auth): add JWT token support
🩹 fix(ui): resolve button styling issue
♻️ refactor(database): simplify query builder
```

Common scopes:

- `(auth)` - Authentication/authorization
- `(ui)` - User interface
- `(api)` - API endpoints
- `(database)` - Database changes
- `(cli)` - Command-line interface
- `(sdk)` - SDK changes

## Multiple Commits

When multiple commits are included in a release, the **highest priority bump** is used:

**Priority order:** MAJOR > MINOR > PATCH > NONE

**Example:**

```
📚 docs: update README        (NONE)
🩹 fix: resolve crash         (PATCH)
✨ feat: add new feature      (MINOR)
🚨 BREAKING CHANGE: remove API (MAJOR)
```

**Result:** MAJOR version bump (1.0.0 → 2.0.0)

## Examples

### Valid Commits

```bash
# Feature with scope
✨ feat(api): add rate limiting

# Bug fix
🩹 fix: handle edge case in parser

# Breaking change with explanation
🚨 BREAKING CHANGE: redesign configuration format

The configuration file format has changed from JSON to TOML.
Users must migrate their config.json to config.toml.

Migration guide: docs/migration.md

# Feature with breaking change in footer
✨ feat: implement new caching system

BREAKING CHANGE: Cache keys have changed format.
Clear existing cache before upgrading.

# Performance improvement
⚡️ perf: optimize image processing

# Documentation
📚 docs: add API examples

# Refactoring
♻️ refactor: simplify error handling
```

### Invalid Commits

```bash
# Missing type
✨ add new feature (missing "feat:")

# Missing colon
feat add new feature (missing ":")

# Missing description
✨ feat:

# Invalid emoji/type
🎉 party: celebrate release (not a valid type)
```

## Version Bump Examples

### MAJOR Bump (Breaking Changes)

**Starting version:** 1.2.3

**Commits:**

```
🚨 BREAKING CHANGE: remove deprecated endpoints
✨ feat: add new API v2
🩹 fix: resolve auth issue
```

**Result:** 2.0.0

---

### MINOR Bump (Features)

**Starting version:** 1.2.3

**Commits:**

```
✨ feat: add user profile page
✨ feat: implement search functionality
🩹 fix: resolve timezone bug
📚 docs: update installation guide
```

**Result:** 1.3.0

---

### PATCH Bump (Fixes)

**Starting version:** 1.2.3

**Commits:**

```
🩹 fix: resolve memory leak
⚡️ perf: optimize query
🧪 test: add integration tests
📚 docs: fix typos
```

**Result:** 1.2.4

---

### No Bump (Non-Releasable)

**Starting version:** 1.2.3

**Commits:**

```
📚 docs: update README
🎨 style: fix formatting
♻️ refactor: simplify code
🧪 test: add unit tests
```

**Result:** No release (all commits are non-releasable)

## Tooling

### Git Commit Message Validation

Use `commitlint` to enforce commit message format:

```bash
npm install --save-dev @commitlint/cli @commitlint/config-conventional
```

### Commit Message Helper

Use `commitizen` with `cz-conventional-emoji` for guided commits:

```bash
npm install --save-dev commitizen cz-conventional-emoji
npx git-cz
```

### Changelog Generation

Use `git-cliff` to generate changelogs from commits:

```bash
cargo install git-cliff
git-cliff --tag v1.2.3
```

## References

- [Conventional Emoji Commits](https://conventional-emoji-commits.site/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [git-cliff](https://git-cliff.org/)

## Summary

- **MAJOR** (X.0.0): Breaking changes, indicated by `🚨`, `BREAKING CHANGE:` footer, or `!` before `:`
- **MINOR** (x.Y.0): New features (`feat`, `feature`)
- **PATCH** (x.y.Z): Bug fixes (`fix`, `bugfix`), performance improvements (`perf`), reverts
- **NONE**: Documentation, style, refactoring, tests, build, CI, chores

When multiple commits are present, use the highest priority bump type.
