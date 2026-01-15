# Implement data-driven immutable core Implementation Plan

## Overview
Transform the core Rust library to use immutable data patterns throughout, ensuring all operations are predictable, testable, and idempotent. This involves replacing mutable methods with builder-style `with_*` methods, updating callers to use the new API, and adding property-based tests to verify immutability guarantees.

## Current State Analysis
The Rust codebase has a **mixed approach to immutability**:

**Good immutable patterns already in place:**
- `src/domain/transitions.rs:67-90` - `apply_state_transition()` is pure (takes `&Item`, returns new `Item`)
- `src/domain/validation.rs` - All validation functions are pure (take references, return results)
- `src/fs/json.rs:53-73` - Atomic writes with temp file + rename pattern
- All query methods (`is_done()`, `is_pending()`) are pure read-only

**Mutable patterns that need fixing:**
- `src/schemas/item.rs:170-172` - `touch(&mut self)` mutates `updated_at` in-place
- `src/schemas/prd.rs:120-128` - `mark_story_done(&mut self, story_id)` mutates story status
- `src/domain/transitions.rs:85-87` - Uses `let mut next_item = item.clone()` then mutates fields
- Test helpers in `src/domain/transitions.rs:103,116` and `src/domain/validation.rs:116,171` mutate fields

**Missing capabilities:**
- No builder-style `with_*` methods for immutable updates
- No property-based tests (TypeScript reference uses fast-check)
- `src/workflow/` directory exists but is empty (will need immutability when implemented)
- `PriorityHint` and `Story` don't derive `PartialEq` (needed for testing)

### Key Discoveries:
- **Compilation error exists**: `src/lib.rs:20` declares `pub mod workflow;` but `src/workflow/mod.rs` doesn't exist - must create empty module file
- **Test utilities mutate data**: Helper functions in tests use direct field mutation (`story.status = *status`)
- **TypeScript reference**: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/transitions.ts:38-42` shows pure spread operator pattern
- **Idempotency already present**: Git operations (`src/git/operations.rs:149-172`) use check-before-create pattern
- **All current usages**: Only test code uses `mark_story_done()` (4 occurrences in `src/schemas/prd.rs` tests)

## Desired End State
A codebase where:
1. **All schema updates use immutable builders** - `item.with_state(WorkflowState::Done)` instead of `item.state = WorkflowState::Done`
2. **No public `&mut self` methods** on core data structures (Item, Prd, Story)
3. **All operations are idempotent** - calling `with_state(Done).with_state(Done)` produces same result as `with_state(Done)`
4. **Property-based tests verify immutability** - proptest checks that operations never mutate inputs
5. **Test helpers use immutable patterns** - no more `story.status = *status` in tests

Verification: Run `cargo test` - all tests pass with no mutable method usage in business logic.

## What We're NOT Doing
- ~~Implementing workflow phase runners~~ (out of scope - workflow module is empty)
- ~~Adding `Arc` for shared data~~ (premature optimization - clone is cheap for small structs)
- ~~Removing `Clone` derives~~ (still needed for test helpers and internal operations)
- ~~Breaking existing public API without deprecation~~ (will add `#[deprecated]` attributes)
- ~~Refactoring git operations~~ (already idempotent - check-before-create pattern is good)
- ~~Changing serialization format** (serde JSON format stays the same - this is internal refactoring only)

## Implementation Approach

**Strategy:** Incremental refactoring with backwards compatibility
1. Add new `with_*` methods alongside old mutable methods
2. Mark old methods as `#[deprecated]` but keep them working
3. Update internal code to use new methods
4. Update tests to use new methods
5. Add property-based tests to verify immutability
6. (Future major version) Remove deprecated methods

**Rationale:**
- Minimizes risk - old code continues working during transition
- Allows gradual migration - can update file-by-file
- Tests can verify both old and new APIs produce same results
- Property tests will catch any accidental mutations

---

