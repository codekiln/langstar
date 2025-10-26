# GitHub Spec-Kit Integration

## Overview

Langstar integrates [GitHub Spec-Kit](https://github.com/github/spec-kit), an open-source toolkit that facilitates spec-driven development workflows. Spec-Kit emphasizes intent-driven development where detailed specifications ("what" and "why") precede implementation ("how").

## Philosophy

Spec-Kit flips traditional development by having developers create detailed specifications first, which AI agents then use to generate implementations. This approach:

- Emphasizes clear requirements before coding
- Reduces ambiguity through structured specifications
- Enables AI agents to generate more accurate implementations
- Creates living documentation that stays in sync with code

## Directory Structure

Spec-Kit creates a `.specify/` directory in the project root:

```
.specify/
├── memory/
│   └── constitution.md       # Project governing principles
├── scripts/                   # Automation scripts
│   └── bash/                  # Shell scripts for workflow automation
├── specs/                     # Feature-specific documentation
└── templates/                 # Templates for specs, plans, and tasks
    ├── agent-file-template.md
    ├── checklist-template.md
    ├── plan-template.md
    ├── spec-template.md
    └── tasks-template.md
```

## Slash Commands

Spec-Kit provides slash commands for Claude Code that guide the development workflow:

### Core Workflow Commands

Use these commands in sequence for spec-driven development:

1. **`/speckit.constitution`** - Establish project principles
   - Define governing principles and constraints
   - Set architectural guidelines
   - Document technical standards

2. **`/speckit.specify`** - Create baseline specification
   - Define requirements for a feature
   - Document expected behavior
   - Establish acceptance criteria

3. **`/speckit.plan`** - Create implementation plan
   - Break down the specification into technical tasks
   - Identify dependencies and risks
   - Plan the implementation approach

4. **`/speckit.tasks`** - Generate actionable tasks
   - Create a detailed task list from the plan
   - Assign priorities and estimates
   - Organize work into manageable chunks

5. **`/speckit.implement`** - Execute implementation
   - Guide AI agent through implementation
   - Ensure alignment with specification
   - Validate against acceptance criteria

### Enhancement Commands (Optional)

These commands improve quality and reduce risk:

- **`/speckit.clarify`** - Ask structured questions to de-risk ambiguous areas
  - Use before `/speckit.plan` to resolve uncertainties
  - Identifies assumptions and edge cases
  - Ensures shared understanding

- **`/speckit.analyze`** - Cross-artifact consistency & alignment report
  - Use after `/speckit.tasks`, before `/speckit.implement`
  - Validates consistency across specs, plans, and tasks
  - Identifies gaps or contradictions

- **`/speckit.checklist`** - Generate quality checklists
  - Use after `/speckit.plan`
  - Validates requirements completeness
  - Ensures clarity and consistency

## Integration with GitHub Workflow

Spec-Kit complements the existing GitHub issue-driven workflow:

### Typical Workflow

1. **Create GitHub Issue** - Document what needs to be done
2. **Create Branch** - Follow branch naming conventions
3. **Run `/speckit.specify`** - Create detailed specification
4. **Run `/speckit.plan`** - Plan the implementation
5. **Run `/speckit.tasks`** - Generate task list
6. **Run `/speckit.implement`** - Execute implementation
7. **Create Pull Request** - Submit changes for review
8. **Review & Merge** - Complete the workflow

### When to Use Spec-Kit

**Use Spec-Kit for:**
- Complex features requiring detailed planning
- Features with ambiguous requirements
- Major architectural changes
- Features that will be implemented by AI agents
- Work that requires clear documentation

**Skip Spec-Kit for:**
- Simple bug fixes
- Trivial changes
- Well-understood modifications
- Documentation updates
- Configuration changes

## Tool Requirements

### Prerequisites

Spec-Kit requires:
- Python 3.11+
- uv package manager
- mise tool version manager

All prerequisites are pre-installed in the Langstar devcontainer.

### Installation

The devcontainer automatically installs Spec-Kit on creation:

```bash
# postCreateCommand in devcontainer.json
mise trust && mise install && uv tool install specify-cli --from git+https://github.com/github/spec-kit.git
```

### Manual Installation

If needed, install manually:

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install specify-cli
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git

# Verify installation
specify --version
```

## Example Usage

### Creating a New Feature

```bash
# 1. Create and checkout branch
git checkout -b username/42-add-authentication

# 2. Use Claude Code with Spec-Kit commands
```

In Claude Code:
```
User: I need to implement user authentication with JWT tokens

Claude: Let me help you with that using Spec-Kit.

# Claude runs /speckit.specify
# Creates detailed specification in .specify/specs/

User: /speckit.plan

# Claude creates implementation plan

User: /speckit.tasks

# Claude generates task list

User: /speckit.implement

# Claude implements according to spec
```

### 3. Commit and Create PR

```bash
git add .
git commit -m "✨ feat: add JWT authentication

Fixes #42"
git push origin username/42-add-authentication

# Create pull request
gh pr create --title "✨ feat: add JWT authentication" --body "Fixes #42"
```

## Best Practices

### Specification Writing

- **Be specific**: Provide clear, unambiguous requirements
- **Include examples**: Show expected behavior with examples
- **Define edge cases**: Document how to handle unusual scenarios
- **Set acceptance criteria**: Make success measurable

### Plan Creation

- **Break down complexity**: Divide large features into smaller components
- **Identify dependencies**: Note what must be done first
- **Consider risks**: Document potential issues and mitigations
- **Be realistic**: Estimate effort accurately

### Implementation

- **Follow the spec**: Stay aligned with documented requirements
- **Update as needed**: Revise specs if requirements change
- **Test thoroughly**: Verify against acceptance criteria
- **Document decisions**: Explain why, not just what

## Version Control

### What to Commit

**Always commit:**
- `.specify/memory/constitution.md` - Project principles
- `.specify/specs/**` - Feature specifications
- `.specify/templates/**` - Custom templates

**Consider gitignoring:**
- `.specify/.agent/` - May contain API keys or credentials
- Temporary working files

### Recommended .gitignore Additions

```gitignore
# Spec-Kit agent folder (may contain credentials)
.specify/.agent/
.specify/**/.cache/

# Temporary specification working files
.specify/**/*-draft.md
.specify/**/*-wip.md
```

## Troubleshooting

### Command Not Found

If `/speckit.*` commands are not available:

1. Check `.claude/commands/` directory exists
2. Verify speckit command files are present:
   ```bash
   ls .claude/commands/speckit.*
   ```
3. Restart Claude Code or reload the workspace

### specify CLI Not Found

If `specify` command is not available:

```bash
# Check if uv is installed
uv --version

# Reinstall specify-cli
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git

# Verify installation
specify --version
```

### Python Version Issues

Spec-Kit requires Python 3.11+:

```bash
# Check Python version
python3 --version

# If version is too old, mise should install correct version
mise install python
mise use python@3.11
```

## Additional Resources

- [GitHub Spec-Kit Repository](https://github.com/github/spec-kit)
- [Spec-Kit Documentation](https://github.com/github/spec-kit/blob/main/README.md)
- [uv Documentation](https://docs.astral.sh/uv/)
- [mise Documentation](https://mise.jdx.dev/)

## Integration with Claude Code

The `.claude/commands/` directory contains all Spec-Kit slash commands, which are automatically available in Claude Code. These commands provide structured prompts that guide the AI through the spec-driven development process.

For more information on Claude Code slash commands, see the [Claude Code documentation](https://docs.claude.com/en/docs/claude-code).