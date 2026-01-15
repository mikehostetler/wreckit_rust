# Research: Implement data-driven immutable core

**Date**: 2026-01-15
**Item**: 002-implement-data-driven-immutable-core

## Research Question
The core logic needs to be predictable, testable, and reliable through immutable data patterns.

**Motivation:** Immutable and idempotent operations make the system more predictable, easier to test, and safer to retry or recover from failures.

**Success criteria:**
- Core uses immutable data operations
- Operations are idempotent

**Technical constraints:**
- Data-driven architecture
- Focus on immutability and idempotency

## Summary

The current Rust implementation has a **mixed approach to immutability**. While the domain transition logic (`apply_state_transition` in `/Users/mhostetler/Source/wreckit_rust/src/domain/transitions.rs:67-90`) correctly follows immutable patterns by cloning and returning new items, there are **mutation hotspots** that violate the immutable core principle:

1. **`Prd::mark_story_done`** (`src/schemas/prd.rs:120-128`) - Mutates story status in-place via `&mut self`
2. **`Item::touch`** (`src/schemas/item.rs:170-172`) - Mutates `updated_at` field in-place via `&mut self`
3. **State transitions in tests** (`src/domain/transitions.rs:86-87, 103`) - Uses `mut` to clone and modify fields

The TypeScript reference implementation (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/transitions.ts:23-45`) demonstrates pure immutable patterns using spread operators (`...item`) that never mutate input, with property-based tests using `Object.freeze()` to enforce immutability detection.

To achieve the success criteria, we need to:
1. **Replace all mutable methods with immutable "with_" builders** that return new instances
2. **Add Builder pattern support** for complex multi-field updates
3. **Remove `&mut self` methods** from core schemas (Item, Prd, Story)
4. **Ensure all operations are idempotent** - calling them multiple times with same input produces same result
5. **Add property-based tests** with frozen data to detect mutations

## Current State Analysis

### Existing Implementation

The Rust codebase currently has:

**Immutable patterns (good):**
- `src/domain/transitions.rs:67-90` - `apply_state_transition()` takes `&Item`, validates, and returns `TransitionResult` with new `Item` (never mutates input)
- `src/domain/validation.rs` - All validation functions are pure (take references, return `ValidationResult`)
- `src/fs/json.rs:53-73` - `write_json()` uses atomic writes (temp file + rename) to prevent corruption
- All state queries (`is_done()`, `is_pending()`, etc.) are pure read-only methods

**Mutable patterns (needs fixing):**
- `src/schemas/prd.rs:120-128` - `mark_story_done(&mut self, story_id)` mutates story.status directly
- `src/schemas/item.rs:170-172` - `touch(&mut self)` mutates `updated_at` timestamp
- Tests in `src/domain/transitions.rs:86` use `let mut next_item = item.clone()` pattern which works but isn't idiomatic for immutable data

**Missing:**
- No Builder pattern for constructing updated Items/Prds/Stories
- No property-based tests (TypeScript has fast-check tests in `src/__tests__/domain.property.test.ts`)
- No immutable update helpers (e.g., `with_story_status()`, `with_state()`, `with_updated_at()`)

### Key Files

- **`src/schemas/item.rs:62-173`** - Item struct with mutable `touch()` method (line 170)
- **`src/schemas/prd.rs:67-129`** - Prd struct with mutable `mark_story_done()` method (line 120)
- **`src/domain/transitions.rs:67-90`** - Immutable transition implementation (good reference)
- **`src/domain/transitions.rs:85-89`** - Uses `mut` with clone pattern (could be more idiomatic)
- **`src/fs/json.rs:53-73`** - Atomic writes (idempotent file operations)
- **`src/git/operations.rs:149-172`** - Git operations that are inherently idempotent (check if exists, then create)

## Technical Considerations

### Dependencies

**External dependencies needed:**
- Consider adding `proptest` for property-based testing (TypeScript uses `fast-check`)
- Current `serde` provides immutable serialization/deserialization
- `tokio` for async operations (already in use)

**Internal modules to integrate with:**
- `domain/transitions` - Already follows immutable patterns (good reference)
- `domain/validation` - Pure functions (good reference)
- `workflow` (currently empty) - Will be the primary consumer of immutable operations
- `agent/runner` - Should work with immutable data structures
- `git/operations` - Already idempotent (check-before-create pattern)

### Patterns to Follow

**From TypeScript reference:**

1. **Pure transition functions** (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/transitions.ts:23-45`):
   ```typescript
   const nextItem: Item = {
     ...item,  // Spread copy (immutable)
     state: nextState,
     updated_at: new Date().toISOString(),
   };
   return { nextItem };
   ```

