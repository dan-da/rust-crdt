/// Contains the implementation of a crdt-tree

use crdts::Actor;
use crdts::tree::{Clock, State, OpMove};
use quickcheck::{Arbitrary, Gen, TestResult};
use rand::Rng;

//use crdts::CvRDT;
//use rand::distributions::Alphanumeric;

//use crate::{Actor, CmRDT};
//use super::{TreeMeta, TreeNode, OpMove, LogOpMove, Tree, Clock};

type TActor = u8;
type TMeta = char;

#[derive(Debug, Clone)]
struct OperationList(pub Vec<OpMove<TMeta, TActor>>);

impl Iterator for OperationList {
    type Item = OpMove<TMeta, TActor>;
    fn next(&mut self) -> Option<OpMove<TMeta, TActor>> {
        self.0.iter().next().cloned()
    }    
}

impl Arbitrary for OperationList {
    fn arbitrary<G: Gen>(g: &mut G) -> OperationList {
        let size = {
            let s = g.size();
            if s == 0 {
                0
            } else {
                g.gen_range(0, s)
            }
        };

        let mut clock = Clock::arbitrary(g);
        let mut nodes: Vec::<TActor> = Vec::new();
        let mut parent_id = TActor::arbitrary(g);

        let mut ops: Vec<OpMove<TMeta, TActor>> = Vec::new();
        for _ in 0..size {
            let next_id = TActor::arbitrary(g);
            nodes.push(next_id.clone());
            let meta = TMeta::arbitrary(g);

            let op = OpMove::new(tick(&mut clock), parent_id, meta, next_id);
            let idx: usize = rand::random::<usize>() % nodes.len();
            parent_id = nodes[idx];

            ops.push(op);
        }
        Self(ops)
    }

/*    
    // implement shrinking ://
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
//        let mut c = self.0.clone();
//        c.pop();
//        let x = OperationList(c);
//        Box::new(vec![x].into_iter())

        let mut shrunk_ops = Vec::new();
//        println!("{:#?}", self.0);
        

        let mut parents: HashMap::<TActor, bool> = HashMap::new();
        for op in &self.0 {
            parents.insert(op.parent_id, true);
        }

        for op in self.0.clone() {
            if parents.get(&op.child_id).is_none() {
                let leaf_id = op.child_id;

                for op in &self.0 {
                    if op.child_id != leaf_id {
                        shrunk_ops.push(op.clone());
                    }
                }
                break;
            }
        }
        let x = OperationList(shrunk_ops);
        Box::new(vec![x].into_iter())
    }
*/    
}


