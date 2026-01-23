@AGENTS.md

## Testing Standards

This project enforces the **Toyota Andon Cord principle** for testing: any failing test stops the merge process. No exceptions.

**Pre-commit requirements:**
```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo nextest run --profile ci --all-features --workspace
```

**For comprehensive testing guidelines:**
- The TOC is auto-loaded via AGENTS.md (`docs/dev/testing/README.md`)
- Always follow `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` (load explicitly)
- Use CRUD lifecycle pattern for integration tests (see `docs/dev/testing/crud-lifecycle-pattern.md`)

**Never acceptable:**
- "My changes didn't introduce this failure" ❌
- Merging with failing tests ❌
- Exit-code-only tests (must verify actual behavior) ❌

See milestone #556 and issue #536 case study for context.

## Output Token Budget

EXTREMELY IMPORTANT - before you begin any task, check the value of the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable and avoid going over that limit in your output. This helps manage API costs and ensures responses stay within configured limits.

**Default behavior when not set:**
- If `CLAUDE_CODE_MAX_OUTPUT_TOKENS` is not set or empty, there is no explicit token limit
- However, always aim for concise, focused responses regardless of limits
- Typical values range from 4000-16000 tokens depending on task complexity
  (4000 tokens may be suitable for very simple or highly concise tasks, but)
- **Recommended default:** 8000 tokens for most tasks, 16000 for complex analysis

When working on complex tasks:
- Break work into smaller chunks if needed
- Use tools efficiently to gather information
- Summarize findings rather than dumping raw data
- Prioritize actionable information over exhaustive detail

**Handling tasks that exceed token limits:**
- For large file reviews: process in sections or use focused analysis
- For extensive code changes: break into multiple commits
- For comprehensive reports: create summary first, offer detailed sections on request
- **If a task inherently requires more tokens:** Inform the user and ask if they want to:
  1. Proceed with a summarized version within the limit
  2. Break the task into multiple smaller requests
  3. Temporarily increase the limit for this specific task

## Background Task Management

**CRITICAL: Never use `tail -f` to monitor background tasks** - it runs indefinitely and leaves processes running.

**Correct approach for monitoring background tasks:**

```bash
# ❌ WRONG - leaves tail -f running forever
cargo nextest run --workspace &
tail -f /tmp/output.log

# ✅ CORRECT - use TaskOutput with block=true
cargo nextest run --workspace  # This runs in background automatically
# Then use TaskOutput tool with block=true to wait for completion
```

**Rules:**
- Use `TaskOutput` tool with `block=true` to wait for background command completion
- NEVER use `tail -f` - it runs until manually killed
- If you accidentally start `tail -f`, immediately use `KillShell` to stop it
- Background tasks from Bash tool are automatically monitored - don't add extra monitoring
