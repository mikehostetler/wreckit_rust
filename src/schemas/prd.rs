//! PRD schema - Product Requirements Document with user stories

use serde::{Deserialize, Serialize};

/// Status of a user story
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoryStatus {
    /// Story not yet implemented
    Pending,
    /// Story implementation complete
    Done,
}

impl Default for StoryStatus {
    fn default() -> Self {
        StoryStatus::Pending
    }
}

/// A user story within a PRD
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
    /// Unique identifier (e.g., "US-001")
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// List of acceptance criteria
    pub acceptance_criteria: Vec<String>,

    /// Priority for ordering (lower = higher priority)
    pub priority: u32,

    /// Current implementation status
    pub status: StoryStatus,

    /// Additional notes
    pub notes: String,
}

impl Story {
    /// Create a new pending story
    pub fn new(id: String, title: String, acceptance_criteria: Vec<String>, priority: u32) -> Self {
        Story {
            id,
            title,
            acceptance_criteria,
            priority,
            status: StoryStatus::Pending,
            notes: String::new(),
        }
    }

    // ===== IMMUTABLE BUILDER METHODS =====

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

/// Product Requirements Document containing user stories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prd {
    /// Schema version for forward compatibility
    pub schema_version: u32,

    /// Item ID this PRD belongs to
    pub id: String,

    /// Branch name for the implementation
    pub branch_name: String,

    /// List of user stories
    pub user_stories: Vec<Story>,
}

impl Prd {
    /// Create a new empty PRD
    pub fn new(id: String, branch_name: String) -> Self {
        Prd {
            schema_version: 1,
            id,
            branch_name,
            user_stories: Vec::new(),
        }
    }

    /// Check if all stories are done
    pub fn all_stories_done(&self) -> bool {
        if self.user_stories.is_empty() {
            return false;
        }
        self.user_stories.iter().all(|s| s.is_done())
    }

    /// Check if there are any pending stories
    pub fn has_pending_stories(&self) -> bool {
        self.user_stories.iter().any(|s| s.is_pending())
    }

    /// Get pending stories sorted by priority
    pub fn pending_stories(&self) -> Vec<&Story> {
        let mut stories: Vec<_> = self.user_stories.iter().filter(|s| s.is_pending()).collect();
        stories.sort_by_key(|s| s.priority);
        stories
    }

    /// Get the next pending story (lowest priority number)
    pub fn next_pending_story(&self) -> Option<&Story> {
        self.pending_stories().first().copied()
    }

    // ===== IMMUTABLE BUILDER METHODS =====

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
        for story in &mut self.user_stories {
            if story.id == story_id {
                story.status = StoryStatus::Done;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)] // Allow testing deprecated methods
    use super::*;

    #[test]
    fn test_story_status_serialization() {
        assert_eq!(serde_json::to_string(&StoryStatus::Pending).unwrap(), "\"pending\"");
        assert_eq!(serde_json::to_string(&StoryStatus::Done).unwrap(), "\"done\"");
    }

