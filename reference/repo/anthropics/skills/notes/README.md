# anthropics/skills Reference

## Repository Information

- **URL**: https://github.com/anthropics/skills
- **Purpose**: Official Claude Code skills marketplace and examples
- **Cloned**: 2025-11-07

## Why This Reference Exists

This repository is referenced by the Langstar project for the following purposes:

### 1. Skill Development Patterns

The anthropics/skills repository contains official examples of Claude Code skills that demonstrate:
- **Naming conventions**: How skills should be named (gerund form: `processing-pdfs`, `analyzing-spreadsheets`)
- **Structure patterns**: Organization of SKILL.md, supporting scripts, and documentation
- **Best practices**: Real-world implementations following Claude Code guidelines
- **Metadata format**: Proper frontmatter configuration for skill discovery

### 2. Example Skills for Reference

When creating new skills for Langstar (like `gh-sub-issue` and the upcoming `git-worktrees` skill), we reference these examples to ensure:
- Consistent structure with the broader Claude Code ecosystem
- Adherence to documented best practices
- Proper use of progressive disclosure architecture
- Appropriate degrees of freedom for different task types

### 3. Skill Transformation Learning

Referenced while transforming `gh-issue-link-parent-to-child` → `gh-sub-issue`:
- Studied skill organization patterns
- Verified naming conventions
- Reviewed description format (functionality + activation triggers)
- Confirmed documentation structure and length guidelines

### 4. Upcoming Worktree Skill Development

Will be referenced for creating a new `git-worktrees` skill (or similar) to standardize:
- Git worktree lifecycle management
- Integration with project branching conventions
- Cleanup and hygiene procedures
- Sub-issue hierarchy handling

## Related Project Documentation

- `.claude/skills/` - Langstar's project-specific skills
- `docs/dev/github-workflow.md` - Branching conventions and issue workflow
- Best practices: https://docs.claude.com/en/docs/agents-and-tools/agent-skills/best-practices.md

## Usage Notes

- The `code/` directory contains the full clone (gitignored)
- This `notes/` directory is version-controlled for team knowledge sharing
- Add additional `.md` files here for specific skill analysis or learnings
