//! Implements TreeMeta, a trait for representing Tree (Node) metadata
//! 
//! For usage/examples, see:
//!   examples/tree.rs
//!   test/tree.rs
//! 
//! This code aims to be an accurate implementation of the
//! tree crdt described in:
//! 
//! "A highly-available move operation for replicated trees 
//! and distributed filesystems" [1] by Martin Klepmann, et al.
//! 
//! [1] https://martin.kleppmann.com/papers/move-op.pdf
//! 
//! For clarity, data structures in this implementation are named
//! the same as in the paper (State, Tree) or close to
//! (OpMove --> Move, LogOpMove --> LogOp).  Some are not explicitly
//! named in the paper, such as TreeId, TreeMeta, TreeNode, Clock.

/// TreeMeta trait. TreeMeta are application-defined pieces of data that are stored
/// with each node in the Tree.
pub trait TreeMeta: Clone {}
impl<TM: Clone> TreeMeta for TM {}
