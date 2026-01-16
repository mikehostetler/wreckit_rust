//! Validation rules for state transitions

use crate::schemas::{Prd, WorkflowState};

use super::get_allowed_next_states;

/// Context required for validating state transitions
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Whether research.md exists
    pub has_research_md: bool,

    /// Whether plan.md exists
    pub has_plan_md: bool,

    /// The PRD (if it exists and is valid)
    pub prd: Option<Prd>,

    /// Whether a PR exists
    pub has_pr: bool,

    /// Whether the PR is merged
    pub pr_merged: bool,
}

impl Default for ValidationContext {
    fn default() -> Self {
        ValidationContext {
            has_research_md: false,
            has_plan_md: false,
            prd: None,
            has_pr: false,
            pr_merged: false,
        }
    }
}

/// Result of a validation check
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub valid: bool,

    /// Reason for failure (if valid is false)
    pub reason: Option<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        ValidationResult {
            valid: true,
            reason: None,
        }
    }

    /// Create a failed validation result
    pub fn failure(reason: impl Into<String>) -> Self {
        ValidationResult {
            valid: false,
            reason: Some(reason.into()),
        }
    }
}

/// Check if all stories in a PRD are done
pub fn all_stories_done(prd: Option<&Prd>) -> bool {
    match prd {
        None => false,
        Some(prd) => {
            if prd.user_stories.is_empty() {
                return false;
            }
            prd.user_stories.iter().all(|s| s.is_done())
        }
    }
}

/// Check if a PRD has any pending stories
pub fn has_pending_stories(prd: Option<&Prd>) -> bool {
    match prd {
        None => false,
        Some(prd) => prd.user_stories.iter().any(|s| s.is_pending()),
    }
}

/// Validate entering the "researched" state
pub fn can_enter_researched(has_research_md: bool) -> ValidationResult {
    if !has_research_md {
        return ValidationResult::failure("research.md does not exist");
    }
    ValidationResult::success()
}

/// Validate entering the "planned" state
pub fn can_enter_planned(has_plan_md: bool, prd: Option<&Prd>) -> ValidationResult {
    if !has_plan_md {
        return ValidationResult::failure("plan.md does not exist");
    }
    if prd.is_none() {
        return ValidationResult::failure("prd.json is not valid");
    }
    ValidationResult::success()
}

/// Validate entering the "implementing" state
pub fn can_enter_implementing(prd: Option<&Prd>) -> ValidationResult {
    if !has_pending_stories(prd) {
        return ValidationResult::failure("prd.json has no stories with status pending");
    }
    ValidationResult::success()
}

/// Validate entering the "in_pr" state
pub fn can_enter_in_pr(prd: Option<&Prd>, has_pr: bool) -> ValidationResult {
    if !all_stories_done(prd) {
        return ValidationResult::failure("not all stories are done");
    }
    if !has_pr {
        return ValidationResult::failure("PR not created");
    }
    ValidationResult::success()
}

/// Validate entering the "done" state
pub fn can_enter_done(pr_merged: bool) -> ValidationResult {
    if !pr_merged {
        return ValidationResult::failure("PR not merged");
    }
    ValidationResult::success()
}

