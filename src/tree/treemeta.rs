use serde::{Serialize};
use std::cmp::{PartialEq, Eq};

/// treemeta trait
pub trait TreeMeta: Serialize + PartialEq + Eq + Clone {}
impl<TM: Serialize + PartialEq + Eq + Clone> TreeMeta for TM {}
