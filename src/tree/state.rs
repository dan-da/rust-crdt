/// Contains the implementation of a crdt-tree

use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use crate::{Actor, CmRDT};
use super::{TreeMeta, TreeNode, OpMove, LogOpMove, Tree, Clock};

/// State
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State<TM: TreeMeta, A:Actor> {
    log_op_list: Vec<LogOpMove<TM, A>>,  // a list of LogMove in descending timestamp order.
    /// tree
    tree: Tree<TM, A>,
}

impl<TM: TreeMeta, A: Actor> State<TM, A> {

    /// new
    pub fn new() -> Self {
        Self {
            log_op_list: Vec::<LogOpMove<TM, A>>::default(),
            tree: Tree::<TM, A>::new(),
        }
    }

    /// from_existing
    pub fn from_existing(log_op_list: Vec<LogOpMove<TM, A>>, tree: Tree<TM, A>) -> Self {
        Self {
            log_op_list,
            tree,
        }
    }

    /// tree
    pub fn tree(&self) -> &Tree<TM, A> {
        &self.tree
    }

    /// mutable tree reference
    pub fn tree_mut(&mut self) -> &mut Tree<TM, A> {
        &mut self.tree
    }

    /// log
    pub fn log(&self) -> &Vec<LogOpMove<TM, A>> {
        &self.log_op_list
    }

    /// add_log_entry
    pub fn add_log_entry(&mut self, entry: LogOpMove<TM, A>) {
        // add at beginning of array
        self.log_op_list.insert(0, entry);
    }

    /// removes log entries before a given timestamp.
    /// not part of crdt-tree algo.
    pub fn truncate_log_before(&mut self, timestamp: &Clock<A>) -> bool {

        // newest entries are at start of list, so to find
        // oldest entries we iterate from the end towards start.
        let len = self.log_op_list.len();
        let mut last_idx: usize = len - 1;
        for (i, v) in self.log_op_list.iter().rev().enumerate() {
            if v.timestamp < *timestamp {
                last_idx = len - 1 - i;
            } else {
                break;
            }
        }

        loop {
            let idx = self.log_op_list.len() - 1;
            if idx < last_idx {
                break;
            }
            self.log_op_list.remove(idx);
        }

        last_idx + 1 < len
    }

    /// for testing. not part of crdt-tree algo.
    pub fn check_log_is_descending(&self) {
        let mut i = 0;
        while i < self.log_op_list.len()-1 {
            let first = &self.log_op_list[i];
            let second = &self.log_op_list[i+1];

            if !(first.timestamp > second.timestamp) {
                panic!("Log not in descending timestamp order!");
            }
            i += 1;
        }
    }

    /// The do_op function performs the actual work of applying
    /// a move operation.
    ///
    /// This function takes as argument a pair consisting of a 
    /// Move operation and the current tree and it returns a pair
    /// consisting of a LogMove operation (which will be added to the log) and
    /// an updated tree.
    pub fn do_op(&mut self, op: OpMove<TM, A>) -> LogOpMove<TM, A> {

        // When a replica applies a Move op to its tree, it also records
        // a corresponding LogMove op in its log.  The t, p, m, and c
        // fields are taken directly from the Move record, while the oldp
        // field is filled in based on the state of the tree before the move.
        // If c did not exist in the tree, oldp is set to None.  Otherwise
        // oldp records the previous parent and metadata of c.
        let oldp = self.tree.find(&op.child_id);
        let log = LogOpMove::new(&op, oldp.cloned());

        // ensures no cycles are introduced.  If the node c
        // is being moved, and c is an ancestor of the new parent
        // newp, then the tree is returned unmodified, ie the operation
        // is ignored.
        // Similarly, the operation is also ignored if c == newp
        if op.child_id == op.parent_id ||
        self.tree.is_ancestor(&op.parent_id, &op.child_id) {
            return log;
        }

        // Otherwise, the tree is updated by removing c from
        // its existing parent, if any, and adding the new
        // parent-child relationship (newp, m, c) to the tree.
        self.tree.rm_child(&op.child_id);
        let tt = TreeNode::new(op.parent_id, op.metadata);
        self.tree.add_node(op.child_id, tt);
        log
    }

    /// undo_op
    pub fn undo_op(&mut self, log: &LogOpMove<TM, A>) {
        self.tree.rm_child(&log.child_id);

        if let Some(oldp) = &log.oldp {
            let tn = TreeNode::new(oldp.parent_id().clone(), oldp.metadata().clone());
            self.tree.add_node(log.child_id.clone(), tn);
        } 
    }

    /// redo_op uses do_op to perform an operation
    /// again and recomputes the LogMove record (which
    /// might have changed due to the effect of the new operation)
    pub fn redo_op(&mut self, logop: &LogOpMove<TM, A>) {
        let op = OpMove::from_log_op_move(logop);
        let logop2 = self.do_op(op);

        self.add_log_entry(logop2);
    }

    /// See general description of apply/undo/redo above.
    ///
    /// The apply_op func takes two arguments:
    /// a Move operation to apply and the current replica
    /// state; and it returns the new replica state.
    /// The constrains `t::{linorder} in the type signature
    /// indicates that timestamps `t are instance if linorder
    /// type class, and they can therefore be compared with the
    /// < operator during a linear (or total) order.
    pub fn apply_op(&mut self, op1: OpMove<TM, A>) {
        if self.log_op_list.len() == 0 {
            let op2 = self.do_op(op1);
            self.log_op_list = vec![op2];
        } else {
            if op1.timestamp == self.log_op_list[0].timestamp {
                // This case should never happen in normal operation
                // because it is required that all timestamps are unique.
                // The crdt paper does not even check for this case.
                //
                // We throw an exception to catch it during dev/test.
                // #[cfg(debug_assertions)]
                // panic!("applying op with timestamp equal to previous op.  Every op should have a unique timestamp.");

                // Production code should just treat it as a non-op.
                // #[cfg(not(debug_assertions))]
            } else if op1.timestamp < self.log_op_list[0].timestamp {
                let logop = self.log_op_list.remove(0);  // take from beginning of array
                self.undo_op(&logop);
                self.apply_op(op1);
                self.redo_op(&logop);
            } else {
                let op2 = self.do_op(op1);
                self.add_log_entry(op2);
            }
        }
    }

    /// todo
    pub fn apply_ops_into(&mut self, ops: Vec<OpMove<TM, A>>) {
        for op in ops {
            self.apply_op(op);
        }
    }    

    /// todo
    pub fn apply_ops(&mut self, ops: &Vec<OpMove<TM, A>>) {
        self.apply_ops_into(ops.clone())
    }

}

impl<TM: TreeMeta, A: Actor> CmRDT for State<TM, A> {
    type Op = OpMove<TM, A>;

    /// Apply an operation to a State instance.
    fn apply(&mut self, op: Self::Op) {
        self.apply_op(op);
    }
}

// See <root>/test/tree.rs for tests