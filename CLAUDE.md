@AGENTS.md

## Output Token Budget

EXTREMELY IMPORTANT - before you begin any task, check the value of the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable and avoid going over that limit in your output. This helps manage API costs and ensures responses stay within configured limits.

**Default behavior when not set:**
- If `CLAUDE_CODE_MAX_OUTPUT_TOKENS` is not set or empty, there is no explicit token limit
- However, always aim for concise, focused responses regardless of limits
- Typical values range from 4000-16000 tokens depending on task complexity

When working on complex tasks:
- Break work into smaller chunks if needed
- Use tools efficiently to gather information
- Summarize findings rather than dumping raw data
- Prioritize actionable information over exhaustive detail

**Handling tasks that exceed token limits:**
- For large file reviews: process in sections or use focused analysis
- For extensive code changes: break into multiple commits
- For comprehensive reports: create summary first, offer detailed sections on request
