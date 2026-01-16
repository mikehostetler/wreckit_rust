//! Workflow state machine definitions
//!
//! The state machine follows a linear progression:
//! idea → researched → planned → implementing → in_pr → done

use crate::schemas::WorkflowState;

/// The canonical ordering of workflow states.
///
/// IMPORTANT: This is the source of truth for state ordering.
/// The state machine follows a linear progression: idea → researched → planned → implementing → in_pr → done
pub const WORKFLOW_STATES: &[WorkflowState] = &[
    WorkflowState::Idea,
    WorkflowState::Researched,
    WorkflowState::Planned,
    WorkflowState::Implementing,
    WorkflowState::InPr,
    WorkflowState::Done,
];

/// Get the 0-based index of a state in the workflow progression.
///
/// Returns the position in WORKFLOW_STATES, or usize::MAX if not found.
pub fn get_state_index(state: WorkflowState) -> usize {
    WORKFLOW_STATES
        .iter()
        .position(|&s| s == state)
        .unwrap_or(usize::MAX)
}

/// Returns the next state in the workflow progression.
///
/// Uses WORKFLOW_STATES to determine the linear state sequence.
/// Returns None for the terminal "done" state.
///
/// # Arguments
/// * `current` - The current workflow state
///
/// # Returns
/// The next state, or None if at the end of the workflow
pub fn get_next_state(current: WorkflowState) -> Option<WorkflowState> {
    let index = get_state_index(current);
    if index >= WORKFLOW_STATES.len() - 1 {
        return None;
    }
    Some(WORKFLOW_STATES[index + 1])
}

/// Returns the allowed next states for a given current state.
///
/// This workflow enforces linear progression - only the immediate next state is allowed.
/// Wrapper around get_next_state() that returns a Vec for API convenience.
///
/// # Arguments
/// * `current` - The current workflow state
///
/// # Returns
/// Vec of allowed next states (will contain 0 or 1 states)
pub fn get_allowed_next_states(current: WorkflowState) -> Vec<WorkflowState> {
    match get_next_state(current) {
        Some(next) => vec![next],
        None => vec![],
    }
}

/// Check if a state is the terminal state (done).
pub fn is_terminal_state(state: WorkflowState) -> bool {
    state == WorkflowState::Done
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_states_order() {
        assert_eq!(WORKFLOW_STATES.len(), 6);
        assert_eq!(WORKFLOW_STATES[0], WorkflowState::Idea);
        assert_eq!(WORKFLOW_STATES[1], WorkflowState::Researched);
        assert_eq!(WORKFLOW_STATES[2], WorkflowState::Planned);
        assert_eq!(WORKFLOW_STATES[3], WorkflowState::Implementing);
        assert_eq!(WORKFLOW_STATES[4], WorkflowState::InPr);
        assert_eq!(WORKFLOW_STATES[5], WorkflowState::Done);
    }

    #[test]
    fn test_get_state_index() {
        assert_eq!(get_state_index(WorkflowState::Idea), 0);
        assert_eq!(get_state_index(WorkflowState::Researched), 1);
        assert_eq!(get_state_index(WorkflowState::Planned), 2);
        assert_eq!(get_state_index(WorkflowState::Implementing), 3);
        assert_eq!(get_state_index(WorkflowState::InPr), 4);
        assert_eq!(get_state_index(WorkflowState::Done), 5);
    }

    #[test]
    fn test_get_next_state() {
        assert_eq!(get_next_state(WorkflowState::Idea), Some(WorkflowState::Researched));
        assert_eq!(get_next_state(WorkflowState::Researched), Some(WorkflowState::Planned));
        assert_eq!(get_next_state(WorkflowState::Planned), Some(WorkflowState::Implementing));
        assert_eq!(get_next_state(WorkflowState::Implementing), Some(WorkflowState::InPr));
        assert_eq!(get_next_state(WorkflowState::InPr), Some(WorkflowState::Done));
        assert_eq!(get_next_state(WorkflowState::Done), None);
    }

    #[test]
    fn test_get_allowed_next_states() {
        assert_eq!(get_allowed_next_states(WorkflowState::Idea), vec![WorkflowState::Researched]);
        assert_eq!(get_allowed_next_states(WorkflowState::Done), vec![]);
    }

    #[test]
    fn test_is_terminal_state() {
        assert!(!is_terminal_state(WorkflowState::Idea));
        assert!(!is_terminal_state(WorkflowState::Researched));
        assert!(!is_terminal_state(WorkflowState::Planned));
        assert!(!is_terminal_state(WorkflowState::Implementing));
        assert!(!is_terminal_state(WorkflowState::InPr));
        assert!(is_terminal_state(WorkflowState::Done));
    }
}
