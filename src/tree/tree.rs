use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};

use super::{TreeId, TreeMeta, TreeNode};

/// tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree<ID: TreeId, TM: TreeMeta> {
    triples: HashMap<ID, TreeNode<ID, TM>>,   // tree_nodes, indexed by child_id.
    children: HashMap<ID, HashMap<ID, bool>>,  // parent_id => [child_id => true].  optimization.
}

impl<ID: TreeId, TM: TreeMeta> Tree<ID, TM> {

    /// new 
    pub fn new() -> Self {
        Self {
            triples: HashMap::<ID, TreeNode<ID, TM>>::new(),   // tree_nodes, indexed by child_id.
            children: HashMap::<ID, HashMap<ID, bool>>::new(),  // parent_id => [child_id => true].  optimization.
        }
    }

    /// helper for removing a triple based on child_id
    pub fn rm_child(&mut self, child_id: &ID) {
        let result = self.triples.get(child_id);
        if let Some(t) = result {
            if let Some(map) = self.children.get_mut(t.parent_id()) {
                map.remove(child_id);
                // cleanup parent entry if empty.
                if map.len() == 0 {
                    self.children.remove(t.parent_id());
                }
            }
            self.triples.remove(child_id);
        }
    }

    /// removes a subtree.  useful for emptying trash.
    /// not used by crdt algo.
    pub fn rm_subtree(&mut self, parent_id: &ID, include_parent: bool) {
        for c in self.children(parent_id) {
            self.rm_subtree(&c, false);
            self.rm_child(&c);
        }
        if include_parent {
            self.rm_child(parent_id)
        }
    }

    /// adds a node to the tree
    pub fn add_node(&mut self, child_id: ID, tt: TreeNode<ID, TM>) {
        if let Some(n) = self.children.get_mut(tt.parent_id()) {
            n.insert(child_id.clone(), true);
        } else {
            let mut h: HashMap<ID, bool> = HashMap::new();
            h.insert(child_id.clone(), true);
            self.children.insert(tt.parent_id().clone(), h);
        }
        self.triples.insert(child_id, tt);
    }

    /// returns matching node, or None.
    pub fn find(&self, child_id: &ID) -> Option<&TreeNode<ID, TM>> {
        self.triples.get(child_id)
    }

    /// returns children (IDs) of a given parent node.
    /// useful for walking tree.
    /// not used by crdt algo.
    pub fn children(&self, parent_id: &ID) -> Vec<ID> {
        if let Some(list) = self.children.get(parent_id) {
            list.keys().cloned().collect()
        } else {
            Vec::<ID>::default()
        }
    }

    /// walks tree and calls callback fn for each node.
    /// not used by crdt algo.
    fn walk_worker<F>(&self, parent_id: &ID, f: &F, depth: usize) 
        where F: Fn(&Self, &ID, usize) {

        f(self, parent_id, depth);
        let children = self.children(parent_id);
        for c in children {
            self.walk_worker(&c, f, depth+1);
        }
    }

    /// walks tree and calls callback fn for each node.
    /// not used by crdt algo.
    pub fn walk<F>(&self, parent_id: &ID, f: &F) 
        where F: Fn(&Self, &ID, usize) {
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
    pub fn is_ancestor(&self, child_id: &ID, ancestor_id: &ID) -> bool {
        let mut target_id = child_id;
        loop {
            if let Some(n) = self.find(&target_id) {
                if n.parent_id() == ancestor_id {
                    return true;
                }
                target_id = n.parent_id();
            } else {
                break;
            }
        }
        false
    }
}
