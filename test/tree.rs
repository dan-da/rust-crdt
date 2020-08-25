/// tests for crdt-tree

use crdts::tree::{Clock, State, OpMove};
use quickcheck::{Arbitrary, Gen, TestResult};
use rand::Rng;
use std::collections::HashMap;

// Define some "real" types for use in the tests.
type TypeId = u8;
type TypeActor = u8;
type TypeMeta = char;             // used by quickcheck tests
type TypeMetaStr<'a> = &'a str;   // used by manual tests

// A list of quasi-random operations for use by quickcheck.
#[derive(Debug, Clone)]
struct OperationList {
    pub ops: Vec<OpMove<TypeId, TypeMeta, TypeActor>>
}

impl Iterator for OperationList {
    type Item = OpMove<TypeId, TypeMeta, TypeActor>;
    fn next(&mut self) -> Option<OpMove<TypeId, TypeMeta, TypeActor>> {
        self.ops.iter().next().cloned()
    }    
}

// generates a list of quasi-random operations.
// For each op:
//  1. child_id is generated randomly or picked randomly 
//      from existing ids if at least 5 existing.  (50/50 chance)
//  2. metadata is generated randomly
//  3. parent id is picked randomly from existing ids.
//
// (3) ensures that the tree is connected.
// (1) gives us both ops that create tree nodes and ops 
//      that move existing tree nodes.
//
// Note that when two OperationList are merged, the
// resulting trees will probably be disconnected.
//
// Note also that two OperationList may use the same
// clock/timestamp but have different parent/child/meta
// data.  This is an error condition for Tree, so
// the test cases must detect and discard if this occurs. 
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
        let mut nodes: Vec::<TypeId> = Vec::new();
        let mut parent_id = TypeId::arbitrary(g);

        let mut ops: Vec<OpMove<TypeId, TypeMeta, TypeActor>> = Vec::new();
        for _ in 0..size {
            let next_id = if nodes.len() > 5 && rand::random::<usize>() % 2 == 0 {
                nodes[rand::random::<usize>() % nodes.len()]
            } else {
                TypeId::arbitrary(g)
            };
            nodes.push(next_id.clone());
            let meta = TypeMeta::arbitrary(g);

            let op = OpMove::new(clock.tick(), parent_id, meta, next_id);
            let idx: usize = rand::random::<usize>() % nodes.len();
            parent_id = nodes[idx];

            ops.push(op);
        }
        Self{ ops }
    }
}

// helper: generate a new random id
fn new_id() -> TypeId {
    rand::random::<TypeId>()
}

// helper: generate a new random actor
fn new_actor() -> TypeActor {
    rand::random::<TypeActor>()
}

/// helper: checks if ops are stored in descending order in log.
fn check_log_is_descending(s: &State<TypeId, TypeMeta, TypeActor>) -> bool {
    let mut i = 0;
    let log = s.log();
    while i < log.len()-1 {
        let first = &log[i];
        let second = &log[i+1];

        if !(first.timestamp() > second.timestamp()) {
            return false;
        }
        i += 1;
    }
    true
}

// helper: checks if tree is acyclic (good) or contains cycles (bad)
fn acyclic(s: &State<TypeId, TypeMeta, TypeActor>) -> bool {
    let tree = s.tree();

    // Iterate all tree nodes and check if any node is an ancestor of itself.
    for (child_id, _) in tree.clone().into_iter() {
        if tree.is_ancestor(&child_id, &child_id) {
            return false;
        }
    }
    true
}

// helper: checks if any node has more than one parent.
fn parent_unique(s: &State<TypeId, TypeMeta, TypeActor>) -> bool {
    // A map of (child_id,parent_id) --> count
    let mut cnts: HashMap<(TypeId,TypeId), usize> = HashMap::new();

    // Iterate all tree nodes and store count of each child_id, parent_id pair.
    // If any pair is found to exist more than once, the invariant is broken.
    for (child_id, tn) in s.tree().clone().into_iter() {
        let key = (child_id.clone(), tn.parent_id().clone());
        let cnt = cnts.get(&key).unwrap_or(&0) + 1;
        cnts.insert(key, cnt);

        if cnt > 1 {
            return false;
        }
    }
    true
}

