# Task Parsing Patterns

This document provides detailed information about the task list formats supported by the `github-issue-breakdown` skill.

## Overview

The skill parses various common task list formats found in GitHub issue descriptions and converts them into structured sub-issue data. This enables flexible authoring of parent issues while maintaining consistent sub-issue creation.

## Supported Formats

### 1. Markdown Checkboxes (Unchecked)

**Format:**
```markdown
- [ ] Task description here
```

**Behavior:**
- Parses unchecked checkboxes only (by default)
- Skips already-checked items `- [x]` to avoid creating sub-issues for completed tasks
- The `- [ ]` prefix is removed, and the remaining text becomes the sub-issue title

**Example:**

Parent issue body:
```markdown
## Tasks

- [ ] Create login endpoint
- [ ] Create registration endpoint
- [x] Set up database schema
- [ ] Add authentication middleware
```

Parsed tasks:
1. Create login endpoint
2. Create registration endpoint
3. Add authentication middleware

Note: "Set up database schema" is skipped because it's already checked.

### 2. Numbered Lists

**Format:**
```markdown
1. First task
2. Second task
3. Third task
```

**Behavior:**
- Parses any line starting with a number followed by a period
- Number is removed, remaining text becomes sub-issue title
- Order is preserved but numbers are not significant for parsing

**Example:**

Parent issue body:
```markdown
Implementation Steps:

1. Design API schema
2. Implement database models
3. Create REST endpoints
4. Write unit tests
```

Parsed tasks:
1. Design API schema
2. Implement database models
3. Create REST endpoints
4. Write unit tests

### 3. Bullet Points

**Format:**
```markdown
* Task with asterisk
- Task with hyphen
```

**Behavior:**
- Parses lines starting with `*` or `-` (excluding checkboxes)
- Bullet character is removed, remaining text becomes sub-issue title
- Both asterisk and hyphen bullets are supported

**Example:**

Parent issue body:
```markdown
Development Tasks:

* Implement user authentication
* Add error handling
* Update documentation
```

Parsed tasks:
1. Implement user authentication
2. Add error handling
3. Update documentation

### 4. Mixed Formats

**Behavior:**
- The parser supports multiple formats in the same issue
- Each recognized format is parsed independently
- Duplicate detection is not performed (be mindful of mixed formats)

**Example:**

Parent issue body:
```markdown
## Backend Tasks

- [ ] Create API endpoints
- [ ] Add validation

## Frontend Tasks

1. Update login form
2. Add loading states

## DevOps

* Configure CI/CD
* Set up monitoring
```

Parsed tasks:
1. Create API endpoints
2. Add validation
3. Update login form
4. Add loading states
5. Configure CI/CD
6. Set up monitoring

## Parsing Rules

### Line Processing

1. **Split by newline** - Issue body is split into individual lines
2. **Trim whitespace** - Leading and trailing whitespace is removed
3. **Skip empty lines** - Blank lines are ignored
4. **Match patterns** - Each line is tested against regex patterns
5. **Extract text** - Matched prefix is removed to get task title

### Regular Expressions

The script uses these regex patterns:

```python
# Unchecked markdown checkbox
r'^-\s*\[\s*\]'

# Numbered list
r'^\d+\.\s+'

# Bullet point (excluding checkboxes)
r'^[\*-]\s+(?!\[)'
```

### Edge Cases

**Nested lists** - Not supported. Only top-level items are parsed:
```markdown
- [ ] Main task
  - [ ] Subtask (ignored)
```

**Indented lists** - Indentation is stripped, so indented items are treated as top-level:
```markdown
    - [ ] Indented task (parsed as top-level)
```

**Code blocks** - Tasks inside code blocks are parsed (be careful):
````markdown
```
- [ ] This will be parsed
```
````

To avoid this, ensure code blocks don't contain task-like patterns.

**Inline markdown** - Task titles can contain inline markdown:
```markdown
- [ ] Add **authentication** with `JWT` tokens
```
Result: "Add **authentication** with `JWT` tokens"

## Spec-Kit Integration

When using with Spec-Kit generated task lists:

### Spec-Kit Task Format

Spec-Kit may generate task lists like:
```markdown
## Tasks

1. Task 1: Description
   - Subtask details
   - Implementation notes

2. Task 2: Description
   - More details
```

**Parsing behavior:**
- Only numbered items at the start of lines are parsed as tasks
- Indented subtasks and details are ignored
- Consider editing the parent issue to clean up or simplify before creating sub-issues

