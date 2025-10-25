# Claude Code Configuration

This directory contains project-level configuration for Claude Code.

## Files

### `settings.json` (version controlled)
Project-wide configuration that defines:
- Plugin marketplaces
- Enabled plugins
- Shared team settings

**Important**: This file is committed to git and shared with all team members.

### `settings.local.json` (gitignored)
Local overrides for:
- API credentials
- Environment variables
- Personal preferences
- Permissions

**Important**: This file is NOT committed to git. It's for your local environment only.

## Plugin Configuration

This project uses the official Anthropic skills marketplace:
- **Repository**: [anthropics/skills](https://github.com/anthropics/skills)
- **Marketplace Name**: `anthropic-agent-skills`

### Enabled Plugins

- **example-skills@anthropic-agent-skills**
  - Skill creation and MCP building
  - Visual design and algorithmic art
  - Internal communications templates
  - Web application testing
  - Artifact building
  - Slack GIF creation
  - Theme styling
  - Brand guidelines

- **document-skills@anthropic-agent-skills**
  - Excel (.xlsx) processing
  - Word (.docx) editing
  - PowerPoint (.pptx) creation
  - PDF manipulation

## How It Works

When you open this project in a devcontainer and trust the repository:
1. Claude Code reads `settings.json`
2. Automatically clones the `anthropics/skills` repository
3. Installs and enables the specified plugins
4. Merges with any settings from `settings.local.json`

No manual plugin installation needed!