/// Validate a state transition
pub fn validate_transition(
    current: WorkflowState,
    target: WorkflowState,
    ctx: &ValidationContext,
) -> ValidationResult {
    let allowed = get_allowed_next_states(current);
    if !allowed.contains(&target) {
        return ValidationResult::failure(format!(
            "cannot transition from {} to {}",
            current, target
        ));
    }

    match target {
        WorkflowState::Researched => can_enter_researched(ctx.has_research_md),
        WorkflowState::Planned => can_enter_planned(ctx.has_plan_md, ctx.prd.as_ref()),
        WorkflowState::Implementing => can_enter_implementing(ctx.prd.as_ref()),
        WorkflowState::InPr => can_enter_in_pr(ctx.prd.as_ref(), ctx.has_pr),
        WorkflowState::Done => can_enter_done(ctx.pr_merged),
        WorkflowState::Idea => ValidationResult::failure("cannot transition to idea state"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{Story, StoryStatus};

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
    fn test_all_stories_done_with_none() {
        assert!(!all_stories_done(None));
    }

    #[test]
    fn test_all_stories_done_with_empty_prd() {
        let prd = Prd::new("test".to_string(), "wreckit/test".to_string());
        assert!(!all_stories_done(Some(&prd)));
    }

    #[test]
    fn test_all_stories_done_with_pending() {
        let prd = make_prd_with_stories(&[StoryStatus::Pending, StoryStatus::Done]);
        assert!(!all_stories_done(Some(&prd)));
    }

    #[test]
    fn test_all_stories_done_all_done() {
        let prd = make_prd_with_stories(&[StoryStatus::Done, StoryStatus::Done]);
        assert!(all_stories_done(Some(&prd)));
    }

    #[test]
    fn test_has_pending_stories_with_none() {
        assert!(!has_pending_stories(None));
    }

    #[test]
    fn test_has_pending_stories_with_pending() {
        let prd = make_prd_with_stories(&[StoryStatus::Pending]);
        assert!(has_pending_stories(Some(&prd)));
    }

    #[test]
    fn test_has_pending_stories_all_done() {
        let prd = make_prd_with_stories(&[StoryStatus::Done]);
        assert!(!has_pending_stories(Some(&prd)));
    }

    #[test]
    fn test_can_enter_researched() {
        assert!(can_enter_researched(true).valid);
        assert!(!can_enter_researched(false).valid);
        assert_eq!(
            can_enter_researched(false).reason,
            Some("research.md does not exist".to_string())
        );
    }

    #[test]
    fn test_can_enter_planned() {
        let prd = make_prd_with_stories(&[StoryStatus::Pending]);

        assert!(can_enter_planned(true, Some(&prd)).valid);
        assert!(!can_enter_planned(false, Some(&prd)).valid);
        assert!(!can_enter_planned(true, None).valid);
        assert!(!can_enter_planned(false, None).valid);
    }

    #[test]
    fn test_can_enter_implementing() {
        let prd_pending = make_prd_with_stories(&[StoryStatus::Pending]);
        let prd_done = make_prd_with_stories(&[StoryStatus::Done]);

        assert!(can_enter_implementing(Some(&prd_pending)).valid);
        assert!(!can_enter_implementing(Some(&prd_done)).valid);
        assert!(!can_enter_implementing(None).valid);
    }

    #[test]
    fn test_can_enter_in_pr() {
        let prd_done = make_prd_with_stories(&[StoryStatus::Done]);
        let prd_pending = make_prd_with_stories(&[StoryStatus::Pending]);

        assert!(can_enter_in_pr(Some(&prd_done), true).valid);
        assert!(!can_enter_in_pr(Some(&prd_done), false).valid);
        assert!(!can_enter_in_pr(Some(&prd_pending), true).valid);
        assert!(!can_enter_in_pr(None, true).valid);
    }

    #[test]
    fn test_can_enter_done() {
        assert!(can_enter_done(true).valid);
        assert!(!can_enter_done(false).valid);
    }

    #[test]
    fn test_validate_transition_valid() {
        let prd = make_prd_with_stories(&[StoryStatus::Pending]);
        let ctx = ValidationContext {
            has_research_md: true,
            has_plan_md: true,
            prd: Some(prd),
            has_pr: false,
            pr_merged: false,
        };

        // Valid transition: idea -> researched
        let result = validate_transition(WorkflowState::Idea, WorkflowState::Researched, &ctx);
        assert!(result.valid);

        // Valid transition: researched -> planned
        let result = validate_transition(WorkflowState::Researched, WorkflowState::Planned, &ctx);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_transition_invalid_skip() {
        let ctx = ValidationContext::default();

        // Invalid: cannot skip states
        let result = validate_transition(WorkflowState::Idea, WorkflowState::Planned, &ctx);
        assert!(!result.valid);
        assert!(result.reason.unwrap().contains("cannot transition"));
    }

    #[test]
    fn test_validate_transition_invalid_backward() {
        let ctx = ValidationContext::default();

        // Invalid: cannot go backward
        let result = validate_transition(WorkflowState::Planned, WorkflowState::Idea, &ctx);
        assert!(!result.valid);
    }

    #[test]
    fn test_validate_transition_missing_artifact() {
        let ctx = ValidationContext {
            has_research_md: false,
            ..Default::default()
        };

        // Invalid: missing research.md
        let result = validate_transition(WorkflowState::Idea, WorkflowState::Researched, &ctx);
        assert!(!result.valid);
        assert!(result.reason.unwrap().contains("research.md"));
    }

    #[test]
    fn test_validate_transition_from_terminal() {
        let ctx = ValidationContext::default();

        // Cannot transition from terminal state
        let result = validate_transition(WorkflowState::Done, WorkflowState::Idea, &ctx);
        assert!(!result.valid);
    }
}
