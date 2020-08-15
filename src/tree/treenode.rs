use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use crate::Actor;
use super::TreeMeta;

/// tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<TM: TreeMeta, A: Actor> {
    parent_id: A,
    metadata: TM,
    // note: child_id is stored only as a map key in tree.
}

impl<TM: TreeMeta, A: Actor> TreeNode<TM, A> {
    // parent_id: A,
    // metadata: TM,
    // note: child_id is stored only as a map key in tree.

    /// new
    pub fn new(parent_id: A, metadata: TM) -> Self {
        Self {
            parent_id,
            metadata,
        }
    }

    /// parent_id
    pub fn parent_id(&self) -> &A {
        &self.parent_id
    }

    /// metadata
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }
}
