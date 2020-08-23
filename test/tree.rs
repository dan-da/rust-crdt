/// tests for crdt-tree

use crdts::Actor;
use crdts::tree::{Clock, State, OpMove};
use quickcheck::{Arbitrary, Gen, TestResult};
use rand::Rng;

type TypeId = u8;
type TypeActor = u8;
type TypeMeta = char;
type TypeMetaStr<'a> = &'a str;

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
            let next_id = TypeId::arbitrary(g);
            nodes.push(next_id.clone());
            let meta = TypeMeta::arbitrary(g);

            let op = OpMove::new(tick(&mut clock), parent_id, meta, next_id);
            let idx: usize = rand::random::<usize>() % nodes.len();
            parent_id = nodes[idx];

            ops.push(op);
        }
        Self{ ops }
    }
}

fn new_id() -> TypeId {
    rand::random::<TypeId>()
}

fn new_actor() -> TypeActor {
    rand::random::<TypeActor>()
}

fn tick<A: Actor>(clock: &mut Clock<A>) -> Clock<A> {
    *clock = clock.inc();
    clock.clone()
}

#[test]
fn test_concurrent_moves() {
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

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
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

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

fn state_from_ops(oplist: &OperationList) -> State<TypeId, TypeMeta, TypeActor> {
    let mut s: State<TypeId, TypeMeta, TypeActor> = State::new();
    for op in oplist.ops.iter().cloned() {
        s.apply_op(op);
    }
    s
}

quickcheck! {
    fn prop_idempotent(o: OperationList) -> TestResult {
        let r1 = state_from_ops(&o);
        let r2 = state_from_ops(&o);

        // r ^ r = r
        TestResult::from_bool(r1 == r2)
    }

    fn prop_commutative(o1: OperationList, o2: OperationList) -> TestResult {
        
        if o1.ops.len() > 0 && o2.ops.len() > 0 &&
            o1.ops[0].timestamp.actor_id() == o2.ops[0].timestamp.actor_id() {
            return TestResult::discard();
        }

        let mut r1 = state_from_ops(&o1);
        r1.apply_ops(&o2.ops);

        let mut r2 = state_from_ops(&o2);
        r2.apply_ops(&o1.ops);

        TestResult::from_bool(r1 == r2)
    }
    
    fn prop_associative(
        o1: OperationList, 
        o2: OperationList,
        o3: OperationList
    ) -> TestResult {

        // discard if o1 actor is same as o2 actor
        if o1.ops.len() > 0 && o2.ops.len() > 0 && 
            o1.ops[0].timestamp.actor_id() == o2.ops[0].timestamp.actor_id() {
            return TestResult::discard();
        }

        // discard if o1 actor is same as o3 actor
        if o1.ops.len() > 0 && o3.ops.len() > 0 && 
            o1.ops[0].timestamp.actor_id() == o3.ops[0].timestamp.actor_id() {
            return TestResult::discard();
        }

        // discard if o2 actor is same as o3 actor
        if o2.ops.len() > 0 && o3.ops.len() > 0 && 
            o2.ops[0].timestamp.actor_id() == o3.ops[0].timestamp.actor_id() {
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
    
}
