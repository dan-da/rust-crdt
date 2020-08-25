use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, Ord, PartialOrd, PartialEq, Eq};
use crate::quickcheck::{Arbitrary, Gen};

use crate::Actor;

/// lamport clock + actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock<A: Actor> {
    actor_id: A,
    counter: u64,
}

impl<A: Actor> Clock<A> {

    /// create new Clock instance
    /// 
    /// typically counter should be None
    pub fn new(actor_id: A, counter: Option<u64>) -> Self {
        Self {
            actor_id,
            counter: counter.unwrap_or(0),
        }
    }

    /// returns a new Clock with same actor but counter incremented by 1.
    pub fn inc(&self) -> Self {
        Self::new(self.actor_id.clone(), Some(self.counter + 1))
    }

    /// increments clock counter and returns a clone
    pub fn tick(&mut self) -> Self {
        self.counter += 1;
        self.clone()
    }

    /// returns actor_id reference
    pub fn actor_id(&self) -> &A {
        return &self.actor_id;
    }

    /// returns a new Clock with same actor but counter is
    /// max(this_counter, other_counter)
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.actor_id.clone(), Some(std::cmp::max(self.counter, other.counter)))
    }
}

impl<A: Actor> Ord for Clock<A> {

    /// compares this Clock with another.
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

// Generate arbitrary (random) clocks.  needed by quickcheck.
impl<A: Actor + Arbitrary> Arbitrary for Clock<A> {

    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self {
            actor_id: A::arbitrary(g),
            counter: u64::arbitrary(g),
        }
    }    

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut shrunk_clocks = Vec::new();
        if self.counter > 0 {
            shrunk_clocks.push(Self::new(self.actor_id.clone(), Some(self.counter - 1)));
        }
        Box::new(shrunk_clocks.into_iter())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn inc_increments_only_the_counter(clock: Clock<u8>) -> bool {
            clock.inc() == Clock::new(clock.actor_id, Some(clock.counter + 1))
        }

        fn test_total_order(a: Clock<u8>, b: Clock<u8>) -> bool {
            let cmp_ab = a.cmp(&b);
            let cmp_ba = b.cmp(&a);

            match (cmp_ab, cmp_ba) {
                (Ordering::Less, Ordering::Greater) => a.counter < b.counter || a.counter == b.counter && a.actor_id < b.actor_id,
                (Ordering::Greater, Ordering::Less) => a.counter > b.counter || a.counter == b.counter && a.actor_id > b.actor_id,
                (Ordering::Equal, Ordering::Equal) => a.actor_id == b.actor_id && a.counter == b.counter,
                _ => false,
            }
        }

        fn test_partial_order(a: Clock<u8>, b: Clock<u8>) -> bool {
            let cmp_ab = a.partial_cmp(&b);
            let cmp_ba = b.partial_cmp(&a);

            match (cmp_ab, cmp_ba) {
                (None, None) => a.actor_id != b.actor_id,
                (Some(Ordering::Less), Some(Ordering::Greater)) => a.counter < b.counter || a.counter == b.counter && a.actor_id < b.actor_id,
                (Some(Ordering::Greater), Some(Ordering::Less)) => a.counter > b.counter || a.counter == b.counter && a.actor_id > b.actor_id,
                (Some(Ordering::Equal), Some(Ordering::Equal)) => a.actor_id == b.actor_id && a.counter == b.counter,
                _ => false
            }
        }
    }
}