//    use super::*;
//    extern crate rand;
//    use quickcheck::{quickcheck, TestResult};

    fn new_id() -> u64 {
        rand::random::<u64>()
    }

    fn tick<A: Actor>(clock: &mut Clock<A>) -> Clock<A> {
        *clock = clock.inc();
        clock.clone()
    }

    #[test]
    fn test_concurrent_moves() {
        let mut r1: State<&str, u64> = State::new();
        let mut r2: State<&str, u64> = State::new();

        let (r1_id, r2_id) = (new_id(), new_id());
        let mut r1t = Clock::<u64>::new(r1_id, None);
        let mut r2t = Clock::<u64>::new(r2_id, None);

        let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

        // Setup initial tree state.
        let ops = vec![OpMove::new(tick(&mut r1t), 0, "root", root_id),
                       OpMove::new(tick(&mut r1t), root_id, "a", a_id),
                       OpMove::new(tick(&mut r1t), root_id, "b", b_id),
                       OpMove::new(tick(&mut r1t), root_id, "c", c_id),
        ];

        for op in ops {
            r1.apply_op(op.clone());
            r2.apply_op(op);        
        }

        // replica_1 moves /root/a to /root/b
        let r1_op = OpMove::new(tick(&mut r1t), b_id, "a", a_id);
        // replica_2 "simultaneously" moves /root/a to /root/c
        let r2_op = OpMove::new(tick(&mut r2t), c_id, "a", a_id);

        // apply both ops to r1
        r1.apply_op(r1_op.clone());
        r1.apply_op(r2_op.clone());

        // apply both ops to r2
        r2.apply_op(r2_op);
        r2.apply_op(r1_op);

        assert_eq!(r1, r2);
    }

    #[test]
    fn test_concurrent_moves_cycle() {
        let mut r1: State<&str, u64> = State::new();
        let mut r2: State<&str, u64> = State::new();

        let (r1_id, r2_id) = (new_id(), new_id());
        let mut r1t = Clock::<u64>::new(r1_id, None);
        let mut r2t = Clock::<u64>::new(r2_id, None);

        let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

        // Setup initial tree state.
        let ops = vec![OpMove::new(tick(&mut r1t), 0, "root", root_id),
                       OpMove::new(tick(&mut r1t), root_id, "a", a_id),
                       OpMove::new(tick(&mut r1t), root_id, "b", b_id),
                       OpMove::new(tick(&mut r1t), a_id, "c", c_id),
        ];

        for op in ops {
            r1.apply_op(op.clone());
            r2.apply_op(op);        
        }

        // replica_1 moves /root/b to /root/a
        let r1_op = OpMove::new(tick(&mut r1t), a_id, "b", b_id);
        // replica_2 "simultaneously" moves /root/a to /root/b
        let r2_op = OpMove::new(tick(&mut r2t), b_id, "a", a_id);

        // apply both ops to r1
        r1.apply_op(r1_op.clone());
        r1.apply_op(r2_op.clone());

        // apply both ops to r2
        r2.apply_op(r2_op);
        r2.apply_op(r1_op);

        assert_eq!(r1, r2);
    }

    fn state_from_ops(ops: &OperationList) -> State<TMeta, TActor> {
        let mut s: State<TMeta, TActor> = State::new();
        for op in ops.0.iter().cloned() {
            s.apply_op(op);
        }
        s
    }

    quickcheck! {
        fn prop_idempotent(ops: OperationList) -> TestResult {
            let r1 = state_from_ops(&ops);
            let r2 = state_from_ops(&ops);

            // r ^ r = r
            TestResult::from_bool(r1 == r2)
        }

        fn prop_commutative(ops1: OperationList, ops2: OperationList) -> TestResult {
            
            if ops1.0.len() > 0 && ops2.0.len() > 0 &&
               ops1.0[0].timestamp.actor_id() == ops2.0[0].timestamp.actor_id() {
                return TestResult::discard();
            }

            let mut r1 = state_from_ops(&ops1);
            r1.apply_ops(&ops2.0);

            let mut r2 = state_from_ops(&ops2);
            r2.apply_ops(&ops1.0);

            TestResult::from_bool(r1 == r2)
        }
        
        fn prop_associative(
            ops1: OperationList, 
            ops2: OperationList,
            ops3: OperationList
        ) -> TestResult {

            // discard if ops1 actor is same as ops2 actor
            if ops1.0.len() > 0 && ops2.0.len() > 0 && 
               ops1.0[0].timestamp.actor_id() == ops2.0[0].timestamp.actor_id() {
                return TestResult::discard();
            }

            // discard if ops1 actor is same as ops3 actor
            if ops1.0.len() > 0 && ops3.0.len() > 0 && 
               ops1.0[0].timestamp.actor_id() == ops3.0[0].timestamp.actor_id() {
                return TestResult::discard();
            }

            // discard if ops2 actor is same as ops3 actor
            if ops2.0.len() > 0 && ops3.0.len() > 0 && 
               ops2.0[0].timestamp.actor_id() == ops3.0[0].timestamp.actor_id() {
                return TestResult::discard();
            }

            let mut r1 = state_from_ops(&ops1);
            let mut r2 = state_from_ops(&ops2);

            // r1 <- r2
            r1.apply_ops(&ops2.0);

            // (r1 <- r2) <- r3
            r1.apply_ops(&ops3.0);

            // r2 <- r3
            r2.apply_ops(&ops3.0);

            // (r2 <- r3) <- r1
            r2.apply_ops(&ops1.0);

            TestResult::from_bool(r1 == r2)
        }
        
    }
