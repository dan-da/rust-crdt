/// Contains the implementation of a crdt-tree

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cmp::{Ordering, Ord, PartialOrd, PartialEq, Eq};

use crate::{Actor, CmRDT};

/// treemeta trait
pub trait TreeMeta: Serialize + PartialEq + Eq + Clone + std::fmt::Debug {}
impl<TM: Serialize + PartialEq + Eq + Clone + std::fmt::Debug> TreeMeta for TM {}


/// lamport clock + actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock<A: Actor> {
    actor_id: A,
    counter: u64,
}

/// tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<TM: TreeMeta, A: Actor> {
    parent_id: A,
    metadata: TM,
    // note: child_id is stored only as a map key in tree.
}

/// tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree<TM: TreeMeta, A: Actor> {
    triples: HashMap<A, TreeNode<TM, A>>,   // tree_nodes, indexed by child_id.
    children: HashMap<A, HashMap<A, bool>>,  // parent_id => [child_id => true].  optimization.
}

/// At time $timestamp, $child_id is moved to be a child of $parent_id.
/// Old location doesn't matter.
/// If child_id does not exist, it is created.
///
/// In a filesystem, parent and child are inodes of a directory
/// and and file within it, respectively.  The metadata contains
/// the filename of the child.  Thus a file with inode c can be renamed
/// by performing a Move t p m c, where the new parent directory p is
/// the inode of the existing parent (unchanged), but the metadata
/// m contains the new filename.
///
/// When users want to make changes to the tree on their local replica
/// they generate new Move t p m c operations for these changes, and
/// apply these operations using the algorithm described in the rest of
/// this section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpMove<TM: TreeMeta, A:Actor> {
    /// lamport clock + actor
    pub timestamp: Clock<A>,
    /// parent identifier
    pub parent_id: A,
    /// metadata
    pub metadata: TM,
    /// child identifier
    pub child_id: A,
}

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

/// State
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State<TM: TreeMeta, A:Actor> {
    log_op_list: Vec<LogOpMove<TM, A>>,  // a list of LogMove in descending timestamp order.
    /// tree
    tree: Tree<TM, A>,
}


impl<A: Actor> Clock<A> {
//    actor_id: A,
//    counter: u64,

    /// new
    pub fn new(actor_id: A, counter: Option<u64>) -> Self {
        Self {
            actor_id,
            counter: counter.unwrap_or(0),
        }
    }

    /// returns a new la_time with same actor but counter incremented by 1.
    pub fn inc(&self) -> Self {
        Self::new(self.actor_id.clone(), Some(self.counter + 1))
    }

    /// actor_id
    pub fn actor_id(&self) -> &A {
        return &self.actor_id;
    }

    /// returns a new la_time with same actor but counter is
    /// max(this_counter, other_counter)
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.actor_id.clone(), Some(std::cmp::max(self.counter, other.counter)))
    }
}

impl<A: Actor> Ord for Clock<A> {

    /// compares this la_time with another.
    /// if counters are unequal, returns -1 or 1 accordingly.
    /// if counters are equal, returns -1, 0, or 1 based on actor_id.
    ///    (this is arbitrary, but deterministic.)
    fn cmp(&self, other: &Self) -> Ordering {
        if self.counter == other.counter {
            if self.actor_id < other.actor_id {
                return Ordering::Less;
            }
            else if self.actor_id > other.actor_id {
                return Ordering::Greater;
            }
            else {
                return Ordering::Equal;
            }
        }
        else if self.counter > other.counter {
            return Ordering::Greater;
        }
        else { // self.counter < other.counter
            return Ordering::Less;
        }
    }
}

impl<A: Actor> PartialOrd for Clock<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Actor> PartialEq for Clock<A> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<A: Actor> Eq for Clock<A> {}


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


impl<TM: TreeMeta, A: Actor> Tree<TM, A> {
    // triples: HashMap<A, TreeNode<T, A>>,   // tree_nodes, indexed by child_id.
    // children: HashMap<A, HashMap<A, bool>>,  // parent_id => [child_id => true].  optimization.

    /// new 
    pub fn new() -> Self {
        Self {
            triples: HashMap::<A, TreeNode<TM, A>>::new(),   // tree_nodes, indexed by child_id.
            children: HashMap::<A, HashMap<A, bool>>::new(),  // parent_id => [child_id => true].  optimization.
        }
    }

