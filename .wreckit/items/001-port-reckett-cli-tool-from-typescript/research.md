# Research: Port Reckett CLI tool from TypeScript

**Date**: 2026-01-15
**Item**: 001-port-reckett-cli-tool-from-typescript

## Research Question
The Reckett tool exists in TypeScript and needs to be ported to a new implementation while preserving all existing features.

**Motivation:** Enable continued development and potentially improved architecture through a fresh implementation that incorporates lessons learned from the TypeScript version.

**Success criteria:**
- All features from the TypeScript version are evaluated and mapped
- Tool can orchestrate prompts through each loop step
- Tool can push a pull request with end results

**Technical constraints:**
- Reference existing TypeScript implementation in local file system
- Must support the same deterministic workflow loop pattern

## Summary

The TypeScript "wreckit" implementation is a full-featured CLI tool that transforms ideas into automated PRs through an autonomous agent loop. Located at `/Users/mhostetler/Source/MikeHostetler/wreckit`, it implements a deterministic state machine workflow (idea → researched → planned → implementing → in_pr → done) with comprehensive validation, multiple agent backends, a React/Ink TUI, and extensive test coverage.

The tool orchestrates the Claude Agent SDK to execute phases: research, plan, implement, and pr. Each phase uses template-driven prompts with variable substitution, creates specific artifacts (research.md, plan.md, prd.json), and validates transitions before advancing state. The implementation is modular with clear separation between CLI commands, domain logic, workflow execution, agent runners, git operations, and file system handling.

Porting to Rust will require implementing: a state machine with immutable transitions, template-based prompt rendering, agent SDK integration (likely via CLI wrapper initially), git/gh command execution, JSON schema validation, and either a TUI framework (like ratatui) or simple progress output. The TypeScript codebase's test suite provides an excellent specification for expected behavior.

## Current State Analysis

### Existing Implementation

The TypeScript implementation at `/Users/mhostetler/Source/MikeHostetler/wreckit` provides a complete reference:

**Package & Build:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/package.json:1-65` - Defines the npm package "wreckit" v1.0.0
- Runtime: Bun (primary) or Node.js 18+
- Build tool: tsup (ESM output)
- Binary entry: `./dist/index.js`

**Key Dependencies:**
- `@anthropic-ai/claude-agent-sdk@^0.2.7` - Claude Agent SDK integration
- `commander@^14` - CLI framework
- `ink@^6` + `react@^19` - Terminal UI
- `zod@^4` - Runtime schema validation
- `pino@^10` - Structured logging

**Integration Points:**
- Git operations via spawned `git` commands
- GitHub CLI (`gh`) for PR management
- Claude Agent SDK for AI execution
- MCP (Model Context Protocol) servers for tool callbacks

### Key Files

**Entry Point & CLI:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/index.ts:1-527` - Main CLI using Commander.js with 12+ commands
- Commands: ideas, idea, status, list, show, research, plan, implement, pr, complete, run, next, doctor, init

**State Machine & Domain:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/schemas.ts:1-145` - Zod schemas for all data types
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/states.ts:1-58` - Workflow state definitions and progression
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/transitions.ts:1-46` - Pure state transition logic
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/validation.ts:1-112` - Transition validation rules

**Workflow Execution:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/workflow/itemWorkflow.ts:1-856` - Phase runners (research, plan, implement, pr, complete)
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/commands/run.ts:1-143` - Run single item through all phases
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/commands/orchestrator.ts:1-214` - Orchestrate all/next items

**Agent Execution:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/agent/runner.ts:1-434` - Agent dispatch with multiple backends
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/agent/claude-sdk-runner.ts:1-258` - Claude SDK integration
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/agent/mcp/wreckitMcpServer.ts:1-121` - MCP server for tool callbacks

**Prompt System:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts.ts:1-113` - Template loading and rendering
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts/research.md:1-121` - Research phase template
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts/plan.md:1-214` - Planning phase template
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts/implement.md:1-42` - Implementation phase template
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts/pr.md:1-57` - PR generation template

**Git Operations:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/git/index.ts:1-516` - Git/GitHub CLI wrapper functions

**File System & Config:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/fs/paths.ts:1-90` - Path resolution utilities
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/fs/json.ts:1-97` - JSON file operations with schema validation
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/config.ts:1-146` - Configuration loading and merging

**TUI:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/dashboard.ts:1-178` - TUI state management
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/runner.ts:1-234` - TUI runner with Ink
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/agentEvents.ts:1-8` - Agent event types

