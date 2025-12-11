# ls-claude-code-plugin Feasibility Scout

**Issue**: #681
**Date**: 2025-12-10
**Status**: Complete - Recommendation: GO

## Executive Summary

**Feasibility**: **GO** ✅

**Vision**: Create a Claude Code plugin marketplace in the langstar repo, enabling non-technical SMEs to manage LangSmith and LangGraph through natural language in VS Code.

**Key Findings**:
- **Low-Medium Complexity**: Plugin infrastructure requires only configuration files; commands wrap existing CLI
- **No Blockers**: All required infrastructure exists (langstar CLI, Claude Code plugin system, Codespaces)
- **High Value**: Bridges gap between comprehensive CRUD CLI and natural language SME workflows
- **Clear Path**: Start with 5 MVP commands, expand based on user feedback

**Recommendation**: Proceed to full 8-phase milestone with Phase 0 experiments and Phase 1 MVP focusing on 5 high-value commands.

## 1. User Requirements

### Target Users
Professional but non-technical Subject Matter Experts:
- Using Claude Code extension in VS Code
- Working in GitHub Codespaces with pre-configured LangSmith env vars
- Need to accomplish LangSmith/LangGraph tasks through natural language

### Required Workflows

**Prompt Management:**
- Workshop prompts (get details, edit, prompt engineering, save)
- Add prompt versions with different models
- Find and get recommendations for reasoning models
- Create prompts with structured outputs matching other prompts
- Draft classification prompts

**Run & Trace Management:**
- Find recent runs matching filtering criteria
- Add runs to annotation queues
- Get or create annotation queues by naming criteria

**Deployment & Graph Management:**
- Add assistants to graphs in deployments referencing prompts

**Dataset & Evaluation:**
- Find/create datasets and populate with runs from projects
- Create evals for datasets

## 2. Claude Code Plugin Architecture Research

### How Plugin System Works

**Installation & Discovery:**
- Users install plugins via `/plugin` command or configure in `.claude/settings.json`
- Plugins can be shared across projects and teams
- Multiple community marketplaces supported (not just official Anthropic plugins)
- Plugins discovered through marketplace registries

**Plugin Components** (all optional):
- **Commands** (`/commands/`) - Slash commands that extend Claude's capabilities
- **Agents** (`/agents/`) - Specialized sub-agents for complex workflows
- **Skills** (`/skills/`) - Reusable knowledge and tool combinations
- **Hooks** (`/hooks/`) - Event handlers for workflow automation
- **MCP** (`.mcp.json`) - External tool integrations

### Plugin Structure

Standard plugin directory structure:
```
plugin-name/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata (name, version, author, description)
├── commands/                # Slash commands (optional)
│   └── *.md                # Command definitions with YAML frontmatter
├── agents/                  # Specialized agents (optional)
├── skills/                  # Agent Skills (optional)
├── hooks/                   # Event handlers (optional)
├── .mcp.json               # External tool configuration (optional)
└── README.md               # Plugin documentation
```

### Command File Format

Commands are markdown files with YAML frontmatter:

```markdown
---
allowed-tools: Bash(git add:*), Bash(git status:*), Bash(git commit:*)
description: Create a git commit
---

## Context

- Current git status: !`git status`
- Current git diff: !`git diff HEAD`

## Your task

Based on the above changes, create a single git commit.
```

**Key elements:**
- `allowed-tools`: Restricts which tools Claude can use
- `description`: Shown in command help
- Context section: Uses `!` prefix to execute commands and show output to Claude
- Task section: Instructs Claude what to do

### Marketplace System

**Repository-based marketplace:**
- Plugins stored in `plugins/` directory
- Users reference marketplace URL to discover plugins
- Plugin metadata in `.claude-plugin/plugin.json`
- No central registry required - any repo can be a marketplace


## 3. Existing Langstar Capabilities

### Already Implemented

