
/// TreeMeta trait. TreeMeta are application-defined pieces of data that are stored
/// with each node in the Tree.
pub trait TreeMeta: Clone {}
impl<TM: Clone> TreeMeta for TM {}
