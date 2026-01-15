# Port Reckett CLI tool from TypeScript Implementation Plan

## Overview

This plan ports the "wreckit" CLI tool from TypeScript to Rust, maintaining feature parity with the original implementation. The tool transforms ideas into automated PRs through an autonomous agent loop, implementing a deterministic state machine workflow (idea → researched → planned → implementing → in_pr → done).

The Rust implementation will use idiomatic patterns (serde for serialization, clap for CLI, tokio for async) while maintaining the same directory structure, file formats, and workflow semantics as the TypeScript version.

## Current State Analysis

### Existing Implementation

The TypeScript implementation at `/Users/mhostetler/Source/MikeHostetler/wreckit` provides the complete reference with:
- 12+ CLI commands (ideas, status, list, show, research, plan, implement, pr, complete, run, next, doctor, init)
- State machine with 6 states and validation rules for each transition
- Agent execution via process spawning (claude CLI) or SDK integration
- Template-based prompt system with variable substitution
- Git/GitHub operations for branch management and PR creation
- MCP tool callbacks for PRD saving and story status updates

### Key Discoveries

- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/schemas.ts:1-145` - All data models use Zod for runtime validation; we'll use serde derives for compile-time safety
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/states.ts:12-19` - WORKFLOW_STATES array defines the canonical state ordering
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/validation.ts:85-112` - Validation is target-state-specific via switch statement
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/workflow/itemWorkflow.ts:144-244` - Phase runners follow consistent pattern: load item, check state, run agent, validate artifacts, transition state
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/agent/runner.ts:175-274` - Process agent uses spawn with stdin/stdout streaming
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/prompts.ts:55-95` - Template rendering handles `{{variable}}`, `{{#if}}`, and `{{#ifnot}}` syntax

### Target Rust Project Structure

```
wreckit_rust/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── cli/
│   │   ├── mod.rs           # CLI module
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── init.rs
│   │   │   ├── status.rs
│   │   │   ├── list.rs
│   │   │   ├── show.rs
│   │   │   ├── research.rs
│   │   │   ├── plan.rs
│   │   │   ├── implement.rs
│   │   │   ├── pr.rs
│   │   │   ├── complete.rs
│   │   │   ├── run.rs
│   │   │   ├── next.rs
│   │   │   ├── doctor.rs
│   │   │   └── ideas.rs
│   ├── domain/
│   │   ├── mod.rs           # Domain re-exports
│   │   ├── states.rs        # Workflow states
│   │   ├── transitions.rs   # State transitions
│   │   └── validation.rs    # Transition validation
│   ├── schemas/
│   │   ├── mod.rs           # Schema types
│   │   ├── item.rs
│   │   ├── config.rs
│   │   ├── prd.rs
│   │   └── index.rs
│   ├── workflow/
│   │   ├── mod.rs
│   │   ├── phases.rs        # Phase runners
│   │   └── orchestrator.rs  # Run/next orchestration
│   ├── agent/
│   │   ├── mod.rs
│   │   └── runner.rs        # Process agent
│   ├── git/
│   │   ├── mod.rs
│   │   └── operations.rs    # Git/gh commands
│   ├── fs/
│   │   ├── mod.rs
│   │   ├── paths.rs         # Path utilities
│   │   ├── json.rs          # JSON read/write
│   │   └── atomic.rs        # Atomic writes
│   ├── prompts/
│   │   ├── mod.rs
│   │   └── template.rs      # Template rendering
│   ├── config/
│   │   ├── mod.rs
│   │   └── loader.rs        # Config loading
│   └── errors.rs            # Error types
├── prompts/                  # Bundled prompt templates
│   ├── research.md
│   ├── plan.md
│   ├── implement.md
│   ├── pr.md
│   └── ideas.md
└── tests/
    ├── domain_test.rs
    ├── schemas_test.rs
    └── integration/
```

## Desired End State