**Prompt Management** (`langstar prompt`):
- ✅ list - List all prompts
- ✅ get - Get specific prompt details
- ✅ search - Search for prompts
- ✅ push - Push/create prompt in PromptHub
- ✅ pull - Pull prompt from PromptHub

**Run Management** (`langstar runs`):
- ✅ query - Query runs with filtering and pagination

**Queue Management** (`langstar queue`):
- ✅ list - List annotation queues
- ✅ create - Create new queue
- ✅ get - Get queue details
- ✅ update - Update queue
- ✅ delete - Delete queue
- ✅ add-runs - Add runs to queue
- ✅ remove-run - Remove run from queue
- ✅ items - List runs in queue

**Dataset Management** (`langstar dataset`):
- ✅ create - Create new dataset
- ✅ list - List datasets
- ✅ get - Get dataset details
- ✅ update - Update dataset
- ✅ delete - Delete dataset
- ✅ import - Import examples from file (JSONL/CSV)
- ✅ list-examples - List examples in dataset
- ✅ export - Export examples to file

**Evaluation Management** (`langstar eval`):
- ✅ create - Create evaluation configuration
- ✅ run - Run evaluation
- ✅ list - List evaluations
- ✅ get - Get evaluation details
- ✅ export - Export evaluation results

**Model Configuration** (`langstar model-config`):
- ✅ Manage LangSmith model configurations (playground settings)

**Deployment & Graph Management**:
- ✅ `langstar assistant` - Manage LangGraph assistants
- ✅ `langstar deployment` - Manage deployments
- ✅ `langstar graph` - Inspect graphs within deployments

**Configuration**:
- ✅ `langstar config` - Manage configuration settings
- ✅ `langstar secrets` - Manage workspace secrets

### Gaps Requiring New Development

**Prompt Workflow Enhancements:**
- ❌ Prompt "workshopping" - interactive editing with prompt engineering guidance
- ❌ Model recommendation system - suggesting best models for use cases
- ❌ Structured output template creation - generating JSON schemas
- ❌ Prompt versioning comparison - diffing versions

**Run Management Enhancements:**
- ❌ Natural language query builder - convert "recent runs from project X matching Y" to API query
- ❌ Bulk operations - adding multiple runs to queues at once

**Queue Management Enhancements:**
- ❌ "Get or create" pattern - idempotent queue creation by name
- ❌ Smart queue population - rules for auto-adding runs

**Deployment Workflow:**
- ❌ Assistant creation with prompt references - linking prompts to assistants in graphs

**Cross-Cutting:**
- ❌ Natural language command construction - "find recent runs" → `langstar runs query...`
- ❌ Context-aware suggestions - recommending next steps based on workflow
- ❌ Interactive confirmation for destructive operations

**Key Insight:** The langstar CLI provides excellent **CRUD coverage** but lacks **workflow orchestration** and **natural language interfaces** that SMEs need.


## 4. Proposed Plugin Design

### Directory Structure
```
langstar/
├── .claude-plugins/
│   └── marketplace.json              # Marketplace registry
├── plugins/
│   └── manage-langstar/
│       ├── .claude-plugin/
│       │   └── plugin.json           # Plugin metadata
│       ├── commands/                 # Slash commands
│       │   ├── prompt-workshop.md
│       │   ├── find-runs.md
│       │   ├── queue-manage.md
│       │   └── dataset-create.md
│       ├── skills/                   # Skills (if needed)
│       │   └── langstar-cli.md       # Knowledge about langstar CLI
│       ├── agents/                   # Sub-agents (Phase 2+)
│       └── README.md
└── [existing langstar repo structure]
```

### Marketplace.json Structure

```json
{
  "name": "langstar-plugin-marketplace",
  "description": "Official plugin marketplace for langstar - LangSmith and LangGraph CLI tools",
  "plugins": [
    {
      "name": "manage-langstar",
      "path": "./plugins/manage-langstar",
      "description": "Natural language interface for managing LangSmith prompts, runs, queues, datasets, and evaluations"
    }
  ]
}
```

