//! Item schema - The main workflow item type

use serde::{Deserialize, Serialize};

/// Workflow state for an item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    /// Initial state - idea captured
    Idea,
    /// Research phase completed
    Researched,
    /// Planning phase completed
    Planned,
    /// Implementation in progress
    Implementing,
    /// Pull request created
    InPr,
    /// Work complete
    Done,
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowState::Idea => write!(f, "idea"),
            WorkflowState::Researched => write!(f, "researched"),
            WorkflowState::Planned => write!(f, "planned"),
            WorkflowState::Implementing => write!(f, "implementing"),
            WorkflowState::InPr => write!(f, "in_pr"),
            WorkflowState::Done => write!(f, "done"),
        }
    }
}

impl std::str::FromStr for WorkflowState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "idea" => Ok(WorkflowState::Idea),
            "researched" => Ok(WorkflowState::Researched),
            "planned" => Ok(WorkflowState::Planned),
            "implementing" => Ok(WorkflowState::Implementing),
            "in_pr" => Ok(WorkflowState::InPr),
            "done" => Ok(WorkflowState::Done),
            _ => Err(format!("Unknown workflow state: {}", s)),
        }
    }
}

/// Priority hint for an item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriorityHint {
    Low,
    Medium,
    High,
    Critical,
}

/// A workflow item representing a feature or task to be implemented
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Schema version for forward compatibility
    pub schema_version: u32,

    /// Unique identifier for the item
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Optional section/category for organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,

    /// Current workflow state
    pub state: WorkflowState,

    /// Overview/description of the item
    pub overview: String,

    /// Git branch name (null if not yet created)
    #[serde(default)]
    pub branch: Option<String>,

    /// PR URL (null if not yet created)
    #[serde(default)]
    pub pr_url: Option<String>,

    /// PR number (null if not yet created)
    #[serde(default)]
    pub pr_number: Option<u32>,

    /// Last error message (null if no error)
    #[serde(default)]
    pub last_error: Option<String>,

    /// ISO 8601 creation timestamp
    pub created_at: String,

    /// ISO 8601 last update timestamp
    pub updated_at: String,

    // Structured context fields for richer research/planning

    /// Problem statement for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_statement: Option<String>,

    /// Motivation for the work
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motivation: Option<String>,

    /// Success criteria for the item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<Vec<String>>,

    /// Technical constraints to consider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_constraints: Option<Vec<String>>,

    /// Items in scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_in_scope: Option<Vec<String>>,

    /// Items out of scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_out_of_scope: Option<Vec<String>>,

    /// Priority hint for ordering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_hint: Option<PriorityHint>,

    /// Urgency hint for scheduling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urgency_hint: Option<String>,
}

impl Item {
    /// Create a new item with default values
    pub fn new(id: String, title: String, overview: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Item {
            schema_version: 1,
            id,
            title,
            section: None,
            state: WorkflowState::Idea,
            overview,
            branch: None,
            pr_url: None,
            pr_number: None,
            last_error: None,
            created_at: now.clone(),
            updated_at: now,
            problem_statement: None,
            motivation: None,
            success_criteria: None,
            technical_constraints: None,
            scope_in_scope: None,
            scope_out_of_scope: None,
            priority_hint: None,
            urgency_hint: None,
        }
    }

    // ===== IMMUTABLE BUILDER METHODS =====

    /// Return a new Item with the given state, updating the timestamp
    pub fn with_state(mut self, state: WorkflowState) -> Self {
        self.state = state;
        self.touch_returning()
    }

