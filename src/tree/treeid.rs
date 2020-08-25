use std::hash::Hash;

/// TreeId trait. TreeId are unique identifiers for each node in a tree.
pub trait TreeId: Eq + Clone + Hash {}
impl<ID: Eq + Clone + Hash> TreeId for ID {}