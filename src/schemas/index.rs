//! Index schema - Optional item index cache

use serde::{Deserialize, Serialize};

use super::WorkflowState;

/// An entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexItem {
    /// Item ID
    pub id: String,

    /// Current workflow state
    pub state: WorkflowState,

    /// Item title
    pub title: String,
}

/// Index of all items (optional cache)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Schema version for forward compatibility
    pub schema_version: u32,

    /// List of index entries
    pub items: Vec<IndexItem>,

    /// ISO 8601 timestamp when index was generated
    pub generated_at: String,
}

impl Index {
    /// Create a new empty index
    pub fn new() -> Self {
        Index {
            schema_version: 1,
            items: Vec::new(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_item_serialization() {
        let item = IndexItem {
            id: "test-001".to_string(),
            state: WorkflowState::Idea,
            title: "Test Item".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: IndexItem = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "test-001");
        assert_eq!(parsed.state, WorkflowState::Idea);
        assert_eq!(parsed.title, "Test Item");
    }

    #[test]
    fn test_index_round_trip() {
        let mut index = Index::new();
        index.items.push(IndexItem {
            id: "test-001".to_string(),
            state: WorkflowState::Idea,
            title: "Test Item 1".to_string(),
        });
        index.items.push(IndexItem {
            id: "test-002".to_string(),
            state: WorkflowState::Done,
            title: "Test Item 2".to_string(),
        });

        let json = serde_json::to_string_pretty(&index).unwrap();
        let parsed: Index = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.items[0].id, "test-001");
        assert_eq!(parsed.items[1].state, WorkflowState::Done);
    }
}