### Plugin.json Structure

```json
{
  "name": "manage-langstar",
  "description": "Natural language interface for LangSmith and LangGraph workflows via langstar CLI",
  "version": "0.1.0",
  "author": {
    "name": "Codekiln",
    "email": "support@codekiln.com"
  }
}
```

### Mapping: Workflows → Implementation

**Phase 1: Slash Commands (MVP)**

| User Workflow | Command | Implementation Strategy |
|--------------|---------|-------------------------|
| Workshop prompt | `/langstar-prompt-workshop <name>` | Fetch prompt → present to user → guide editing → save |
| Find runs | `/langstar-find-runs` | Interactive query builder → execute `langstar runs query` |
| Manage queues | `/langstar-queue-manage` | List queues → get/create by name → add runs |
| Create dataset | `/langstar-dataset-create` | Interactive wizard → populate from runs |
| Model recommendations | `/langstar-models` | Query model-config → present capabilities → recommend |

**Allowed Tools Pattern:**
```markdown
---
allowed-tools: Bash(langstar prompt:*), Bash(langstar runs:*), Bash(langstar queue:*), Read(*), Write(*)
description: Workshop a LangSmith prompt interactively
---
```

**Phase 2: Skills** (optional, if common patterns emerge)

Skills provide reusable knowledge:
- `skills/langstar-cli.md` - Comprehensive guide to langstar CLI commands
- `skills/prompt-engineering.md` - Best practices for prompt design
- `skills/evaluation-design.md` - How to structure evals

**Phase 3: Sub-Agents** (for complex multi-step workflows)

Agents for complex workflows:
- `agents/prompt-optimizer/` - Iterative prompt improvement with testing
- `agents/dataset-curator/` - Intelligent dataset creation from run filters
- `agents/eval-designer/` - Full eval creation from requirements to execution

### Key Design Decisions

**Start with Commands, not Skills/Agents:**
- Commands provide immediate value with clear scope
- Skills and agents can be added later if patterns emerge
- Reduces complexity for initial implementation

**Leverage Existing CLI:**
- Plugin wraps langstar CLI, doesn't duplicate functionality
- `allowed-tools: Bash(langstar:*)` gives full CLI access
- No new SDK methods required for Phase 1

**Natural Language → Structured Commands:**
- Commands translate user intent to CLI invocations
- Use Claude's understanding to build complex queries
- Interactive prompting when parameters unclear


## 5. Complexity Assessment

**Overall Complexity**: **Low-Medium**

### Breakdown by Component

**Plugin infrastructure setup: LOW**
- Create `.claude-plugins/marketplace.json` (1 file)
- Create `plugins/manage-langstar/.claude-plugin/plugin.json` (1 file)
- Create `plugins/manage-langstar/README.md` (1 file)
- Total: ~3 files, ~100 lines of JSON/markdown
- No code required, just configuration

**Prompt management workflows: LOW**
- Existing CLI commands cover all CRUD operations
- Commands simply wrap `langstar prompt` CLI
- Example: `/langstar-prompt-workshop` → interactive wrapper around `langstar prompt get`, `langstar prompt push`
- Each command: ~50-100 lines of markdown

**Run/trace management: LOW**
- `langstar runs query` already implements filtering
- Command translates natural language to CLI flags
- Example: "recent runs from project X" → `langstar runs query --project=X --limit=50`
- Minimal complexity, pure orchestration

**Queue management: LOW-MEDIUM**
- Most operations already in CLI (`langstar queue`)
- "Get or create" pattern requires conditional logic in command
- Example workflow: `list → check exists → create if not → add-runs`
- Slightly more complex than simple CRUD wrapping

**Dataset/eval workflows: LOW-MEDIUM**
- CLI provides all primitives
- Commands orchestrate multi-step workflows
- Example: create dataset → import examples from runs → create eval config
- Logic complexity in command markdown, not code

**Deployment/graph management: MEDIUM**
- Less mature CLI support (based on API completeness)
- May require new CLI commands for assistant+prompt linking
- Could be deferred to Phase 2

