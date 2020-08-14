use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, Ord, PartialOrd, PartialEq, Eq};

use crate::Actor;

/// lamport clock + actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock<A: Actor> {
    actor_id: A,
    counter: u64,
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
