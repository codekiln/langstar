# ls-claude-code-plugin Feasibility Scout (Revised)

**Issue**: #681
**Date**: 2025-12-11 (Revised)
**Status**: In Progress - Redesigning approach

## Executive Summary

**Feasibility**: **Conditional GO** - Contingent on ls-prompt-ux milestone co-evolution

**Revised Vision**: Build a single, excellent prompt management experience for Claude Code before expanding to other LangSmith capabilities. Go deep before going wide.

**Key Insight from Feedback**:
> "We should start with a single use case that will be very valuable to SMEs, and really get that interface good."

**Critical Dependency**: The `langstar prompt` CLI is in flux (see [ls-prompt-ux milestone #16](https://github.com/codekiln/langstar/milestone/16)). The plugin and CLI must co-evolve, with the plugin serving as a QA mechanism for validating Claude-friendly CLI design.

## What Changed from v1

| Aspect | v1 Approach | v2 Approach |
|--------|-------------|-------------|
| **Scope** | 10+ workflows, 5 MVP commands | Single workflow: prompt management |
| **Depth** | Surface-level coverage of many areas | Deep, excellent UX for one area |
| **CLI Relationship** | Wrap existing CLI | Co-evolve CLI and plugin together |
| **QA Strategy** | Not addressed | Plugin as QA mechanism for agentic instructions |
| **Sub-agents** | Complex workflows | Context management (filter noise for parent agent) |
| **Other workflows** | Phase 2-4 expansion | Deferred until prompt management is excellent |

---

## 1. Focus: Prompt Management

### Why Prompts First

1. **80% of value**: Prompt management is the most frequent SME workflow
2. **Active CLI work**: ls-prompt-ux milestone (#16) is improving prompt CLI UX
3. **Synergy**: Issue #679 already designs "AI-first UX" recognizing Claude as primary user
4. **Measurable**: Clear success criteria - can Claude help an SME effectively manage prompts?

### Target Workflow: Prompt Engineering Lifecycle

```
User: "Help me workshop my classification prompt"
      ↓
Claude: Gets prompt details (template, metadata, schema if structured)
      ↓
User: Describes desired changes
      ↓
Claude: Suggests improvements, explains prompt engineering principles
      ↓
User: Approves changes
      ↓
Claude: Pushes new version with descriptive commit message
```

This single workflow exercises:
- `langstar prompt get/show` - retrieve prompt details
- `langstar prompt cat` - get editable template text
- `langstar prompt push` - save changes
- Understanding of structured outputs, models, versions

### Current CLI Status (Why Co-Evolution is Required)

From [Issue #679 - Design prompt command structure](https://github.com/codekiln/langstar/issues/679):

> **Problem**: "The current `langstar prompt` commands have fundamental UX issues... Claude is the primary user - help text must be self-explanatory"

**Current confusion** (from #679):
- `get` vs `pull` distinction unclear to AI agents
- Schema only visible in `pull`, not `get`
- No command to get just the template text for editing
- Help text not optimized for Claude parsing

**Proposed solutions being evaluated**:
```bash
# Option B (Multiple commands by intent):
langstar prompt info <handle>    # Metadata
langstar prompt cat <handle>     # Template text only
langstar prompt schema <handle>  # Schema only
langstar prompt show <handle>    # Everything friendly
```

**Implication for plugin**: We cannot build an excellent plugin experience on top of a CLI that Claude can't effectively use. The plugin development must inform and validate the CLI redesign.

---

## 2. Co-Evolution Strategy: CLI + Plugin

### The Feedback Loop

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CO-EVOLUTION CYCLE                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────┐         ┌──────────────────┐                    │
│  │ ls-prompt-ux  │         │ Claude Code      │                    │
│  │ CLI Design    │◄───────►│ Plugin           │                    │
│  │ (#16)         │         │ Development      │                    │
│  └───────┬───────┘         └────────┬─────────┘                    │
│          │                          │                              │
│          │    ┌─────────────────────┘                              │
│          │    │                                                    │
│          ▼    ▼                                                    │
│     ┌─────────────────┐                                            │
│     │ Testing:        │                                            │
│     │ Can Claude help │                                            │
│     │ SME manage      │                                            │
│     │ prompts well?   │                                            │
│     └────────┬────────┘                                            │
│              │                                                     │
│              ▼                                                     │
│     ┌─────────────────┐                                            │
│     │ Iterate until   │                                            │
│     │ excellent       │                                            │
│     └─────────────────┘                                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Development Approach

**Phase 1: Validate CLI Foundations**
1. Review #679 design decisions
2. Implement minimal `/langstar-prompt-workshop` command
3. Test: Can Claude parse CLI help and outputs effectively?
4. Feed findings back to ls-prompt-ux milestone

**Phase 2: Iterate on Both**
1. As CLI improves, update plugin commands
2. As plugin reveals pain points, update CLI
3. Document: "What makes CLI output Claude-friendly?"

**Phase 3: Excellence Gate**
Before expanding to runs/queues/datasets:
- [ ] SME can complete full prompt engineering workflow
- [ ] Claude successfully parses all prompt CLI outputs
- [ ] Error messages guide Claude to correct actions
- [ ] Help text enables Claude to choose correct commands

---

## 3. QA Methodology for Agentic Instructions

### The Challenge

> "AFAIK it's experimental to do QA on agentic coding instructions, but it's extremely important, because essentially one of the only metrics of quality that this project has is how useful it is when wielded by an agentic coding agent on behalf of a non-technical user."

### Proposed QA Approach

#### Level 1: Static Analysis

**Checklist for slash commands:**
- [ ] `allowed-tools` properly scoped (not overly permissive)
- [ ] Instructions clear and unambiguous
- [ ] Error scenarios documented with guidance
- [ ] Success criteria observable and verifiable

**Example checklist item**:
```markdown
# Bad: Overly permissive
allowed-tools: Bash(*)

# Good: Scoped to langstar
allowed-tools: Bash(langstar prompt:*), Read(*), Write(*)
```

#### Level 2: Functional Testing

**Golden path tests** (manual, documented):
```markdown
## Test Case: Prompt Workshop Basic Flow

**Setup**: Existing prompt `test-prompt` in workspace

**Steps**:
1. User: "Help me workshop test-prompt"
2. Claude: Should fetch prompt details
3. User: "Make it more concise"
4. Claude: Should suggest specific changes
5. User: "Looks good, save it"
6. Claude: Should push with commit message

**Pass criteria**:
- [ ] Prompt retrieved correctly
- [ ] Suggestions are relevant
- [ ] New version appears in LangSmith
```

#### Level 3: Regression Testing

**CLI output consistency tests**:
- When CLI commands change, verify plugin commands still work
- Document expected output formats
- Alert when formats change

#### Level 4: User Acceptance

**SME validation protocol**:
1. SME attempts workflow without coaching
2. Note friction points
3. Iterate on instructions
4. Repeat until smooth

### QA Deliverable: Test Plan Document

Create `docs/testing/claude-code-plugin-qa-plan.md`:
- Golden path test cases for each command
- Expected CLI outputs
- Common failure modes and how Claude should handle them
- SME validation checklist

---

## 4. Sub-Agent Strategy (Revised)

### v1 Conception (Too Ambitious)

> "Agents for complex workflows: prompt-optimizer, dataset-curator, eval-designer"

### v2 Conception (Context Management)

> "Sub-agents are good for doing things like making a request with a lighter model, processing it to remove the noise, and removing the noise, so the context window of the parent agent can stay clean."

### Example: Prompt Search Sub-Agent

**Problem**: `langstar prompt list` returns many prompts, cluttering context

**Solution**: Sub-agent filters and summarizes

```markdown
# agents/prompt-finder/README.md

## Purpose
Search prompts and return concise summary to parent agent.

## Why Sub-Agent
- Full prompt list can be 50+ items
- Parent agent only needs top 3-5 matches
- Use lighter model (haiku) for filtering

## Input
User's natural language description of desired prompt

## Output (to parent)
```json
{
  "matches": [
    {"handle": "prompt-1", "relevance": "high", "reason": "..."},
    {"handle": "prompt-2", "relevance": "medium", "reason": "..."}
  ],
  "total_searched": 47
}
```

## Implementation
1. Run `langstar prompt list --output json`
2. Filter by name/description matching user intent
3. Return summarized results (not full objects)
```

### Sub-Agent Use Cases for Prompt Management

| Sub-Agent | Purpose | Model | Context Savings |
|-----------|---------|-------|-----------------|
| `prompt-finder` | Search and summarize prompts | haiku | ~80% |
| `schema-analyzer` | Parse and explain JSON schemas | haiku | ~60% |
| `version-differ` | Compare prompt versions | haiku | ~70% |

### When NOT to Use Sub-Agents

- Simple CRUD operations (just use CLI directly)
- When full context is needed for decision-making
- When user needs to see all details

---

## 5. Deferred Scope

### Items Removed from v1 MVP

| Item | v1 Location | Why Deferred |
|------|-------------|--------------|
| `/langstar-find-runs` | MVP command | Requires project ID discovery; depends on ls-prompt-ux completion |
| `/langstar-queue-manage` | MVP command | Secondary workflow; wait until prompts excellent |
| `/langstar-models` | MVP command | Admin utility, not core SME workflow |
| `/langstar-dataset-create` | MVP command | Complex (requires runs understanding); v2+ |
| Eval management | Phase 3 | CLI not ready (`langstar eval *` incomplete per feedback) |

### Expansion Criteria

Only expand to new workflows when:
1. Prompt management is rated "excellent" by SME testers
2. CLI UX patterns established and documented
3. QA methodology proven effective
4. Sub-agent patterns validated

---

## 6. Revised Plugin Design

### Minimal Structure (v2)

```
langstar/
├── plugins/
│   └── manage-langstar/
│       ├── .claude-plugin/
│       │   └── plugin.json
│       ├── commands/
│       │   └── prompt-workshop.md    # Single command MVP
│       ├── agents/
│       │   └── prompt-finder/        # Context management sub-agent
│       │       └── README.md
│       ├── skills/
│       │   └── prompt-engineering/   # Prompt engineering knowledge
│       │       └── SKILL.md
│       └── README.md
```

### `/langstar-prompt-workshop` Command (MVP)

```markdown
---
allowed-tools: Bash(langstar prompt:*), Read(*), Write(*)
description: Interactive prompt engineering workshop - get, edit, and save prompts
---

## Context

Available prompts in workspace:
!`langstar prompt list --output table --limit 10`

## Your Task

Help the user with prompt engineering. This includes:

1. **Discovering prompts**: Use `langstar prompt list` or `langstar prompt search`
2. **Viewing details**: Use `langstar prompt get <handle>` for metadata,
   `langstar prompt cat <handle>` for template text
3. **Understanding schemas**: For structured prompts, explain the JSON schema
4. **Suggesting improvements**: Apply prompt engineering best practices
5. **Saving changes**: Use `langstar prompt push` with descriptive commit message

## Important Notes

- Always confirm with user before pushing changes
- Explain your prompt engineering reasoning
- If CLI command fails, explain the error and suggest fixes
- Ask clarifying questions when user intent is unclear

## Common Workflows

### View a prompt
```bash
langstar prompt get <handle>  # Metadata and stats
langstar prompt cat <handle>  # Template text for editing
```

### Push changes
```bash
langstar prompt push --content "..." --description "..." <handle>
```
```

### Composability Note

> "Slash commands can call slash commands, skills can call skills and slash commands."

The `prompt-engineering` skill provides reusable knowledge that:
- `/langstar-prompt-workshop` command loads for context
- Future commands can also reference
- Can be updated independently of commands

---

## 7. Revised Recommendation

### Decision: **Conditional GO**

**Conditions**:
1. Co-evolve with ls-prompt-ux milestone (#16)
2. Implement QA methodology before expanding scope
3. Single command MVP, not 5 commands

### Revised Phases

| Phase | Focus | Deliverable |
|-------|-------|-------------|
| **0.0** | Scout (this document) | Feasibility analysis ✅ |
| **0.1** | CLI Validation | Test current prompt CLI, feed back to #679 |
| **0.2** | Minimal Plugin | Single `/langstar-prompt-workshop` command |
| **0.3** | QA Framework | Test plan document, golden path tests |
| **0.4** | Iteration | Improve CLI + plugin based on testing |
| **1.0** | Excellence Gate | SME validation, decision to expand |

### Success Metrics (Revised)

**Prompt Management Excellence**:
- [ ] SME completes prompt workshop workflow without coaching
- [ ] Claude correctly parses all prompt CLI outputs
- [ ] Error recovery is smooth and guided
- [ ] Time-to-task < 2 minutes for common operations

**Before expanding scope**:
- [ ] QA methodology documented and proven
- [ ] CLI UX patterns documented as guidelines
- [ ] At least 3 SME validation sessions completed

---

## 8. Open Questions

### For CLI Design (#679)

1. Which command structure option will be chosen?
2. What output format is most Claude-friendly (JSON vs table vs text)?
3. How will structured output indicators appear in list output?

### For QA Methodology

1. How do we automate golden path tests?
2. Should we create a "Claude Code plugin test harness"?
3. How do we measure "useful to SME" quantitatively?

### For Plugin Development

1. Should the plugin depend on a minimum langstar CLI version?
2. How do we handle CLI version incompatibilities gracefully?
3. What's the plugin distribution strategy (this repo vs separate)?

---

## 9. Next Steps

If this revised approach is approved:

1. **Immediately**: Review and align with #679 design discussion
2. **Phase 0.1**: Create test cases for current prompt CLI
3. **Phase 0.2**: Implement minimal plugin structure
4. **Ongoing**: Document learnings for QA methodology

**Key principle**: One excellent experience > many mediocre experiences.

---

## References

- [ls-prompt-ux milestone #16](https://github.com/codekiln/langstar/milestone/16)
- [Issue #679 - Design prompt command structure](https://github.com/codekiln/langstar/issues/679)
- [Issue #668 - ls-prompt-ux parent issue](https://github.com/codekiln/langstar/issues/668)
- [Claude Code plugin docs](https://github.com/anthropics/claude-code)
- [Phase 0.0 scouting process](../../dev/feature-development-process.md#phase-00-pre-epic-scouting-optional)