### Risk Assessment

**Technical Risks: LOW**
- Claude Code plugin system is stable and documented
- Langstar CLI is mature with good test coverage
- No new API endpoints required for Phase 1
- No Rust SDK changes needed initially

**Scope Risks: MEDIUM**
- User requirements are broad (10+ workflows)
- Risk of scope creep if trying to handle all workflows in Phase 1
- Mitigation: Start with 3-5 most valuable commands

**Adoption Risks: LOW**
- Target users already use VS Code + Claude Code
- GitHub Codespaces pre-configured with langstar
- Natural language interface matches SME mental model

### Blockers

**No hard blockers identified.**

**Soft dependencies:**
- Langstar CLI must be installed in user environment (already true for Codespaces)
- Claude Code plugin system must support repository-based marketplaces (confirmed in docs)
- Users need GitHub Codespace setup (already requirement)

## 6. Experiments

No experiments run during scout phase. Recommendations for Phase 0 (if Go):

**Experiment 1: Install existing plugin**
- Install a community plugin via `/plugin` command
- Validate marketplace discovery works
- Document user experience

**Experiment 2: Create minimal plugin**
- Create single-command plugin locally
- Test command execution
- Validate `allowed-tools` restrictions work as expected

**Experiment 3: Test langstar CLI in commands**
- Create test command that wraps `langstar prompt list`
- Verify output formatting in Claude context
- Validate JSON vs table output handling

## 7. Recommendation

**Decision**: **GO** ✅

**Rationale:**
1. **High Value, Low Complexity**: Enables non-technical SMEs to use LangSmith/LangGraph through natural language with minimal implementation effort
2. **No Blockers**: All required infrastructure exists (CLI, plugin system, Codespaces)
3. **Incremental Path**: Can start with 3-5 commands, validate value, then expand
4. **Low Risk**: Wraps existing CLI, no SDK changes or new API endpoints needed
5. **Clear User Need**: Existing langstar CLI has CRUD coverage but lacks workflow orchestration

**Next Steps**:

1. **Create Milestone**: `ls-claude-code-plugin`
2. **Create Phase 0 Parent Issue**: Document full 8-phase plan
3. **Prioritize Workflows**: Choose 3-5 highest-value commands for Phase 1
4. **Run Experiments** (Phase 0.1): Validate plugin development workflow

**Recommended Phases**

### Phase 0: Planning & Design (Research)
- 0.0: Scout (this document) ✅
- 0.1: Experiments - validate plugin development workflow
- 0.2: Detailed design - command specifications for MVP

### Phase 1: Plugin Infrastructure (MVP)
- 1.1: Create marketplace.json and plugin.json
- 1.2: Implement 3-5 core commands
- 1.3: Documentation and testing

### Phase 2: Expanded Commands
- 2.1: Add remaining command workflows
- 2.2: Refine based on user feedback

### Phase 3: Skills (optional)
- 3.1: Extract common patterns into reusable skills
- 3.2: Langstar CLI reference skill

### Phase 4: Sub-Agents (advanced)
- 4.1: Complex workflow agents (prompt-optimizer, dataset-curator)
- 4.2: Agent integration testing

### Phase 5-8: Standard release phases
- Phase 5: Testing
- Phase 6: Documentation
- Phase 7: Integration
- Phase 8: Release

**MVP Command Recommendations** (Phase 1):
1. `/langstar-prompt-workshop` - Highest user value, showcases interactive workflows
2. `/langstar-find-runs` - Demonstrates natural language query building
3. `/langstar-queue-manage` - Shows get-or-create pattern
4. `/langstar-models` - Simple but valuable, displays model recommendations
5. `/langstar-dataset-create` - Multi-step workflow, high SME value

**Success Metrics**:
- Plugin installable via `/plugin` command
- All 5 MVP commands functional and tested
- At least 3 user workflows documented with examples
- Positive feedback from initial SME users
