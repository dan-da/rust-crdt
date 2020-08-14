//! A pure-Rust library of thoroughly-tested, serializable CRDT's.
//!
//! [Conflict-free Replicated Data Types][crdt] (CRDTs) are data structures
//! which can be replicated across multiple networked nodes, and whose
//! properties allow for deterministic, local resolution of
//! possible inconsistencies which might result from concurrent
//! operations.
//!
//! [crdt]: https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type
#![deny(missing_docs)]

/// This module contains a Tree.
pub mod tree;

/// This module contains a Clock.
pub mod clock;

/// This module contains OpMove.
pub mod opmove;

/// This module contains LogOpMove.
pub mod logopmove;

/// This module contains TreeMeta.
pub mod treemeta;

/// This module contains TreeNode.
pub mod treenode;