2. **Immutable TUI updates** (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/dashboard.ts:62-67`):
   ```typescript
   export function updateTuiState(state: TuiState, update: Partial<TuiState>): TuiState {
     return { ...state, ...update };  // Always returns new state
   }
   ```

3. **Property-based testing with freeze** (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/__tests__/domain.property.test.ts:163-196`):
   ```typescript
   const frozenItem = Object.freeze({ ...item });
   const originalJson = JSON.stringify(frozenItem);
   // ... call function with frozen item
   // Assert: JSON hasn't changed
   ```

**Rust idioms to adopt:**

1. **Builder pattern with `with_*` methods:**
   ```rust
   impl Item {
       pub fn with_state(self, state: WorkflowState) -> Self {
           Item { state, ..self }
       }
       pub fn with_updated_at(self) -> Self {
           Item { updated_at: chrono::Utc::now().to_rfc3339(), ..self }
       }
   }
   ```

2. **Update helpers for collections:**
   ```rust
   impl Prd {
       pub fn with_story_status(self, story_id: &str, status: StoryStatus) -> Prd {
           Prd {
               user_stories: self.user_stories.iter()
                   .map(|s| if s.id == story_id {
                       Story { status, ..s.clone() }
                   } else {
                       s.clone()
                   })
                   .collect(),
               ..self
           }
       }
   }
   ```

3. **Property-based testing with `proptest`:**
   ```rust
   proptest! {
       #[test]
       fn test_transition_never_mutates(item in any_item()) {
           let original = item.clone();
           let ctx = make_context();
           let _ = apply_state_transition(&item, &ctx);
           prop_assert_eq!(item, original);  // Item unchanged
       }
   }
   ```

**Idempotency patterns:**

Current code already has some idempotent operations:
- `git/operations.rs:149-172` - `ensure_branch()` checks if exists before creating
- `fs/json.rs:53-73` - Atomic writes with temp file + rename

Need to ensure:
- All state transitions can be called multiple times safely
- File writes are atomic (already done)
- Git operations check-before-act (already done)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance degradation from excessive cloning | Medium | Use `Arc` for shared data if profiling shows issues; Rust's move semantics often make copying cheap |
| Complex update chains become verbose | Low | Implement Builder pattern or `with_*` fluent API; consider struct update syntax sugar |
| Breaking existing code that uses `&mut self` | Medium | Add deprecation warnings, keep old methods temporarily, migrate incrementally |
| Property tests are slow to run | Low | Keep property test suite small; run unit tests in CI, property tests locally/nightly |
| TypeScript mutation patterns leak to Rust | Low | Reference TypeScript for architecture, but use Rust idioms (Builder pattern, not spread) |

## Recommended Approach

### Phase 1: Immutable update methods for schemas

**Add `with_*` methods to `Item`** (`src/schemas/item.rs`):
```rust
impl Item {
    pub fn with_state(mut self, state: WorkflowState) -> Self {
        self.state = state;
        self.touch()
    }

    pub fn with_branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self.touch()
    }

    pub fn with_pr(mut self, pr_url: Option<String>, pr_number: Option<u32>) -> Self {
        self.pr_url = pr_url;
        self.pr_number = pr_number;
        self.touch()
    }

    pub fn with_error(mut self, error: Option<String>) -> Self {
        self.last_error = error;
        self.touch()
    }

    // Keep touch() private - internal helper
    fn touch(&mut self) -> Self {
        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.clone()
    }
}
```

