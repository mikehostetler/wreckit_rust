//! Domain logic for workflow states and transitions

mod states;
mod transitions;
mod validation;

// Property-based tests (compiled only in test builds)
#[cfg(test)]
mod property_tests;

pub use states::{
    get_allowed_next_states, get_next_state, get_state_index, is_terminal_state, WORKFLOW_STATES,
};
pub use transitions::{apply_state_transition, TransitionResult};
pub use validation::{
    all_stories_done, can_enter_done, can_enter_implementing, can_enter_in_pr, can_enter_planned,
    can_enter_researched, has_pending_stories, validate_transition, ValidationContext,
    ValidationResult,
};
