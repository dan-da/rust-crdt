use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};

use crate::Actor;
use super::{TreeId, TreeMeta, LogOpMove, Clock};
use crate::quickcheck::{Arbitrary, Gen};

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
pub struct OpMove<ID: TreeId, TM: TreeMeta, A:Actor> {
    /// lamport clock + actor
    timestamp: Clock<A>,
    /// parent identifier
    parent_id: ID,
    /// metadata
    metadata: TM,
    /// child identifier
    child_id: ID,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> OpMove<ID, TM, A> {

    /// new
    pub fn new(timestamp: Clock<A>, parent_id: ID, metadata: TM, child_id: ID) -> Self {
        Self {
            timestamp,
            parent_id,
            metadata,
            child_id,
        }
    }

    /// todo
    pub fn timestamp(&self) -> &Clock<A> {
        &self.timestamp
    }

    /// todo
    pub fn parent_id(&self) -> &ID {
        &self.parent_id
    }

    /// todo
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }

    /// todo
    pub fn child_id(&self) -> &ID {
        &self.child_id
    }

    /// from_log_op_move
    pub fn from_log_op_move(l: LogOpMove<ID, TM, A>) -> Self {
        Self {
            timestamp: l.timestamp().to_owned(),
            parent_id: l.parent_id().to_owned(),
            metadata: l.metadata().to_owned(),
            child_id: l.child_id().to_owned(),
        }
    }
}


impl<ID: TreeId + Arbitrary, A: Actor + Arbitrary, TM: TreeMeta + Arbitrary> Arbitrary for OpMove<ID, TM, A> {

    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self::new(Clock::arbitrary(g),
                  ID::arbitrary(g),
                  TM::arbitrary(g),
                  ID::arbitrary(g)
        )
    }
}