## Phase 1: Fix compilation error and add derived traits

### Overview
Create the missing workflow module file and add `PartialEq` derives needed for testing.

### Changes Required:

#### 1. Create empty workflow module
**File**: `src/workflow/mod.rs`
**Changes**: Create placeholder module file

```rust
//! Workflow phase runners
//!
//! This module will contain the implementation of each workflow phase:
//! - Research phase
//! - Planning phase
//! - Implementation phase
//! - PR phase
//! - Completion phase
//!
// Placeholder for future implementation
```

#### 2. Add PartialEq to Story
**File**: `src/schemas/prd.rs:22`
**Changes**: Add `PartialEq` to Story derive macro

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
```

#### 3. Add PartialEq to Item
**File**: `src/schemas/item.rs:63`
**Changes**: Add `PartialEq` to Item derive macro

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
```

#### 4. Add PartialEq to Prd
**File**: `src/schemas/prd.rs:68`
**Changes**: Add `PartialEq` to Prd derive macro

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prd {
```

### Success Criteria:

#### Automated Verification:
- [ ] Tests compile: `cargo test --lib` succeeds without module errors
- [ ] All existing tests pass: `cargo test --lib` shows all tests passing
- [ ] No new warnings: Check for unused derives or other warnings

**Note**: This phase is purely enabling - no behavior changes. Verify compilation succeeds before proceeding.

---

## Phase 2: Add immutable builder methods to Item

### Overview
Add `with_*` builder methods to `Item` that return new instances instead of mutating self. Mark old `touch()` method as deprecated but keep it functional.

### Changes Required:

#### 1. Add builder methods to Item
**File**: `src/schemas/item.rs:140-173`
**Changes**: Add new immutable builder methods after `new()` and before `touch()`

```rust
impl Item {
    /// Create a new item with default values
    pub fn new(id: String, title: String, overview: String) -> Self {
        // ... existing code unchanged ...
    }

    // ===== NEW IMMUTABLE BUILDER METHODS =====

    /// Return a new Item with the given state, updating the timestamp
    pub fn with_state(mut self, state: WorkflowState) -> Self {
        self.state = state;
        self.touch()
    }

    /// Return a new Item with the given branch, updating the timestamp
    pub fn with_branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self.touch()
    }

    /// Return a new Item with the given PR info, updating the timestamp
    pub fn with_pr(mut self, pr_url: Option<String>, pr_number: Option<u32>) -> Self {
        self.pr_url = pr_url;
        self.pr_number = pr_number;
        self.touch()
    }

    /// Return a new Item with the given error message, updating the timestamp
    pub fn with_error(mut self, error: Option<String>) -> Self {
        self.last_error = error;
        self.touch()
    }

    /// Return a new Item with updated_at set to now
    pub fn with_updated_timestamp(mut self) -> Self {
        self.touch()
    }

    // ===== EXISTING METHOD (NOW DEPRECATED) =====

    /// Update the updated_at timestamp to now
    ///
    /// **Deprecated:** Use `with_updated_timestamp()` for immutable updates instead.
    #[deprecated(since = "0.2.0", note = "Use with_updated_timestamp() for immutable updates")]
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Tests compile: `cargo test --lib schemas::item::tests` succeeds
- [ ] All Item tests pass: Verify `test_item_json_round_trip`, `test_item_with_optional_fields`, etc. pass
- [ ] Deprecation warning emitted: Check that calling `touch()` generates deprecation warning in tests

#### Manual Verification:
- [ ] Builder methods work fluently: Can chain `item.with_state(Done).with_error(None)`
- [ ] Timestamps are updated: `with_state()` produces item with newer `updated_at` than original
- [ ] Original is unchanged: After `item.with_state(Done)`, the original `item.state` is unchanged

**Note**: The deprecated `touch()` is used internally by the new methods, so it can't be fully removed yet.

---

## Phase 3: Add immutable builder methods to Prd and Story

### Overview
Add `with_*` builder methods to `Prd` and `Story` for immutable updates. Mark `mark_story_done()` as deprecated.

### Changes Required:

#### 1. Add builder methods to Story
**File**: `src/schemas/prd.rs:43-65`
**Changes**: Add immutable builder methods after `is_pending()`

```rust
impl Story {
    /// Create a new pending story
    pub fn new(id: String, title: String, acceptance_criteria: Vec<String>, priority: u32) -> Self {
        // ... existing code unchanged ...
    }

    // ===== NEW IMMUTABLE BUILDER METHODS =====

    /// Return a new Story with the given status
    pub fn with_status(mut self, status: StoryStatus) -> Self {
        self.status = status;
        self
    }

    /// Return a new Story with the given notes
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = notes;
        self
    }

    /// Return a new Story marked as done
    pub fn as_done(self) -> Self {
        self.with_status(StoryStatus::Done)
    }

    // ===== EXISTING METHODS (UNCHANGED) =====

    /// Check if the story is done
    pub fn is_done(&self) -> bool {
        self.status == StoryStatus::Done
    }

    /// Check if the story is pending
    pub fn is_pending(&self) -> bool {
        self.status == StoryStatus::Pending
    }
}
```

#### 2. Add builder methods to Prd
**File**: `src/schemas/prd.rs:83-129`
**Changes**: Add immutable builder methods after `next_pending_story()`, before `mark_story_done()`

```rust
impl Prd {
    /// Create a new empty PRD
    pub fn new(id: String, branch_name: String) -> Self {
        // ... existing code unchanged ...
    }

    // ... existing query methods unchanged ...

    // ===== NEW IMMUTABLE BUILDER METHODS =====

    /// Return a new Prd with the given story status updated
    ///
    /// If the story_id is not found, returns the Prd unchanged.
    pub fn with_story_status(&self, story_id: &str, status: StoryStatus) -> Self {
        Prd {
            user_stories: self
                .user_stories
                .iter()
                .map(|s| {
                    if s.id == story_id {
                        s.clone().with_status(status)
                    } else {
                        s.clone()
                    }
                })
                .collect(),
            ..self.clone()
        }
    }

    /// Return a new Prd with the given story added or updated
    pub fn with_story(&self, story: Story) -> Self {
        let mut stories: Vec<_> = self
            .user_stories
            .iter()
            .filter(|s| s.id != story.id)
            .cloned()
            .collect();
        stories.push(story);
        Prd {
            user_stories: stories,
            ..self.clone()
        }
    }

    /// Return a new Prd with a story marked as done
    ///
    /// If the story_id is not found, returns the Prd unchanged.
    pub fn with_story_done(&self, story_id: &str) -> Self {
        self.with_story_status(story_id, StoryStatus::Done)
    }

    /// Return a new Prd with all stories marked as done
    pub fn with_all_stories_done(&self) -> Self {
        Prd {
            user_stories: self
                .user_stories
                .iter()
                .map(|s| s.clone().with_status(StoryStatus::Done))
                .collect(),
            ..self.clone()
        }
    }

    // ===== EXISTING METHOD (NOW DEPRECATED) =====

    /// Mark a story as done by ID
    ///
    /// **Deprecated:** Use `with_story_done()` for immutable updates instead.
    #[deprecated(since = "0.2.0", note = "Use with_story_done() for immutable updates")]
    pub fn mark_story_done(&mut self, story_id: &str) -> bool {
        // ... existing implementation unchanged ...
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Tests compile: `cargo test --lib schemas::prd::tests` succeeds
- [ ] All Prd tests pass: Verify all existing Prd tests still pass
- [ ] Deprecation warnings: Calling `mark_story_done()` shows deprecation warning

#### Manual Verification:
- [ ] Builder methods work: `prd.with_story_done("US-001")` returns new Prd with story updated
- [ ] Original unchanged: After calling `prd.with_story_done()`, original `prd.user_stories` unchanged
- [ ] Missing story handled: `with_story_status("nonexistent", Done)` returns Prd unchanged (no panic)
- [ ] Fluent API works: Can chain `prd.with_story(story).with_all_stories_done()`

**Note**: These methods use `clone()` heavily, which is acceptable for small data structures. Profile before optimizing.

---

## Phase 4: Update domain transitions to use immutable builders

### Overview
Refactor `apply_state_transition()` and test helpers to use the new `with_*` methods instead of direct field mutation.

### Changes Required:

#### 1. Refactor apply_state_transition
**File**: `src/domain/transitions.rs:84-89`
**Changes**: Replace manual field mutation with `with_state()` builder

```rust
// OLD CODE:
// Create a new item with the updated state - never mutate the original
let mut next_item = item.clone();
next_item.state = next_state;
next_item.updated_at = chrono::Utc::now().to_rfc3339();

// NEW CODE:
// Create a new item with the updated state - never mutate the original
let next_item = item.clone().with_state(next_state);
```

#### 2. Update test helper make_item
**File**: `src/domain/transitions.rs:97-105`
**Changes**: Use builder pattern instead of field mutation

```rust
// OLD CODE:
fn make_item(state: WorkflowState) -> Item {
    let mut item = Item::new(
        "test-001".to_string(),
        "Test Item".to_string(),
        "Test overview".to_string(),
    );
    item.state = state;
    item
}

// NEW CODE:
fn make_item(state: WorkflowState) -> Item {
    Item::new(
        "test-001".to_string(),
        "Test Item".to_string(),
        "Test overview".to_string(),
    )
    .with_state(state)
}
```

#### 3. Update test helper make_prd_with_stories
**File**: `src/domain/transitions.rs:107-120`
**Changes**: Use Story builder methods instead of field mutation

```rust
// OLD CODE:
fn make_prd_with_stories(statuses: &[StoryStatus]) -> Prd {
    let mut prd = Prd::new("test".to_string(), "wreckit/test".to_string());
    for (i, status) in statuses.iter().enumerate() {
        let mut story = Story::new(
            format!("US-{:03}", i + 1),
            format!("Story {}", i + 1),
            vec![],
            (i + 1) as u32,
        );
        story.status = *status;
        prd.user_stories.push(story);
    }
    prd
}

// NEW CODE:
fn make_prd_with_stories(statuses: &[StoryStatus]) -> Prd {
    let mut prd = Prd::new("test".to_string(), "wreckit/test".to_string());
    for (i, status) in statuses.iter().enumerate() {
        let story = Story::new(
            format!("US-{:03}", i + 1),
            format!("Story {}", i + 1),
            vec![],
            (i + 1) as u32,
        )
        .with_status(*status);
        prd.user_stories.push(story);
    }
    prd
}
```

#### 4. Update validation test helper
**File**: `src/domain/validation.rs:162-175`
**Changes**: Same pattern as above - use Story builders

```rust
// Same refactoring as step 3 for make_prd_with_stories in validation.rs
```

### Success Criteria:

#### Automated Verification:
- [ ] All domain tests pass: `cargo test --lib domain` succeeds
- [ ] Transition tests pass: `test_transition_idea_to_researched`, `test_transition_does_not_mutate_original`, etc.
- [ ] No mutable field access: Verify no `item.state =` or `story.status =` in domain code

#### Manual Verification:
- [ ] Transitions produce correct state: `apply_state_transition(&item, &ctx)` returns Item with new state
- [ ] Original item unchanged: After transition, original item has same state and updated_at
- [ ] Tests still readable: Test code using builders is as clear as old mutation code

**Note**: The test helpers still use `mut prd` because we're pushing into a vector. That's acceptable - the mutation is local to the helper.

---

## Phase 5: Update PRD tests to use immutable builders

### Overview
Update all tests in `src/schemas/prd.rs` that call `mark_story_done()` to use `with_story_done()` instead.

### Changes Required:

#### 1. Update test_prd_all_stories_done
**File**: `src/schemas/prd.rs:158-186`
**Changes**: Replace `mark_story_done()` calls with immutable version

```rust
// OLD CODE at line 175:
prd.mark_story_done("US-001");

// NEW CODE:
prd = prd.with_story_done("US-001");
```

#### 2. Update test_prd_has_pending_stories
**File**: `src/schemas/prd.rs:188-204`
**Changes**: Same replacement pattern

```rust
// OLD CODE at line 202:
prd.mark_story_done("US-001");

// NEW CODE:
prd = prd.with_story_done("US-001");
```

#### 3. Update test_prd_next_pending_story
**File**: `src/schemas/prd.rs:222-235`
**Changes**: Same replacement pattern

```rust
// OLD CODE at line 233:
prd.mark_story_done("US-001");

// NEW CODE:
prd = prd.with_story_done("US-001");
```

#### 4. Suppress deprecation warnings in tests
**File**: `src/schemas/prd.rs:131-132`
**Changes**: Add test module-level attribute to allow deprecated calls in old tests

```rust
#[cfg(test)]
mod tests {
    #![allow(deprecated)]  // Allow testing deprecated methods
    use super::*;
    // ... rest of test module ...
}
```

### Success Criteria:

#### Automated Verification:
- [ ] All PRD tests pass: `cargo test --lib schemas::prd::tests` succeeds
- [ ] No deprecation warnings: New test code doesn't generate warnings
- [ ] Old tests still work: Deprecation warnings suppressed for old-style tests

#### Manual Verification:
- [ ] Tests use immutable pattern: After `prd = prd.with_story_done("US-001")`, old prd is unchanged
- [ ] Test assertions still valid: `assert!(prd.all_stories_done())` works with new pattern
- [ ] No test logic changes: Only the mechanics of updating state change

**Note**: We're rebinding `prd = prd.with_story_done()` which shadows the old variable. This is idiomatic Rust.

---

## Phase 6: Add property-based tests

### Overview
Add `proptest` dependency and create property-based tests that verify immutability and idempotency guarantees.

### Changes Required:

#### 1. Add proptest dependency
**File**: `Cargo.toml:44-46`
**Changes**: Add proptest to dev-dependencies

```toml
[dev-dependencies]
tempfile = "3"
proptest = "1"  # NEW: Property-based testing
```

#### 2. Create property tests module
**File**: `src/domain/property_tests.rs` (new file)
**Changes**: Create new test module with property-based tests

```rust
//! Property-based tests for domain logic
//!
//! These tests use proptest to verify invariants across many random inputs.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{Item, Prd, Story, StoryStatus, WorkflowState};
    use crate::domain::transitions::apply_state_transition;
    use crate::domain::validation::ValidationContext;
    use proptest::prelude::*;

    // ===== STRATEGY HELPERS =====

    /// Generate a random WorkflowState
    fn any_workflow_state() -> impl Strategy<Value = WorkflowState> {
        prop_oneof![
            Just(WorkflowState::Idea),
            Just(WorkflowState::Researched),
            Just(WorkflowState::Planned),
            Just(WorkflowState::Implementing),
            Just(WorkflowState::InPr),
            Just(WorkflowState::Done),
        ]
    }

    /// Generate a random StoryStatus
    fn any_story_status() -> impl Strategy<Value = StoryStatus> {
        prop_oneof![Just(StoryStatus::Pending), Just(StoryStatus::Done),]
    }

    /// Generate a random Item
    fn any_item() -> impl Strategy<Value = Item> {
        any_workflow_state().prop_map(|state| {
            Item::new(
                "test-001".to_string(),
                "Test Item".to_string(),
                "Test overview".to_string(),
            )
            .with_state(state)
        })
    }

    /// Generate a random Prd with stories
    fn any_prd() -> impl Strategy<Value = Prd> {
        prop::collection::vec(any_story_status(), 0..5).prop_map(|statuses| {
            let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
            for (i, status) in statuses.iter().enumerate() {
                let story = Story::new(
                    format!("US-{:03}", i + 1),
                    format!("Story {}", i + 1),
                    vec!["Criterion".to_string()],
                    (i + 1) as u32,
                )
                .with_status(*status);
                prd.user_stories.push(story);
            }
            prd
        })
    }

    // ===== IMMUTABILITY TESTS =====

    proptest! {
        /// Property: apply_state_transition never mutates its input
        #[test]
        fn test_apply_transition_never_mutates(item in any_item()) {
            let original = item.clone();
            let ctx = ValidationContext::default();
            let _ = apply_state_transition(&item, &ctx);
            prop_assert_eq!(item, original);
        }

        /// Property: with_state returns a new item without modifying original
        #[test]
        fn test_with_state_is_immutable(item in any_item(), new_state in any_workflow_state()) {
            let original = item.clone();
            let _updated = item.with_state(new_state);
            prop_assert_eq!(item, original);
        }

        /// Property: with_story_status returns a new Prd without modifying original
        #[test]
        fn test_with_story_status_is_immutable(
            prd in any_prd(),
            story_id in "[A-Z]{2}-[0-9]{3}",
            status in any_story_status()
        ) {
            let original = prd.clone();
            let _updated = prd.with_story_status(&story_id, status);
            prop_assert_eq!(prd, original);
        }

        /// Property: with_pr doesn't mutate original
        #[test]
        fn test_with_pr_is_immutable(
            item in any_item(),
            pr_url in ".*",
            pr_number in 1u32..1000
        ) {
            let original = item.clone();
            let _updated = item.with_pr(Some(pr_url), Some(pr_number));
            prop_assert_eq!(item, original);
        }
    }

    // ===== IDEMPOTENCY TESTS =====

    proptest! {
        /// Property: with_state is idempotent - applying same state twice yields same result
        #[test]
        fn test_with_state_is_idempotent(item in any_item(), state in any_workflow_state()) {
            let updated_once = item.clone().with_state(state);
            let updated_twice = updated_once.clone().with_state(state);
            prop_assert_eq!(updated_once, updated_twice);
        }

        /// Property: with_error(None) is idempotent
        #[test]
        fn test_with_error_none_is_idempotent(item in any_item()) {
            let updated_once = item.clone().with_error(None);
            let updated_twice = updated_once.with_error(None);
            prop_assert_eq!(updated_once, updated_twice);
        }

        /// Property: with_story_status is idempotent for same story_id and status
        #[test]
        fn test_with_story_status_is_idempotent(
            prd in any_prd(),
            story_id in "[A-Z]{2}-[0-9]{3}",
            status in any_story_status()
        ) {
            let updated_once = prd.with_story_status(&story_id, status);
            let updated_twice = updated_once.with_story_status(&story_id, status);
            prop_assert_eq!(updated_once, updated_twice);
        }

        /// Property: with_all_stories_done is idempotent
        #[test]
        fn test_with_all_stories_done_is_idempotent(prd in any_prd()) {
            let updated_once = prd.with_all_stories_done();
            let updated_twice = updated_once.with_all_stories_done();
            prop_assert_eq!(updated_once, updated_twice);
        }
    }

    // ===== TIMESTAMP TESTS =====

    proptest! {
        /// Property: with_state always updates timestamp
        #[test]
        fn test_with_state_updates_timestamp(item in any_item()) {
            let updated = item.with_state(WorkflowState::Done);
            prop_assert!(updated.updated_at > item.updated_at);
        }

        /// Property: with_pr updates timestamp
        #[test]
        fn test_with_pr_updates_timestamp(item in any_item()) {
            let updated = item.with_pr(Some("https://github.com/".to_string()), Some(1));
            prop_assert!(updated.updated_at > item.updated_at);
        }

        /// Property: with_error updates timestamp
        #[test]
        fn test_with_error_updates_timestamp(item in any_item()) {
            let updated = item.with_error(Some("error".to_string()));
            prop_assert!(updated.updated_at > item.updated_at);
        }
    }
}
```

#### 3. Add module to domain/mod.rs
**File**: `src/domain/mod.rs`
**Changes**: Include property tests module

```rust
//! Domain logic for wreckit
//!
//! This module contains the core business logic for workflow state transitions
//! and validation rules.

