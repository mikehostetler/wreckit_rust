//! State transition logic
//!
//! Pure functions for applying state transitions to items.

use crate::schemas::Item;

use super::states::get_next_state;
use super::validation::{validate_transition, ValidationContext};

/// Result of a state transition attempt
#[derive(Debug)]
pub enum TransitionResult {
    /// Successful transition with the new item state
    Success {
        /// The item with updated state and timestamp
        next_item: Item,
    },
    /// Failed transition with error message
    Error {
        /// Description of why the transition failed
        error: String,
    },
}

impl TransitionResult {
    /// Check if the transition was successful
    pub fn is_success(&self) -> bool {
        matches!(self, TransitionResult::Success { .. })
    }

    /// Check if the transition failed
    pub fn is_error(&self) -> bool {
        matches!(self, TransitionResult::Error { .. })
    }

    /// Get the next item if the transition was successful
    pub fn item(self) -> Option<Item> {
        match self {
            TransitionResult::Success { next_item } => Some(next_item),
            TransitionResult::Error { .. } => None,
        }
    }

    /// Get the error message if the transition failed
    pub fn error(self) -> Option<String> {
        match self {
            TransitionResult::Success { .. } => None,
            TransitionResult::Error { error } => Some(error),
        }
    }
}

/// Pure function that applies a state transition to an item.
///
/// This function:
/// - Never mutates the input item
/// - Validates the transition before applying
/// - Returns a new Item with updated state and updated_at
/// - Returns an error if transition is invalid
///
/// # Arguments
/// * `item` - The current item (immutable reference)
/// * `ctx` - Validation context containing artifact existence flags
///
/// # Returns
/// A TransitionResult indicating success (with new item) or error (with message)
pub fn apply_state_transition(item: &Item, ctx: &ValidationContext) -> TransitionResult {
    let next_state = match get_next_state(item.state) {
        Some(state) => state,
        None => {
            return TransitionResult::Error {
                error: format!("Cannot transition from terminal state: {}", item.state),
            };
        }
    };

    let validation = validate_transition(item.state, next_state, ctx);
    if !validation.valid {
        return TransitionResult::Error {
            error: validation.reason.unwrap_or_else(|| "Transition validation failed".to_string()),
        };
    }

    // Create a new item with the updated state - never mutate the original
    let next_item = item.clone().with_state(next_state);

    TransitionResult::Success { next_item }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{Prd, Story, StoryStatus, WorkflowState};

    fn make_item(state: WorkflowState) -> Item {
        Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        )
        .with_state(state)
    }

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

    #[test]
    fn test_transition_idea_to_researched() {
        let item = make_item(WorkflowState::Idea);
        let ctx = ValidationContext {
            has_research_md: true,
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());

        let next = result.item().unwrap();
        assert_eq!(next.state, WorkflowState::Researched);
        assert_ne!(next.updated_at, item.updated_at);
    }

    #[test]
    fn test_transition_researched_to_planned() {
        let item = make_item(WorkflowState::Researched);
        let prd = make_prd_with_stories(&[StoryStatus::Pending]);
        let ctx = ValidationContext {
            has_plan_md: true,
            prd: Some(prd),
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());

        let next = result.item().unwrap();
        assert_eq!(next.state, WorkflowState::Planned);
    }

    #[test]
    fn test_transition_planned_to_implementing() {
        let item = make_item(WorkflowState::Planned);
        let prd = make_prd_with_stories(&[StoryStatus::Pending]);
        let ctx = ValidationContext {
            prd: Some(prd),
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());

        let next = result.item().unwrap();
        assert_eq!(next.state, WorkflowState::Implementing);
    }

    #[test]
    fn test_transition_implementing_to_in_pr() {
        let item = make_item(WorkflowState::Implementing);
        let prd = make_prd_with_stories(&[StoryStatus::Done]);
        let ctx = ValidationContext {
            prd: Some(prd),
            has_pr: true,
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());

        let next = result.item().unwrap();
        assert_eq!(next.state, WorkflowState::InPr);
    }

    #[test]
    fn test_transition_in_pr_to_done() {
        let item = make_item(WorkflowState::InPr);
        let ctx = ValidationContext {
            pr_merged: true,
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());

        let next = result.item().unwrap();
        assert_eq!(next.state, WorkflowState::Done);
    }

    #[test]
    fn test_transition_from_terminal_state() {
        let item = make_item(WorkflowState::Done);
        let ctx = ValidationContext::default();

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_error());

        let error = result.error().unwrap();
        assert!(error.contains("terminal state"));
    }

    #[test]
    fn test_transition_missing_artifact() {
        let item = make_item(WorkflowState::Idea);
        let ctx = ValidationContext {
            has_research_md: false,
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_error());

        let error = result.error().unwrap();
        assert!(error.contains("research.md"));
    }

    #[test]
    fn test_transition_does_not_mutate_original() {
        let item = make_item(WorkflowState::Idea);
        let original_state = item.state;
        let original_updated = item.updated_at.clone();

        let ctx = ValidationContext {
            has_research_md: true,
            ..Default::default()
        };

        let _ = apply_state_transition(&item, &ctx);

        // Original item should be unchanged
        assert_eq!(item.state, original_state);
        assert_eq!(item.updated_at, original_updated);
    }

    #[test]
    fn test_transition_result_helpers() {
        let item = make_item(WorkflowState::Idea);
        let ctx = ValidationContext {
            has_research_md: true,
            ..Default::default()
        };

        let result = apply_state_transition(&item, &ctx);
        assert!(result.is_success());
        assert!(!result.is_error());
    }
}