**Error Handling:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/errors.ts:1-107` - Custom error types with codes

## Technical Considerations

### Data Models (from schemas.ts)

**WorkflowState enum:** `"idea" | "researched" | "planned" | "implementing" | "in_pr" | "done"`

**Item schema:**
```
- schema_version: number
- id: string
- title: string
- section?: string
- state: WorkflowState
- overview: string
- branch: string | null
- pr_url: string | null
- pr_number: number | null
- last_error: string | null
- created_at: string (ISO)
- updated_at: string (ISO)
- problem_statement?: string
- motivation?: string
- success_criteria?: string[]
- technical_constraints?: string[]
- scope_in_scope?: string[]
- scope_out_of_scope?: string[]
- priority_hint?: "low" | "medium" | "high" | "critical"
- urgency_hint?: string
```

**Config schema:**
```
- schema_version: number (default 1)
- base_branch: string (default "main")
- branch_prefix: string (default "wreckit/")
- merge_mode: "pr" | "direct"
- agent: { mode, command, args, completion_signal }
- max_iterations: number (default 100)
- timeout_seconds: number (default 3600)
```

**PRD schema:**
```
- schema_version: number
- id: string
- branch_name: string
- user_stories: Story[]
```

**Story schema:**
```
- id: string (e.g., "US-001")
- title: string
- acceptance_criteria: string[]
- priority: number
- status: "pending" | "done"
- notes: string
```

### Directory Structure (.wreckit/)

```
.wreckit/
├── config.json                    # Global configuration
├── index.json                     # Optional item index cache
├── prompts/                       # Custom prompt templates (optional)
│   ├── research.md
│   ├── plan.md
│   ├── implement.md
│   └── pr.md
└── items/
    └── {id}/
        ├── item.json              # Item metadata and state
        ├── research.md            # Research output
        ├── plan.md                # Implementation plan
        ├── prd.json               # User stories
        └── progress.log           # Implementation progress