A working Rust binary `wreckit` that:
1. Implements all 12+ CLI commands with identical semantics to the TypeScript version
2. Reads/writes the same `.wreckit/` directory structure and JSON formats
3. Executes agents via process spawning (claude CLI)
4. Creates branches, commits, and PRs via git/gh commands
5. Passes equivalent tests to the TypeScript test suite

### Verification

```bash
# Build succeeds
cargo build --release

# Tests pass
cargo test

# CLI works
./target/release/wreckit --help
./target/release/wreckit init
./target/release/wreckit status
./target/release/wreckit run <item-id>
```

## What We're NOT Doing

1. **TUI (ratatui)**: Defer to future work; use simple stdout progress for MVP
2. **Native Claude SDK**: Use process mode only (spawn `claude` CLI); SDK integration can be added later via FFI
3. **MCP Server**: Use completion signal detection; full bidirectional MCP communication deferred
4. **Multiple agent backends**: Only implement process agent; discriminated union pattern can be added later
5. **Windows support**: Focus on Unix-like systems initially
6. **Ideas interview mode**: Parse ideas from file/stdin only; interactive interview deferred
7. **Property-based tests**: Use standard unit tests; proptest can be added later

---

## Phase 1: Project Setup and Core Schemas

### Overview
Set up the Rust project structure with Cargo, define all data schemas using serde, and implement basic file system utilities.

### Changes Required:

#### 1. Cargo.toml
**File**: `Cargo.toml`
**Changes**: Create project manifest with dependencies

```toml
[package]
name = "wreckit"
version = "0.1.0"
edition = "2021"
description = "A CLI tool for turning ideas into automated PRs through an autonomous agent loop"
license = "MIT"

[[bin]]
name = "wreckit"
path = "src/main.rs"

[lib]
name = "wreckit"
path = "src/lib.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
regex = "1"

[dev-dependencies]
tempfile = "3"
```

#### 2. Schema Types
**File**: `src/schemas/mod.rs`
**Changes**: Define all schema types matching TypeScript

Key types to implement:
- `WorkflowState` enum: `Idea`, `Researched`, `Planned`, `Implementing`, `InPr`, `Done`
- `Item` struct with all fields from TypeScript schema
- `Config` struct with agent configuration
- `Prd` and `Story` structs
- `Index` and `IndexItem` structs
- `PriorityHint` enum

#### 3. Error Types
**File**: `src/errors.rs`
**Changes**: Define custom error types with codes

```rust
#[derive(Debug, thiserror::Error)]
pub enum WreckitError {
    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),
}
```

#### 4. File System Utilities
**File**: `src/fs/paths.rs`
**Changes**: Implement path resolution functions

Functions to implement:
- `find_repo_root(start_cwd: &Path) -> Result<PathBuf>`
- `get_wreckit_dir(root: &Path) -> PathBuf`
- `get_config_path(root: &Path) -> PathBuf`
- `get_items_dir(root: &Path) -> PathBuf`
- `get_item_dir(root: &Path, id: &str) -> PathBuf`
- `get_research_path(root: &Path, id: &str) -> PathBuf`
- `get_plan_path(root: &Path, id: &str) -> PathBuf`
- `get_prd_path(root: &Path, id: &str) -> PathBuf`

**File**: `src/fs/json.rs`
**Changes**: Implement JSON file operations

Functions to implement:
- `read_json<T: DeserializeOwned>(path: &Path) -> Result<T>`
- `write_json<T: Serialize>(path: &Path, data: &T) -> Result<()>`
- `read_item(item_dir: &Path) -> Result<Item>`
- `write_item(item_dir: &Path, item: &Item) -> Result<()>`
- `read_prd(item_dir: &Path) -> Result<Prd>`
- `write_prd(item_dir: &Path, prd: &Prd) -> Result<()>`

### Success Criteria:

#### Automated Verification:
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes all schema serialization tests
- [ ] `cargo clippy` passes with no warnings

#### Manual Verification:
- [ ] JSON round-trip: serialize Item → JSON → deserialize matches original
- [ ] Path functions return correct paths relative to root