    /// helper for removing a triple based on child_id
    pub fn rm_child(&mut self, child_id: &A) {
        let result = self.triples.get(child_id);
        if let Some(t) = result {
            if let Some(map) = self.children.get_mut(&t.parent_id) {
                map.remove(child_id);
                // cleanup parent entry if empty.
                if map.len() == 0 {
                    self.children.remove(&t.parent_id);
                }
            }
            self.triples.remove(child_id);
        }
    }

    /// removes a subtree.  useful for emptying trash.
    /// not used by crdt algo.
    pub fn rm_subtree(&mut self, parent_id: &A, include_parent: bool) {
        for c in self.children(parent_id) {
            self.rm_subtree(&c, false);
            self.rm_child(&c);
        }
        if include_parent {
            self.rm_child(parent_id)
        }
    }

    /// adds a node to the tree
    pub fn add_node(&mut self, child_id: A, tt: TreeNode<TM,A>) {
        if let Some(n) = self.children.get_mut(&tt.parent_id) {
            n.insert(child_id.clone(), true);
        } else {
            let mut h: HashMap<A, bool> = HashMap::new();
            h.insert(child_id.clone(), true);
            self.children.insert(tt.parent_id.clone(), h);
        }
        self.triples.insert(child_id, tt);
    }

    /// returns matching node, or None.
    pub fn find_mut(&mut self, child_id: &A) -> Option<&mut TreeNode<TM,A>> {
        self.triples.get_mut(child_id)
    }

    /// returns matching node, or None.
    pub fn find(&self, child_id: &A) -> Option<&TreeNode<TM,A>> {
        self.triples.get(child_id)
    }


    /// returns children (IDs) of a given parent node.
    /// useful for walking tree.
    /// not used by crdt algo.
    pub fn children(&self, parent_id: &A) -> Vec<A> {
        if let Some(list) = self.children.get(parent_id) {
            let l: Vec<A> = list.keys().cloned().collect();
            return l;
        }
        Vec::<A>::default()
    }

    /// walks tree and calls callback fn for each node.
    /// not used by crdt algo.
    fn walk_worker<F>(&self, parent_id: &A, f: &F, depth: usize) 
        where F: Fn(&Self, &A, usize) {

        f(self, parent_id, depth);
        let children = self.children(parent_id);
        for c in children {
            self.walk_worker(&c, f, depth+1);
        }
    }

    /// walks tree and calls callback fn for each node.
    /// not used by crdt algo.
    pub fn walk<F>(&self, parent_id: &A, f: &F) 
        where F: Fn(&Self, &A, usize) {
        self.walk_worker(parent_id, f, 0)
    }

    /// parent | child
    /// --------------
    /// 1        2
    /// 1        3
    /// 3        5
    /// 2        6
    /// 6        8

    ///                  1
    ///               2     3
    ///             6         5
    ///           8
    ///
    /// is 2 ancestor of 8?  yes.
    /// is 2 ancestor of 5?   no.

    /// determines if ancestor_id is an ancestor of node_id in tree.
    /// returns bool
    pub fn is_ancestor(&self, child_id: &A, ancestor_id: &A) -> bool {
        let mut target_id = child_id;
        loop {
            if let Some(n) = self.find(&target_id) {
                if &n.parent_id == ancestor_id {
                    return true;
                }
                target_id = &n.parent_id;
            } else {
                break;
            }
        }
        false
    }
}

impl<TM: TreeMeta, A: Actor> OpMove<TM, A> {

    /// new
    pub fn new(timestamp: Clock<A>, parent_id: A, metadata: TM, child_id: A) -> Self {
        Self {
            timestamp,
            parent_id,
            metadata,
            child_id,
        }
    }

    /// from_log_op_move
    pub fn from_log_op_move(l: &LogOpMove<TM, A>) -> Self {
        Self {
            timestamp: l.timestamp.clone(),
            parent_id: l.parent_id.clone(),
            metadata: l.metadata.clone(),
            child_id: l.child_id.clone(),
        }
    }
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
            let tn = TreeNode::new(oldp.parent_id.clone(), oldp.metadata.clone());
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
                #[cfg(debug_assertions)]
                panic!("applying op with timestamp equal to previous op.  Every op should have a unique timestamp.");

                // Production code should just treat it as a non-op.
                #[cfg(not(debug_assertions))]
                return state;
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
}

impl<TM: TreeMeta, A: Actor> CmRDT for State<TM, A> {
    type Op = OpMove<TM, A>;

    /// Apply an operation to a State instance.
    fn apply(&mut self, op: Self::Op) {
        self.apply_op(op);
    }
}
