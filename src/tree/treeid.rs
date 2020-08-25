/// Implements TreeId, a trait for representing Tree (Node) Identifiers
/// 
/// For usage/examples, see:
///   examples/tree.rs
///   test/tree.rs
/// 
/// This code aims to be an accurate implementation of the
/// tree crdt described in:
/// 
/// "A highly-available move operation for replicated trees 
/// and distributed filesystems" [1] by Martin Klepmann, et al.
/// 
/// [1] https://martin.kleppmann.com/papers/move-op.pdf
/// 
/// For clarity, data structures in this implementation are named
/// the same as in the paper (State, Tree) or close to
/// (OpMove --> Move, LogOpMove --> LogOp).  Some are not explicitly
/// named in the paper, such as TreeId, TreeMeta, TreeNode, Clock.

use std::hash::Hash;

/// TreeId trait. TreeId are unique identifiers for each node in a tree.
pub trait TreeId: Eq + Clone + Hash {}
impl<ID: Eq + Clone + Hash> TreeId for ID {}