```

### Workflow State Machine

```
idea ──research──> researched ──plan──> planned ──implement──> implementing ──pr──> in_pr ──complete──> done
```

**Validation rules per transition:**
- `idea → researched`: research.md must exist
- `researched → planned`: plan.md and valid prd.json must exist
- `planned → implementing`: prd.json must have pending stories
- `implementing → in_pr`: all stories must be "done", PR must be created
- `in_pr → done`: PR must be merged

### Agent Backends

The TypeScript implementation supports multiple agent backends via discriminated union:
1. **process**: Spawn external process (e.g., `claude --print`)
2. **claude_sdk**: Direct Claude Agent SDK integration
3. **amp_sdk**: AMP SDK (alternative)
4. **codex_sdk**: Codex SDK
5. **opencode_sdk**: OpenCode SDK

For Rust port, recommend starting with process mode (spawning `claude` CLI) and optionally adding native SDK integration later.

### Prompt Template Variables

Templates use `{{variable}}` syntax with conditionals:
- `{{#if variable}}...{{/if}}` - Conditional content
- `{{#ifnot variable}}...{{/ifnot}}` - Inverse conditional

Available variables:
- `id`, `title`, `section`, `overview`
- `item_path`, `branch_name`, `base_branch`
- `completion_signal`, `sdk_mode`
- `research`, `plan`, `prd`, `progress` (file contents)

### MCP Tool Callbacks

The agent can invoke MCP tools during execution:
- `save_interview_ideas` - Save ideas from interview
- `save_parsed_ideas` - Save parsed ideas
- `save_prd` - Save PRD during planning
- `update_story_status` - Mark story as done during implementation

### Dependencies

**External dependencies (Rust crates to consider):**
- `clap` - CLI argument parsing (equivalent to Commander.js)
- `serde` + `serde_json` - JSON serialization
- `tokio` - Async runtime
- `ratatui` - TUI framework (equivalent to Ink)
- `tracing` - Logging (equivalent to Pino)
- `thiserror` - Error handling
- `regex` - Template variable substitution
- `git2` or process spawning for git operations

**Internal modules to implement:**
1. `schemas` - Data type definitions with validation
2. `domain` - State machine, validation, idea parsing
3. `workflow` - Phase runners
4. `agent` - Agent execution abstraction
5. `git` - Git/GitHub operations
6. `fs` - File system utilities
7. `config` - Configuration loading
8. `prompts` - Template system
9. `cli` - Command handlers
10. `tui` - Terminal UI (optional)

### Patterns to Follow

**Immutability:** The TypeScript code uses pure functions that never mutate input:
- `applyStateTransition()` returns new Item
- `updateTuiState()` returns new TuiState

**Idempotency:** Operations are designed to be re-runnable:
- Phases skip if artifacts exist (unless --force)
- File writes use atomic operations

**Validation-first:** All state transitions validate preconditions:
- Check source state is correct
- Check required artifacts exist
- Validate against schema

**Dry-run support:** Every command supports `--dry-run` to preview actions

**Error codes:** Custom errors include machine-readable codes for programmatic handling

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Claude SDK has no official Rust binding | High | Use process mode (spawn `claude` CLI) initially; SDK can be added later via FFI or native implementation |
| MCP tool callbacks require bidirectional communication | Medium | For process mode, parse stdout for completion signal; for SDK mode, implement MCP server |
| React/Ink TUI is JavaScript-specific | Medium | Use ratatui for Rust TUI or start with simple stdout progress |
| Zod schema validation is runtime | Low | Use serde with derive macros for compile-time validation |
| Test isolation requires temp directories | Low | Rust's tempdir crate provides equivalent functionality |
| Complex async agent orchestration | Medium | Use tokio with structured concurrency patterns |
| Git operations are spawned processes | Low | Same pattern works in Rust via std::process::Command |

## Recommended Approach

### Phase 1: Core Data Layer
1. Define all schemas using serde with JSON serialization
2. Implement state machine with pure transition functions
3. Implement validation rules for each transition
4. Add file system utilities (atomic writes, path resolution)
5. Add configuration loading with defaults

### Phase 2: Workflow Engine
1. Implement phase runners (research, plan, implement, pr, complete)
2. Add prompt template loading and rendering
3. Implement agent runner using process spawning (claude CLI)
4. Add git operations (branch, commit, push, PR)
5. Implement run/orchestrate commands

### Phase 3: CLI Interface
1. Implement commands using clap
2. Add status/list/show commands
3. Add init/doctor commands
4. Add ideas ingestion
5. Implement global options (--dry-run, --force, --verbose)

### Phase 4: TUI (Optional)
1. Implement dashboard state management
2. Add ratatui-based UI
3. Implement agent event streaming
4. Add keyboard navigation

### Phase 5: Testing
1. Port unit tests for each module
2. Add property-based tests for state transitions
3. Add integration tests with temp directories
4. Add edge case tests (corruption, concurrency)

## Open Questions

1. **Target Rust edition?** Recommend Rust 2021 edition with latest stable compiler.

2. **Async runtime?** Recommend tokio for async operations (git, agent spawning).

3. **TUI priority?** Should TUI be MVP or can we start with simple progress output?

4. **Agent SDK integration depth?** Process mode (spawn `claude`) is sufficient for MVP; native SDK can be added later.

5. **MCP server implementation?** For MVP, rely on completion signal detection; full MCP can be added later.

6. **Windows support?** The TypeScript version assumes Unix-like environment; should Rust version support Windows?

## Feature Mapping Summary

| TypeScript Feature | Rust Implementation Strategy |
|-------------------|------------------------------|
| Commander.js CLI | clap with derive macros |
| Zod schemas | serde + serde_json with derive |
| Ink/React TUI | ratatui or crossterm |
| Pino logging | tracing + tracing-subscriber |
| fs/promises | std::fs or tokio::fs |
| child_process spawn | std::process::Command or tokio::process |
| Claude Agent SDK | Process spawn initially, optional FFI later |
| MCP servers | completion signal detection initially |
| Property tests (fast-check) | proptest or quickcheck |

## Test Coverage Reference

The TypeScript implementation has 32+ test files covering:
- CLI argument parsing
- Configuration loading/validation
- Domain logic (states, transitions, validation)
- File system operations
- Git operations (mocked)
- Ideas parsing
- Workflow phases
- TUI state management
- Edge cases (corruption, concurrency, cwd handling)
- Property-based tests for state transitions

These tests serve as specifications for the Rust implementation.