**Add `with_*` methods to `Prd` and `Story`** (`src/schemas/prd.rs`):
```rust
impl Prd {
    pub fn with_story_status(&self, story_id: &str, status: StoryStatus) -> Prd {
        Prd {
            user_stories: self.user_stories.iter()
                .map(|s| if s.id == story_id {
                    Story { status, ..s.clone() }
                } else {
                    s.clone()
                })
                .collect(),
            ..self.clone()
        }
    }

    pub fn with_story(&self, story: Story) -> Prd {
        let mut stories: Vec<_> = self.user_stories.iter()
            .filter(|s| s.id != story.id)
            .cloned()
            .collect();
        stories.push(story);
        Prd { user_stories: stories, ..self.clone() }
    }
}

impl Story {
    pub fn with_status(mut self, status: StoryStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = notes;
        self
    }
}
```

**Mark old methods as deprecated:**
```rust
#[deprecated(since = "0.2.0", note = "Use with_story_status() for immutable updates")]
pub fn mark_story_done(&mut self, story_id: &str) -> bool { ... }
```

### Phase 2: Update domain transitions to use new API

**Refactor `apply_state_transition`** (`src/domain/transitions.rs:85-89`):
```rust
// Before (current):
let mut next_item = item.clone();
next_item.state = next_state;
next_item.updated_at = chrono::Utc::now().to_rfc3339();

// After:
let next_item = item.clone()
    .with_state(next_state);
```

### Phase 3: Add property-based tests

**Add `proptest` dependency** to `Cargo.toml`:
```toml
[dev-dependencies]
proptest = "1"
```

**Create `src/domain/property_tests.rs`:**
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_apply_transition_never_mutates(item in any_item_strategy()) {
            let original = item.clone();
            let ctx = make_context_for_state(item.state);
            let _ = apply_state_transition(&item, &ctx);
            prop_assert_eq!(item, original);
        }

        #[test]
        fn test_with_story_status_is_idempotent(prd in any_prd_strategy(), story_id in ".*", status in any_story_status()) {
            let updated_once = prd.with_story_status(&story_id, status);
            let updated_twice = updated_once.with_story_status(&story_id, status);
            prop_assert_eq!(updated_once, updated_twice);
        }
    }
}
```

### Phase 4: Update workflow layer (when implemented)

When `src/workflow/` is implemented, ensure:
- All phase runners work with immutable data
- State updates use `with_*` methods
- File writes happen only after all validations pass

### Phase 5: Documentation

Add documentation to explain the immutable core:
- Update module-level docs to emphasize immutability
- Add examples showing the fluent API
- Document the idempotency guarantees

## Open Questions

1. **Should we keep `&mut self` methods temporarily for compatibility?**
   - **Recommendation:** Yes, mark as `#[deprecated]` and remove in next major version

2. **Should we use derive macros for Builder pattern?**
   - **Recommendation:** No, hand-written `with_*` methods give more control (e.g., `with_state` auto-updates `updated_at`)

3. **Should we add `derive(PartialEq)` to all schemas for easier testing?**
   - **Recommendation:** Yes, already present on `WorkflowState`, should add to `StoryStatus` and ensure all structs have it

4. **Should we implement `Display` for all enums?**
   - **Recommendation:** Already done for `WorkflowState` (good pattern), ensure consistency across enums

5. **How should we handle collection updates (e.g., adding a story to PRD)?**
   - **Recommendation:** Implement `with_story()` and `without_story()` methods that return new Prd

6. **Should we use `Arc` for sharing large immutable structures?**
   - **Recommendation:** Not needed initially; clone is cheap for small structs. Profile first if performance issues arise.