pub mod states;
pub mod transitions;
pub mod validation;

// Property-based tests (compiled only in test builds)
#[cfg(test)]
mod property_tests;
```

### Success Criteria:

#### Automated Verification:
- [ ] Proptest compiles: `cargo test --lib domain::property_tests` succeeds
- [ ] Property tests pass: Run `cargo test --lib` and verify all proptest cases pass
- [ ] Tests find no counterexamples: proptest should complete without finding failures
- [ ] Test coverage: All new property tests run and pass

#### Manual Verification:
- [ ] Run property tests multiple times: Execute `cargo test --lib property_tests` several times - results consistent
- [ ] Check test runtime: Property tests complete in reasonable time (< 30 seconds)
- [ ] Review test output: Verify proptest ran many cases (check output for "passed X tests")

**Note**: Property tests run many random cases (default 256 per property). They may be slower than unit tests but provide much stronger guarantees.

---

## Testing Strategy

### Unit Tests:
All existing unit tests must continue passing:
- `src/schemas/item.rs` tests - Item creation, serialization, field access
- `src/schemas/prd.rs` tests - Prd/Story creation, story status queries
- `src/domain/transitions.rs` tests - All state transitions, immutability checks
- `src/domain/validation.rs` tests - Validation rules, PRD state queries

### Property-Based Tests:
New property tests verify invariants across random inputs:
- Immutability tests - No operation mutates input parameters
- Idempotency tests - Applying same operation twice yields same result
- Timestamp tests - All update operations update `updated_at` field

### Integration Tests:
No integration tests in this phase - all changes are internal refactoring.

### Manual Testing Steps:
1. Compile the project: `cargo build`
2. Run all tests: `cargo test --lib`
3. Check for warnings: Ensure no unexpected warnings
4. Verify deprecation warnings: Old methods should show deprecation notices
5. Run property tests multiple times: Verify consistent results

## Migration Notes

### For Developers Using This Library:
- Old mutable methods still work but generate deprecation warnings
- Update code to use `with_*` methods: `item.with_state(Done)` instead of `item.state = Done`
- All changes are backwards compatible - no breaking changes in this release

### For Internal Code:
- Test helpers updated to use immutable patterns
- Domain logic refactored to use builders
- No functional changes - purely internal refactoring

### Future Work:
- Next major version can remove deprecated `&mut self` methods
- Consider adding `#[must_use]` attribute to builder methods to catch ignored results
- Profile performance if cloning becomes bottleneck (add `Arc` if needed)

## References
- Research: `/Users/mhostetler/Source/wreckit_rust/.wreckit/items/002-implement-data-driven-immutable-core/research.md`
- TypeScript reference: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/domain/transitions.ts:38-42`
- Property test example: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/__tests__/domain.property.test.ts:163-196`
- Existing immutable pattern: `/Users/mhostetler/Source/wreckit_rust/src/domain/transitions.rs:67-90`
- Files to modify:
  - `src/schemas/item.rs` - Add `with_*` methods
  - `src/schemas/prd.rs` - Add `with_*` methods for Prd/Story
  - `src/domain/transitions.rs` - Use builders instead of field mutation
  - `src/domain/validation.rs` - Update test helpers
  - `Cargo.toml` - Add proptest dependency
  - `src/domain/property_tests.rs` - New property test module
  - `src/workflow/mod.rs` - Create placeholder module file
