use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use super::{TreeId, TreeMeta};

/// tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<ID: TreeId, TM: TreeMeta> {
    parent_id: ID,
    metadata: TM,
    // note: child_id is stored only as a map key in tree.
}

impl<ID: TreeId, TM: TreeMeta> TreeNode<ID, TM> {
    // parent_id: ID,
    // metadata: TM,
    // note: child_id is stored only as a map key in tree.

    /// new
    pub fn new(parent_id: ID, metadata: TM) -> Self {
        Self {
            parent_id,
            metadata,
        }
    }

    /// parent_id
    pub fn parent_id(&self) -> &ID {
        &self.parent_id
    }

    /// metadata
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }
}