### Recommended Workflow

1. Run `/speckit.tasks` to generate detailed task breakdown
2. Review the generated tasks in `.specify/tasks/`
3. Create a parent GitHub issue with simplified task list
4. Use this skill to convert to sub-issues
5. Add detailed task information to individual sub-issues as needed

## Customization

### Parsing Checked Items

To parse checked items as well, modify the regex in `scripts/create_subissues.py`:

```python
# Current (unchecked only)
if re.match(r'^-\s*\[\s*\]', line):

# Modified (both checked and unchecked)
if re.match(r'^-\s*\[[x\s]*\]', line):
```

### Custom Delimiters

To add custom task delimiters (e.g., `TODO:` prefix):

```python
# Add after bullet point check
elif line.startswith('TODO:'):
    task = line.replace('TODO:', '').strip()
    if task:
        tasks.append(task)
```

### Filtering by Section

To parse only tasks under specific headers:

```python
def parse_tasks(body: str, section: str = None) -> List[str]:
    """Parse tasks, optionally from a specific section."""
    tasks = []
    in_section = section is None  # Parse all if no section specified

    for line in body.split('\n'):
        if section and line.strip().startswith('#'):
            # Check if we entered or left the target section
            in_section = section.lower() in line.lower()
            continue

        if not in_section:
            continue

        # ... rest of parsing logic
```

## Testing Parsing

### Dry Run

Always test parsing with `--dry-run` first:

```bash
python scripts/create_subissues.py --issue 42 --dry-run
```

This shows what tasks were parsed without creating sub-issues.

### Common Issues

**No tasks found:**
- Check that task list uses supported formats
- Verify there are no extra characters before task markers
- Ensure lines are not within code blocks

**Unexpected tasks parsed:**
- Review issue body for patterns that match regex
- Consider using more specific task formats
- Edit parent issue to clarify task list section

**Duplicate tasks:**
- Avoid mixing formats that create duplicates
- Example: `- [ ] Task` and `- Task` will create two items

## Examples

### Good Task List

```markdown
## Overview
This feature adds user authentication.

## Implementation Tasks

- [ ] Create login endpoint at /api/auth/login
- [ ] Create registration endpoint at /api/auth/register
- [ ] Implement JWT token generation
- [ ] Add authentication middleware for protected routes
- [ ] Write integration tests

## Acceptance Criteria
Users can register, login, and access protected resources.
```

Result: 5 sub-issues created

### Task List with Context

```markdown
## Background
We need to improve our authentication system.

## Technical Approach
Use JWT tokens for stateless authentication.

## Implementation Steps

1. Design JWT token structure
2. Implement token generation service
3. Create authentication middleware
4. Update existing endpoints
5. Add token refresh mechanism

## Testing Plan
- Unit tests for token service
- Integration tests for auth flow
```

Result: 5 sub-issues created (only numbered items)

### Mixed Format (Be Careful)

```markdown
## Tasks

Backend:
- [ ] API endpoints
- [ ] Database models

Frontend:
- Update forms
- Add loading states
```

Result: 4 sub-issues, but "Frontend:" becomes a task title too.
Better to use consistent format or headers.

## Troubleshooting

### Pattern Not Matching

If expected tasks aren't parsed:

1. Copy issue body to a text editor
2. Check for invisible characters or formatting
3. Verify format matches examples above
4. Test with `--dry-run` to see what's parsed

### Unwanted Items Parsed

If unexpected items become tasks:

1. Review issue body for patterns matching regex
2. Edit issue to make task list clearer
3. Use consistent formatting throughout
4. Consider using a single format type

### Empty Task Titles

If task titles are empty or just whitespace:

1. Ensure there's text after the task marker
2. Check for lines that are just markers: `- [ ]`
3. Edit parent issue to add descriptions

## Best Practices

1. **Use consistent formatting** - Pick one format and stick to it
2. **Clear task descriptions** - Make titles self-contained
3. **Avoid nesting** - Keep tasks at the same level
4. **Separate concerns** - Use headers to organize, not for parsing
5. **Test first** - Always dry-run before creating
6. **Edit if needed** - Clean up parent issue body before parsing
7. **Document context** - Add overview sections that won't be parsed

## Future Enhancements

Potential improvements to parsing:

- Support for nested task hierarchies
- Custom delimiter configuration
- Section-based filtering
- Task priority detection (e.g., `HIGH:` prefix)
- Automatic deduplication
- Task dependency detection

These would require script modifications. See `scripts/create_subissues.py` for implementation.