    /// Return a new Item with the given branch, updating the timestamp
    pub fn with_branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self.touch_returning()
    }

    /// Return a new Item with the given PR info, updating the timestamp
    pub fn with_pr(mut self, pr_url: Option<String>, pr_number: Option<u32>) -> Self {
        self.pr_url = pr_url;
        self.pr_number = pr_number;
        self.touch_returning()
    }

    /// Return a new Item with the given error message, updating the timestamp
    pub fn with_error(mut self, error: Option<String>) -> Self {
        self.last_error = error;
        self.touch_returning()
    }

    /// Return a new Item with updated_at set to now
    pub fn with_updated_timestamp(self) -> Self {
        self.touch_returning()
    }

    // ===== PRIVATE HELPER =====

    /// Update the updated_at timestamp to now and return self
    fn touch_returning(mut self) -> Self {
        self.updated_at = chrono::Utc::now().to_rfc3339();
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_serialization() {
        assert_eq!(serde_json::to_string(&WorkflowState::Idea).unwrap(), "\"idea\"");
        assert_eq!(serde_json::to_string(&WorkflowState::Researched).unwrap(), "\"researched\"");
        assert_eq!(serde_json::to_string(&WorkflowState::Planned).unwrap(), "\"planned\"");
        assert_eq!(serde_json::to_string(&WorkflowState::Implementing).unwrap(), "\"implementing\"");
        assert_eq!(serde_json::to_string(&WorkflowState::InPr).unwrap(), "\"in_pr\"");
        assert_eq!(serde_json::to_string(&WorkflowState::Done).unwrap(), "\"done\"");
    }

    #[test]
    fn test_workflow_state_deserialization() {
        assert_eq!(serde_json::from_str::<WorkflowState>("\"idea\"").unwrap(), WorkflowState::Idea);
        assert_eq!(serde_json::from_str::<WorkflowState>("\"researched\"").unwrap(), WorkflowState::Researched);
        assert_eq!(serde_json::from_str::<WorkflowState>("\"planned\"").unwrap(), WorkflowState::Planned);
        assert_eq!(serde_json::from_str::<WorkflowState>("\"implementing\"").unwrap(), WorkflowState::Implementing);
        assert_eq!(serde_json::from_str::<WorkflowState>("\"in_pr\"").unwrap(), WorkflowState::InPr);
        assert_eq!(serde_json::from_str::<WorkflowState>("\"done\"").unwrap(), WorkflowState::Done);
    }

    #[test]
    fn test_item_json_round_trip() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "A test item for verification".to_string(),
        );

        let json = serde_json::to_string_pretty(&item).unwrap();
        let parsed: Item = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, item.id);
        assert_eq!(parsed.title, item.title);
        assert_eq!(parsed.overview, item.overview);
        assert_eq!(parsed.state, WorkflowState::Idea);
    }

    #[test]
    fn test_item_with_optional_fields() {
        let mut item = Item::new(
            "test-002".to_string(),
            "Test Item with Options".to_string(),
            "An item with optional fields set".to_string(),
        );
        item.section = Some("core".to_string());
        item.priority_hint = Some(PriorityHint::High);
        item.success_criteria = Some(vec!["Criterion 1".to_string(), "Criterion 2".to_string()]);

        let json = serde_json::to_string_pretty(&item).unwrap();
        let parsed: Item = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.section, Some("core".to_string()));
        assert_eq!(parsed.priority_hint, Some(PriorityHint::High));
        assert_eq!(parsed.success_criteria.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_item_skips_none_in_serialization() {
        let item = Item::new(
            "test-003".to_string(),
            "Minimal Item".to_string(),
            "An item with minimal fields".to_string(),
        );

        let json = serde_json::to_string(&item).unwrap();

        // Should not contain "section" key since it's None
        assert!(!json.contains("\"section\":"));
        assert!(!json.contains("\"priority_hint\":"));
    }

    #[test]
    fn test_priority_hint_serialization() {
        assert_eq!(serde_json::to_string(&PriorityHint::Low).unwrap(), "\"low\"");
        assert_eq!(serde_json::to_string(&PriorityHint::Medium).unwrap(), "\"medium\"");
        assert_eq!(serde_json::to_string(&PriorityHint::High).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&PriorityHint::Critical).unwrap(), "\"critical\"");
    }

    #[test]
    fn test_item_with_state() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        assert_eq!(item.state, WorkflowState::Idea);

        let updated = item.clone().with_state(WorkflowState::Done);
        assert_eq!(updated.state, WorkflowState::Done);
        assert_eq!(item.state, WorkflowState::Idea); // Original unchanged
        assert!(updated.updated_at > item.updated_at);
    }

    #[test]
    fn test_item_with_branch() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        assert!(item.branch.is_none());

        let updated = item.clone().with_branch(Some("feature/test".to_string()));
        assert_eq!(updated.branch, Some("feature/test".to_string()));
        assert!(item.branch.is_none()); // Original unchanged
        assert!(updated.updated_at > item.updated_at);
    }

    #[test]
    fn test_item_with_pr() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        assert!(item.pr_url.is_none());
        assert!(item.pr_number.is_none());

        let updated = item
            .clone()
            .with_pr(Some("https://github.com/test/pr/1".to_string()), Some(123));
        assert_eq!(updated.pr_url, Some("https://github.com/test/pr/1".to_string()));
        assert_eq!(updated.pr_number, Some(123));
        assert!(item.pr_url.is_none()); // Original unchanged
        assert!(updated.updated_at > item.updated_at);
    }

    #[test]
    fn test_item_with_error() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        assert!(item.last_error.is_none());

        let updated = item.clone().with_error(Some("Something went wrong".to_string()));
        assert_eq!(updated.last_error, Some("Something went wrong".to_string()));
        assert!(item.last_error.is_none()); // Original unchanged
        assert!(updated.updated_at > item.updated_at);
    }

    #[test]
    fn test_item_builder_chaining() {
        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        let updated = item
            .clone()
            .with_state(WorkflowState::Implementing)
            .with_branch(Some("feature/test".to_string()))
            .with_error(None);

        assert_eq!(updated.state, WorkflowState::Implementing);
        assert_eq!(updated.branch, Some("feature/test".to_string()));
        assert!(updated.last_error.is_none());
        assert_eq!(item.state, WorkflowState::Idea); // Original unchanged
    }
}
