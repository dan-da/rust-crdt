use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use super::{TreeId, TreeMeta};

/// Represents a Node in a Tree.
/// 
/// Logically, each Node consists of a triple (parent_id, metadata, child_id).
/// However, in this implementation, the child_id is stored as the
/// key in Tree::triples HashMap<ID, TreeNode>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<ID: TreeId, TM: TreeMeta> {
    parent_id: ID,
    metadata: TM,
}

impl<ID: TreeId, TM: TreeMeta> TreeNode<ID, TM> {
    // parent_id: ID,
    // metadata: TM,
    // note: child_id is stored only as a map key in tree.

    /// creates a new TreeNode instance
    pub fn new(parent_id: ID, metadata: TM) -> Self {
        Self {
            parent_id,
            metadata,
        }
    }

    /// returns parent_id reference
    pub fn parent_id(&self) -> &ID {
        &self.parent_id
    }

    /// returns metadata reference
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }
}