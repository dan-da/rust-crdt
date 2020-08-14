use serde::{Serialize};
use std::cmp::{PartialEq, Eq};

/// treemeta trait
pub trait TreeMeta: Serialize + PartialEq + Eq + Clone + std::fmt::Debug {}
impl<TM: Serialize + PartialEq + Eq + Clone + std::fmt::Debug> TreeMeta for TM {}