    #[test]
    fn test_story_creation() {
        let story = Story::new(
            "US-001".to_string(),
            "Test Story".to_string(),
            vec!["Criterion 1".to_string(), "Criterion 2".to_string()],
            1,
        );

        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "Test Story");
        assert_eq!(story.acceptance_criteria.len(), 2);
        assert_eq!(story.priority, 1);
        assert!(story.is_pending());
        assert!(!story.is_done());
    }

    #[test]
    fn test_prd_all_stories_done() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        // Empty PRD is not "all done"
        assert!(!prd.all_stories_done());

        // Add a pending story
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));
        assert!(!prd.all_stories_done());

        // Mark it done
        prd = prd.with_story_done("US-001");
        assert!(prd.all_stories_done());

        // Add another pending story
        prd.user_stories.push(Story::new(
            "US-002".to_string(),
            "Story 2".to_string(),
            vec![],
            2,
        ));
        assert!(!prd.all_stories_done());
    }

    #[test]
    fn test_prd_has_pending_stories() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        assert!(!prd.has_pending_stories());

        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));
        assert!(prd.has_pending_stories());

        prd = prd.with_story_done("US-001");
        assert!(!prd.has_pending_stories());
    }

    #[test]
    fn test_prd_pending_stories_sorted() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        // Add stories out of priority order
        prd.user_stories.push(Story::new("US-003".to_string(), "Story 3".to_string(), vec![], 3));
        prd.user_stories.push(Story::new("US-001".to_string(), "Story 1".to_string(), vec![], 1));
        prd.user_stories.push(Story::new("US-002".to_string(), "Story 2".to_string(), vec![], 2));

        let pending = prd.pending_stories();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].id, "US-001");
        assert_eq!(pending[1].id, "US-002");
        assert_eq!(pending[2].id, "US-003");
    }

    #[test]
    fn test_prd_next_pending_story() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        assert!(prd.next_pending_story().is_none());

        prd.user_stories.push(Story::new("US-002".to_string(), "Story 2".to_string(), vec![], 2));
        prd.user_stories.push(Story::new("US-001".to_string(), "Story 1".to_string(), vec![], 1));

        assert_eq!(prd.next_pending_story().unwrap().id, "US-001");

        prd = prd.with_story_done("US-001");
        assert_eq!(prd.next_pending_story().unwrap().id, "US-002");
    }

    #[test]
    fn test_prd_json_round_trip() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Test Story".to_string(),
            vec!["Criterion 1".to_string()],
            1,
        ));

        let json = serde_json::to_string_pretty(&prd).unwrap();
        let parsed: Prd = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, prd.id);
        assert_eq!(parsed.branch_name, prd.branch_name);
        assert_eq!(parsed.user_stories.len(), 1);
        assert_eq!(parsed.user_stories[0].id, "US-001");
    }

    #[test]
    fn test_story_with_status() {
        let story = Story::new(
            "US-001".to_string(),
            "Test Story".to_string(),
            vec!["Criterion 1".to_string()],
            1,
        );

        assert!(story.is_pending());

        let done_story = story.clone().with_status(StoryStatus::Done);
        assert!(done_story.is_done());
        assert!(story.is_pending()); // Original unchanged
    }

    #[test]
    fn test_story_with_notes() {
        let story = Story::new(
            "US-001".to_string(),
            "Test Story".to_string(),
            vec!["Criterion 1".to_string()],
            1,
        );

        assert_eq!(story.notes, "");

        let updated = story.clone().with_notes("Some notes".to_string());
        assert_eq!(updated.notes, "Some notes");
        assert_eq!(story.notes, ""); // Original unchanged
    }

    #[test]
    fn test_story_as_done() {
        let story = Story::new(
            "US-001".to_string(),
            "Test Story".to_string(),
            vec!["Criterion 1".to_string()],
            1,
        );

        let done_story = story.clone().as_done();
        assert!(done_story.is_done());
        assert!(story.is_pending()); // Original unchanged
    }

    #[test]
    fn test_prd_with_story_status() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));
        prd.user_stories.push(Story::new("US-002".to_string(), "Story 2".to_string(), vec![], 2));

        let updated = prd.with_story_status("US-001", StoryStatus::Done);

        assert!(updated.user_stories[0].is_done());
        assert!(updated.user_stories[1].is_pending());
        assert!(prd.user_stories[0].is_pending()); // Original unchanged
    }

    #[test]
    fn test_prd_with_story_status_missing_id() {
        let prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        // Should not panic, just return unchanged
        let updated = prd.with_story_status("US-999", StoryStatus::Done);
        assert_eq!(updated.user_stories.len(), 0);
    }

    #[test]
    fn test_prd_with_story() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));

        let new_story = Story::new("US-002".to_string(), "Story 2".to_string(), vec![], 2);
        let updated = prd.with_story(new_story);

        assert_eq!(updated.user_stories.len(), 2);
        assert_eq!(prd.user_stories.len(), 1); // Original unchanged
    }

    #[test]
    fn test_prd_with_story_replace() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));

        let updated_story = Story::new(
            "US-001".to_string(),
            "Updated Story 1".to_string(),
            vec![],
            1,
        )
        .with_status(StoryStatus::Done);

        let updated = prd.with_story(updated_story);

        assert_eq!(updated.user_stories.len(), 1);
        assert_eq!(updated.user_stories[0].title, "Updated Story 1");
        assert!(updated.user_stories[0].is_done());
        assert_eq!(prd.user_stories[0].title, "Story 1"); // Original unchanged
    }

    #[test]
    fn test_prd_with_story_done() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new(
            "US-001".to_string(),
            "Story 1".to_string(),
            vec![],
            1,
        ));

        let updated = prd.with_story_done("US-001");

        assert!(updated.user_stories[0].is_done());
        assert!(prd.user_stories[0].is_pending()); // Original unchanged
    }

    #[test]
    fn test_prd_with_all_stories_done() {
        let mut prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());
        prd.user_stories.push(Story::new("US-001".to_string(), "Story 1".to_string(), vec![], 1));
        prd.user_stories.push(Story::new("US-002".to_string(), "Story 2".to_string(), vec![], 2));

        let updated = prd.with_all_stories_done();

        assert!(updated.all_stories_done());
        assert!(!prd.all_stories_done()); // Original unchanged
    }
}
