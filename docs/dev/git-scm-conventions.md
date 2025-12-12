# Git SCM Conventions

## Conventional Emoji Commits

This project uses Conventional Emoji Commits, a specification that blends gitmoji with Conventional Commits to create more expressive and standardized commit messages.

For more details, see the [Conventional Emoji Commits documentation](https://conventional-emoji-commits.site/quick-summary/summary).

## Commit Message Format

All commit messages should follow this format:

```
<emoji> <type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Commit Types

Use the following commit types with their corresponding emojis:

**Primary Types:**

- `✨ feat` - A new feature (triggers MINOR version bump)
- `🩹 fix` - A bug fix (triggers PATCH version bump)
- `🚨 BREAKING CHANGE` - API-breaking modifications (triggers MAJOR version bump)

**Additional Types:**

- `📚 docs` - Documentation changes
- `♻️ refactor` - Code refactoring without functional changes
- `🧪 test` - Adding or modifying tests
- `🔧 build` - Changes to build system or dependencies
- `🎨 style` - Formatting changes, missing semicolons, whitespace, etc.
- `⚡️ perf` - Performance improvements

## Examples

**Feature with breaking change:**

```
✨ feat(auth): add OAuth2 support

Implements OAuth2 authentication flow for third-party logins

BREAKING CHANGE: removes legacy authentication endpoints
```

**Simple bug fix:**

```
🩹 fix: resolve image upload bug in Safari
```

**Documentation update:**

```
📚 docs: update installation instructions
```

## Ticket References

For ticket-based projects, add the ticket reference as the last line of the commit message:

```
✨ feat: implement user profile page

PD-12345 User Profile Implementation
```

For non-ticket projects, this reference is optional.

## Additional Resources

For extended documentation on conventional commits, see [this reference](https://github.com/codekiln/alits/blob/feature/story-1.1-foundation-core-package-setup/docs/dev/scm/conventional_commits.md).
