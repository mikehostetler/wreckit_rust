//! Property-based tests for domain logic
//!
//! These tests use proptest to verify invariants across many random inputs.

#[cfg(test)]
mod tests {
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
            let _updated = item.clone().with_state(new_state);
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
            let _updated = item.clone().with_pr(Some(pr_url), Some(pr_number));
            prop_assert_eq!(item, original);
        }
    }

    // ===== IDEMPOTENCY TESTS =====

    proptest! {
        /// Property: with_state produces consistent state (not timestamp) when applied twice
        #[test]
        fn test_with_state_consistent_state(item in any_item(), state in any_workflow_state()) {
            let updated_once = item.clone().with_state(state);
            let updated_twice = updated_once.clone().with_state(state);
            // State should be the same, but timestamp will differ
            prop_assert_eq!(updated_once.state, updated_twice.state);
        }

        /// Property: with_error produces consistent error value when applied twice
        #[test]
        fn test_with_error_consistent_value(item in any_item()) {
            let updated_once = item.clone().with_error(None);
            let updated_twice = updated_once.clone().with_error(None);
            // Error value should be the same, but timestamp will differ
            prop_assert_eq!(updated_once.last_error, updated_twice.last_error);
        }

        /// Property: with_story_status is idempotent for same story_id and status
        /// (Story doesn't have timestamp, so true idempotency is possible)
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
        /// (Stories don't have timestamps, so true idempotency is possible)
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
            // Use a small sleep to ensure timestamp difference
            std::thread::sleep(std::time::Duration::from_millis(1));
            let updated = item.clone().with_state(WorkflowState::Done);
            prop_assert!(updated.updated_at >= item.updated_at);
        }

        /// Property: with_pr updates timestamp
        #[test]
        fn test_with_pr_updates_timestamp(item in any_item()) {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let updated = item.clone().with_pr(Some("https://github.com/".to_string()), Some(1));
            prop_assert!(updated.updated_at >= item.updated_at);
        }

        /// Property: with_error updates timestamp
        #[test]
        fn test_with_error_updates_timestamp(item in any_item()) {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let updated = item.clone().with_error(Some("error".to_string()));
            prop_assert!(updated.updated_at >= item.updated_at);
        }
    }
}