// helper: creates State and applies initial ops.
fn state_from_ops(oplist: &OperationList) -> State<TypeId, TypeMeta, TypeActor> {
    let mut s: State<TypeId, TypeMeta, TypeActor> = State::new();
    for op in oplist.ops.iter().cloned() {
        s.apply_op(op);
    }
    s
}


// Tests case 1 in the paper.  Concurrent moves of the same node.
//
// Initial State:
// root
//  - A
//  - B
//  - C
//
// Replica 1 moves A to be a child of B, while concurrently
// replica 2 moves the same node A to be a child of C.
// a child of B.  This could potentially result in A being
// duplicated under B and C, or A having 2 parents, B and C.
//
// The only valid result is for one operation
// to succeed and the other to be ignored, but both replica's
// must pick the same success case.
//
// See paper for diagram.
#[test]
fn concurrent_moves() {
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

    let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

    // Create ops for initial tree state.
    let ops = vec![OpMove::new(r1t.tick(), 0, "root", root_id),
                    OpMove::new(r1t.tick(), root_id, "a", a_id),
                    OpMove::new(r1t.tick(), root_id, "b", b_id),
                    OpMove::new(r1t.tick(), root_id, "c", c_id),
    ];

    // Apply initial ops to both replicas
    for op in ops {
        r1.apply_op(op.clone());
        r2.apply_op(op);        
    }

    // replica_1 moves /root/a to /root/b
    let r1_op = OpMove::new(r1t.tick(), b_id, "a", a_id);
    // replica_2 "simultaneously" moves /root/a to /root/c
    let r2_op = OpMove::new(r2t.tick(), c_id, "a", a_id);

    // apply both ops to r1
    r1.apply_op(r1_op.clone());
    r1.apply_op(r2_op.clone());

    // apply both ops to r2
    r2.apply_op(r2_op);
    r2.apply_op(r1_op);

    assert_eq!(r1, r2);
}

// Tests case 2 in the paper.  Moving a node to be a descendant of itself.
//
// Initial State:
// root
//  - A
//    - C
//  - B
//
// Initially, nodes A and B are siblings.  Replica 1 moves B
// to be a child of A, while concurrently replica 2 moves A to be
// a child of B.  This could potentially result in a cyle, or
// duplication.  The only valid result is for one operation
// to succeed and the other to be ignored, but both replica's
// must pick the same success case.
//
// See paper for diagram.
#[test]
fn concurrent_moves_cycle() {
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

    let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

    // Create ops for initial tree state.
    let ops = vec![OpMove::new(r1t.tick(), 0, "root", root_id),
                    OpMove::new(r1t.tick(), root_id, "a", a_id),
                    OpMove::new(r1t.tick(), root_id, "b", b_id),
                    OpMove::new(r1t.tick(), a_id, "c", c_id),
    ];

    // Apply initial ops to both replicas
    for op in ops {
        r1.apply_op(op.clone());
        r2.apply_op(op);        
    }

    // replica_1 moves /root/b to /root/a
    let r1_op = OpMove::new(r1t.tick(), a_id, "b", b_id);
    // replica_2 "simultaneously" moves /root/a to /root/b
    let r2_op = OpMove::new(r2t.tick(), b_id, "a", a_id);

    // apply both ops to r1
    r1.apply_op(r1_op.clone());
    r1.apply_op(r2_op.clone());

    // apply both ops to r2
    r2.apply_op(r2_op);
    r2.apply_op(r1_op);

    assert_eq!(r1, r2);
}


