use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use crate::Actor;
use super::{TreeMeta, TreeNode, OpMove, Clock};

/// When a replica applies a Move operation to its tree it
/// also records a corresponding LogMove operation in its log.
/// The t, p, m, and c fields are taken directly from the Move
/// record while the oldp field is filled in based on the
/// state of the tree before the move.  If c did not exist
/// in the tree, oldp is set to None. Else oldp records the
/// previous parent metadata of c: if there exist p' and m'
/// such that (p', m', c') E tree, then oldp is set to Some(p', m').
/// The get_parent() function implements this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogOpMove<TM: TreeMeta, A:Actor> {
    /// lamport clock + actor
    pub timestamp: Clock<A>,
    /// parent identifier
    pub parent_id: A,
    /// metadata
    pub metadata: TM,
    /// child identifier
    pub child_id: A,
    /// previous TreeNode, or None
    pub oldp: Option<TreeNode<TM, A>>,
}

impl<TM: TreeMeta, A: Actor> LogOpMove<TM, A> {
    /// new
    pub fn new(op: &OpMove<TM, A>, oldp: Option<TreeNode<TM, A>>) -> LogOpMove<TM, A> {
        LogOpMove {
            timestamp: op.timestamp.clone(),
            parent_id: op.parent_id.clone(),
            metadata: op.metadata.clone(),
            child_id: op.child_id.clone(),
            oldp,
        }
    }
}