---

## Phase 2: Domain Logic - State Machine

### Overview
Implement the workflow state machine with pure transition functions and validation rules.

### Changes Required:

#### 1. State Definitions
**File**: `src/domain/states.rs`
**Changes**: Implement state machine logic

```rust
pub const WORKFLOW_STATES: &[WorkflowState] = &[
    WorkflowState::Idea,
    WorkflowState::Researched,
    WorkflowState::Planned,
    WorkflowState::Implementing,
    WorkflowState::InPr,
    WorkflowState::Done,
];

pub fn get_state_index(state: WorkflowState) -> usize;
pub fn get_next_state(current: WorkflowState) -> Option<WorkflowState>;
pub fn get_allowed_next_states(current: WorkflowState) -> Vec<WorkflowState>;
pub fn is_terminal_state(state: WorkflowState) -> bool;
```

#### 2. Validation Rules
**File**: `src/domain/validation.rs`
**Changes**: Implement transition validation

```rust
pub struct ValidationContext {
    pub has_research_md: bool,
    pub has_plan_md: bool,
    pub prd: Option<Prd>,
    pub has_pr: bool,
    pub pr_merged: bool,
}

pub struct ValidationResult {
    pub valid: bool,
    pub reason: Option<String>,
}

pub fn can_enter_researched(ctx: &ValidationContext) -> ValidationResult;
pub fn can_enter_planned(ctx: &ValidationContext) -> ValidationResult;
pub fn can_enter_implementing(ctx: &ValidationContext) -> ValidationResult;
pub fn can_enter_in_pr(ctx: &ValidationContext) -> ValidationResult;
pub fn can_enter_done(ctx: &ValidationContext) -> ValidationResult;
pub fn validate_transition(current: WorkflowState, target: WorkflowState, ctx: &ValidationContext) -> ValidationResult;
```

#### 3. State Transitions
**File**: `src/domain/transitions.rs`
**Changes**: Implement pure transition function

```rust
pub enum TransitionResult {
    Success { next_item: Item },
    Error { error: String },
}

/// Pure function that applies a state transition to an item.
/// Never mutates the input item.
pub fn apply_state_transition(item: &Item, ctx: &ValidationContext) -> TransitionResult;
```

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test domain` passes all state machine tests
- [ ] All edge cases from TypeScript tests are covered

#### Manual Verification:
- [ ] State transitions match TypeScript behavior exactly
- [ ] Invalid transitions (skip states, backward) are rejected
- [ ] Validation rules match documented requirements

---

## Phase 3: Configuration and Prompt Templates

### Overview
Implement configuration loading with defaults and prompt template rendering.

### Changes Required:

#### 1. Configuration Loading
**File**: `src/config/loader.rs`
**Changes**: Implement config loading with defaults

```rust
pub struct ConfigResolved {
    pub schema_version: u32,
    pub base_branch: String,
    pub branch_prefix: String,
    pub merge_mode: MergeMode,
    pub agent: AgentConfig,
    pub max_iterations: u32,
    pub timeout_seconds: u32,
}

pub const DEFAULT_CONFIG: ConfigResolved = ConfigResolved {
    schema_version: 1,
    base_branch: "main",
    branch_prefix: "wreckit/",
    merge_mode: MergeMode::Pr,
    agent: AgentConfig {
        mode: AgentMode::Process,
        command: "claude",
        args: vec!["--dangerously-skip-permissions", "--print"],
        completion_signal: "<promise>COMPLETE</promise>",
    },
    max_iterations: 100,
    timeout_seconds: 3600,
};