quickcheck! {

    // tests that operations are idempotent
    fn prop_idempotent(o: OperationList) -> TestResult {
        let r1 = state_from_ops(&o);
        let r2 = state_from_ops(&o);

        // r ^ r = r
        TestResult::from_bool(r1 == r2)
    }

    // tests that operations are commutative
    fn prop_commutative(o1: OperationList, o2: OperationList) -> TestResult {
        
        if o1.ops.len() > 0 && o2.ops.len() > 0 &&
            o1.ops[0].timestamp().actor_id() == o2.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        r1.apply_ops(&o2.ops);

        let mut r2 = state_from_ops(&o2);
        r2.apply_ops(&o1.ops);

        TestResult::from_bool(r1 == r2)
    }
    
    // tests that operations are associative
    fn prop_associative(
        o1: OperationList, 
        o2: OperationList,
        o3: OperationList
    ) -> TestResult {

        // discard if o1 actor is same as o2 actor
        if o1.ops.len() > 0 && o2.ops.len() > 0 && 
            o1.ops[0].timestamp().actor_id() == o2.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        // discard if o1 actor is same as o3 actor
        if o1.ops.len() > 0 && o3.ops.len() > 0 && 
            o1.ops[0].timestamp().actor_id() == o3.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        // discard if o2 actor is same as o3 actor
        if o2.ops.len() > 0 && o3.ops.len() > 0 && 
            o2.ops[0].timestamp().actor_id() == o3.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        let mut r2 = state_from_ops(&o2);

        // r1 <- r2
        r1.apply_ops(&o2.ops);

        // (r1 <- r2) <- r3
        r1.apply_ops(&o3.ops);

        // r2 <- r3
        r2.apply_ops(&o3.ops);

        // (r2 <- r3) <- r1
        r2.apply_ops(&o1.ops);

        TestResult::from_bool(r1 == r2)
    }

    // tests that the tree is always acyclic
    //
    // From the paper:
    // ----
    // A graph contains no cycles if no node is an ancestor of itself. 
    // ----    
    fn prop_acyclic(o1: OperationList, o2: OperationList) -> TestResult {

        if o1.ops.len() > 0 && o2.ops.len() > 0 &&
            o1.ops[0].timestamp().actor_id() == o2.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        r1.apply_ops(&o2.ops);

        let mut r2 = state_from_ops(&o2);
        r2.apply_ops(&o1.ops);

        let truth = acyclic(&r1) && acyclic(&r2);

        TestResult::from_bool(truth)
    }

    // tests that each child node has exactly one parent.
    //
    // From the paper:
    // ----
    // Each tree node must have either no parent (if the root of a tree)
    // or exactly one parent (if a non-root node).
    // Whenever the tree contains a triple whose third element is
    // the child node c, then the first and second elements of the
    // triple (the parent node and the metadata) are uniquely defined.
    // ----
    fn prop_parent_unique(o1: OperationList, o2: OperationList) -> TestResult {

        if o1.ops.len() > 0 && o2.ops.len() > 0 &&
            o1.ops[0].timestamp().actor_id() == o2.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        r1.apply_ops(&o2.ops);

        let mut r2 = state_from_ops(&o2);
        r2.apply_ops(&o1.ops);

        let truth = parent_unique(&r1) && parent_unique(&r2);

        TestResult::from_bool(truth)
    }

    // tests that the operation log is always in descending order
    // (even after applying ops from other replica)
    fn prop_log_descending(o1: OperationList, o2: OperationList) -> TestResult {

        if o1.ops.len() > 0 && o2.ops.len() > 0 &&
            o1.ops[0].timestamp().actor_id() == o2.ops[0].timestamp().actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        r1.apply_ops(&o2.ops);

        let mut r2 = state_from_ops(&o2);
        r2.apply_ops(&o1.ops);

        let descending = check_log_is_descending(&r1) &&
                         check_log_is_descending(&r2);

        TestResult::from_bool(descending)
    }
}
