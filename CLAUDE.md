@AGENTS.md

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
