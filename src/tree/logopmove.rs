use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use crate::Actor;
use super::{TreeId, TreeMeta, TreeNode, OpMove, Clock};

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
pub struct LogOpMove<ID: TreeId, TM: TreeMeta, A:Actor> {
    // todo
    op: OpMove<ID, TM, A>,
    /// previous TreeNode, or None
    oldp: Option<TreeNode<ID, TM>>,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> LogOpMove<ID, TM, A> {
    /// new
    pub fn new(op: OpMove<ID, TM, A>, oldp: Option<TreeNode<ID, TM>>) -> LogOpMove<ID, TM, A> {
        LogOpMove {
            op,
            oldp,
        }
    }

    /// todo
    pub fn timestamp(&self) -> &Clock<A> {
        self.op.timestamp()
    }

    /// todo
    pub fn parent_id(&self) -> &ID {
        self.op.parent_id()
    }

    /// todo
    pub fn metadata(&self) -> &TM {
        self.op.metadata()
    }

    /// todo
    pub fn child_id(&self) -> &ID {
        &self.op.child_id()
    }

    /// todo
    pub fn oldp(&self) -> &Option<TreeNode<ID, TM>> {
        &self.oldp
    }

    /// todo
    pub fn op_into(self) -> OpMove<ID, TM, A> {
        self.op
    }

}