pub fn load_config(root: &Path) -> Result<ConfigResolved>;
pub fn merge_with_defaults(partial: &Config) -> ConfigResolved;
```

#### 2. Prompt Template System
**File**: `src/prompts/template.rs`
**Changes**: Implement template loading and rendering

```rust
pub struct PromptVariables {
    pub id: String,
    pub title: String,
    pub section: String,
    pub overview: String,
    pub item_path: String,
    pub branch_name: String,
    pub base_branch: String,
    pub completion_signal: String,
    pub sdk_mode: bool,
    pub research: Option<String>,
    pub plan: Option<String>,
    pub prd: Option<String>,
    pub progress: Option<String>,
}

pub fn load_prompt_template(root: &Path, name: &str) -> Result<String>;
pub fn render_prompt(template: &str, variables: &PromptVariables) -> String;
```

Template rendering must handle:
- `{{variable}}` - Simple substitution
- `{{#if variable}}...{{/if}}` - Conditional content
- `{{#ifnot variable}}...{{/ifnot}}` - Inverse conditional

#### 3. Bundled Prompts
**Directory**: `prompts/`
**Changes**: Copy prompt templates from TypeScript

Copy these files:
- `prompts/research.md`
- `prompts/plan.md`
- `prompts/implement.md`
- `prompts/pr.md`
- `prompts/ideas.md`

Use `include_str!` macro to bundle at compile time.

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test config` passes
- [ ] `cargo test prompts` passes template rendering tests
- [ ] Template conditionals work correctly

#### Manual Verification:
- [ ] Default config matches TypeScript defaults
- [ ] Config merging preserves partial overrides
- [ ] Rendered prompts match TypeScript output for same inputs

---

## Phase 4: Agent Execution

### Overview
Implement process-based agent execution with stdin/stdout streaming and completion signal detection.

### Changes Required:

#### 1. Agent Runner
**File**: `src/agent/runner.rs`
**Changes**: Implement process agent

```rust
pub struct AgentConfig {
    pub command: String,
    pub args: Vec<String>,
    pub completion_signal: String,
    pub timeout_seconds: u32,
}

pub struct AgentResult {
    pub success: bool,
    pub output: String,
    pub timed_out: bool,
    pub exit_code: Option<i32>,
    pub completion_detected: bool,
}

pub struct RunAgentOptions {
    pub config: AgentConfig,
    pub cwd: PathBuf,
    pub prompt: String,
    pub dry_run: bool,
    pub on_stdout_chunk: Option<Box<dyn Fn(&str)>>,
    pub on_stderr_chunk: Option<Box<dyn Fn(&str)>>,
}

pub async fn run_agent(options: RunAgentOptions) -> Result<AgentResult>;
```

Implementation pattern (from TypeScript):
1. Spawn process with `command` and `args`
2. Write prompt to stdin, then close
3. Read stdout/stderr, buffering output
4. Check for completion signal in combined output
5. Apply timeout (kill SIGTERM, then SIGKILL after 5s)
6. Return result with exit code and completion status

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test agent` passes
- [ ] Dry-run mode returns mock result without spawning
- [ ] Timeout handling works correctly

#### Manual Verification:
- [ ] Agent spawns and receives prompt via stdin
- [ ] Output streaming works (stdout/stderr callbacks)
- [ ] Completion signal detection works

---

## Phase 5: Git Operations

### Overview
Implement git and GitHub CLI operations for branch management and PR creation.

### Changes Required:

#### 1. Git Operations
**File**: `src/git/operations.rs`
**Changes**: Implement git command wrappers

```rust
pub struct GitOptions {
    pub cwd: PathBuf,
    pub dry_run: bool,
}

pub struct BranchResult {
    pub branch_name: String,
    pub created: bool,
}

pub struct PrResult {
    pub url: String,
    pub number: u32,
    pub created: bool,
}

// Basic operations
pub async fn run_git_command(args: &[&str], options: &GitOptions) -> Result<String>;
pub async fn run_gh_command(args: &[&str], options: &GitOptions) -> Result<String>;
pub async fn is_git_repo(cwd: &Path) -> bool;
pub async fn get_current_branch(options: &GitOptions) -> Result<String>;

// Branch operations
pub async fn branch_exists(branch_name: &str, options: &GitOptions) -> bool;
pub async fn ensure_branch(base_branch: &str, branch_prefix: &str, item_slug: &str, options: &GitOptions) -> Result<BranchResult>;
pub async fn has_uncommitted_changes(options: &GitOptions) -> bool;
pub async fn commit_all(message: &str, options: &GitOptions) -> Result<()>;
pub async fn push_branch(branch_name: &str, options: &GitOptions) -> Result<()>;

// PR operations
pub async fn get_pr_by_branch(branch_name: &str, options: &GitOptions) -> Option<PrResult>;
pub async fn create_or_update_pr(base_branch: &str, head_branch: &str, title: &str, body: &str, options: &GitOptions) -> Result<PrResult>;
pub async fn is_pr_merged(pr_number: u32, options: &GitOptions) -> bool;

// Direct merge (YOLO mode)
pub async fn merge_and_push_to_base(base_branch: &str, feature_branch: &str, commit_message: &str, options: &GitOptions) -> Result<()>;

// Preflight checks
pub async fn check_git_preflight(options: &GitOptions) -> GitPreflightResult;
```

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test git` passes with mocked commands
- [ ] Dry-run mode logs commands without executing

#### Manual Verification:
- [ ] Branch creation/checkout works
- [ ] Commit and push operations work
- [ ] PR creation via `gh` works

---

## Phase 6: Workflow Phase Runners

### Overview
Implement the phase runners (research, plan, implement, pr, complete) that orchestrate agent execution and state transitions.

### Changes Required:

#### 1. Phase Runners
**File**: `src/workflow/phases.rs`
**Changes**: Implement all phase runners

```rust
pub struct WorkflowOptions {
    pub root: PathBuf,
    pub config: ConfigResolved,
    pub force: bool,
    pub dry_run: bool,
    pub on_agent_output: Option<Box<dyn Fn(&str)>>,
}

pub struct PhaseResult {
    pub success: bool,
    pub item: Item,
    pub error: Option<String>,
}

pub async fn run_phase_research(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;
pub async fn run_phase_plan(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;
pub async fn run_phase_implement(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;
pub async fn run_phase_pr(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;
pub async fn run_phase_complete(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;

pub fn get_next_phase(item: &Item) -> Option<Phase>;
```

Each phase runner follows the pattern from TypeScript:
1. Load current item from disk
2. Check if artifacts exist (skip if --force not set)
3. Validate item is in correct state
4. Build prompt variables
5. Run agent with rendered prompt
6. Validate agent created required artifacts
7. Transition item state
8. Save updated item to disk

#### 2. Orchestrator
**File**: `src/workflow/orchestrator.rs`
**Changes**: Implement run/next orchestration

```rust
pub struct OrchestrateResult {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    pub remaining: Vec<String>,
}

pub async fn orchestrate_all(options: &WorkflowOptions) -> Result<OrchestrateResult>;
pub async fn orchestrate_next(options: &WorkflowOptions) -> Result<Option<String>>;
pub async fn run_item(item_id: &str, options: &WorkflowOptions) -> Result<PhaseResult>;
```

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test workflow` passes
- [ ] Phase runners validate state correctly
- [ ] Orchestrator processes items in priority order

#### Manual Verification:
- [ ] Research phase creates research.md
- [ ] Plan phase creates plan.md and prd.json
- [ ] Implement phase iterates through stories
- [ ] PR phase creates/updates PR
- [ ] Complete phase verifies PR is merged

---

## Phase 7: CLI Commands

### Overview
Implement all CLI commands using clap with derive macros.

### Changes Required:

#### 1. CLI Structure
**File**: `src/cli/mod.rs`
**Changes**: Define CLI with clap

```rust
#[derive(Parser)]
#[command(name = "wreckit")]
#[command(about = "A CLI tool for turning ideas into automated PRs through an autonomous agent loop")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub dry_run: bool,

    #[arg(long, global = true)]
    pub cwd: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init { #[arg(long)] force: bool },
    Status { #[arg(long)] json: bool },
    List { #[arg(long)] json: bool, #[arg(long)] state: Option<String> },
    Show { id: String, #[arg(long)] json: bool },
    Research { id: String, #[arg(long)] force: bool },
    Plan { id: String, #[arg(long)] force: bool },
    Implement { id: String, #[arg(long)] force: bool },
    Pr { id: String, #[arg(long)] force: bool },
    Complete { id: String },
    Run { id: String, #[arg(long)] force: bool },
    Next,
    Doctor { #[arg(long)] fix: bool },
    Ideas { #[arg(short, long)] file: Option<PathBuf> },
}
```

#### 2. Command Implementations
**Files**: `src/cli/commands/*.rs`
**Changes**: Implement each command

Commands to implement:
- `init` - Initialize .wreckit/ directory
- `status` - List all items with state
- `list` - List items with filtering
- `show` - Show item details
- `research` - Run research phase
- `plan` - Run plan phase
- `implement` - Run implement phase
- `pr` - Create/update PR
- `complete` - Mark as done
- `run` - Run item through all phases
- `next` - Run next incomplete item
- `doctor` - Validate items and fix issues
- `ideas` - Ingest ideas from file

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test cli` passes
- [ ] All commands parse correctly
- [ ] Help text is generated

#### Manual Verification:
- [ ] `wreckit --help` shows all commands
- [ ] `wreckit init` creates .wreckit/ directory
- [ ] `wreckit status` lists items
- [ ] `wreckit run <id>` executes workflow

---

## Phase 8: Integration and Testing

### Overview
Add integration tests and ensure end-to-end workflow works correctly.

### Changes Required:

#### 1. Unit Tests
**Directory**: `tests/`
**Changes**: Port tests from TypeScript

Test files to create:
- `tests/schemas_test.rs` - JSON serialization/deserialization
- `tests/domain_test.rs` - State machine and validation
- `tests/config_test.rs` - Config loading
- `tests/prompts_test.rs` - Template rendering
- `tests/git_test.rs` - Git operations (mocked)

#### 2. Integration Tests
**Directory**: `tests/integration/`
**Changes**: End-to-end tests

Tests to implement:
- Init creates correct directory structure
- Full workflow: idea → done (with mocked agent)
- Idempotent operations (re-running phases)
- Error recovery

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test` passes all tests
- [ ] Test coverage includes edge cases from TypeScript

#### Manual Verification:
- [ ] End-to-end workflow completes successfully
- [ ] Error messages are clear and actionable

---

## Testing Strategy

### Unit Tests
- All schema types: serialization round-trip
- State machine: all state transitions, edge cases
- Validation: each rule with passing/failing cases
- Config: defaults, merging, file loading
- Prompts: variable substitution, conditionals
- Git: command construction (mocked execution)

### Integration Tests
- Full workflow with mocked agent
- Directory structure creation
- File I/O with temp directories
- Error handling and recovery

### Manual Testing Steps
1. Build release binary: `cargo build --release`
2. Run `wreckit init` in a test repo
3. Create test item manually in `.wreckit/items/`
4. Run `wreckit status` to verify listing
5. Run `wreckit research <id>` with real claude CLI
6. Verify research.md is created
7. Run `wreckit run <id>` through full workflow
8. Verify PR is created on GitHub

## Migration Notes

- Existing `.wreckit/` directories from TypeScript version are fully compatible
- JSON schemas are identical between implementations
- Prompt templates can be reused without modification
- No data migration needed - this is a drop-in replacement

## References
- Research: `/Users/mhostetler/Source/wreckit_rust/.wreckit/items/001-port-reckett-cli-tool-from-typescript/research.md`
- TypeScript schemas: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/schemas.ts:1-145`
- TypeScript domain: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/states.ts:1-58`
- TypeScript workflow: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/workflow/itemWorkflow.ts:1-856`
- TypeScript tests: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/__tests__/domain.test.ts:1-443`
