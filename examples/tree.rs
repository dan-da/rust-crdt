extern crate crdts;

use crdts::Actor;
use crdts::tree::{tree::Tree, treemeta::TreeMeta, clock::Clock, opmove::OpMove};
use crdts::treestate::State;
use std::collections::HashMap;
use std::env;
use rand::Rng;

#[derive(Debug)]
struct Replica<TM: TreeMeta, A: Actor> {
    id: A,                // globally unique id.
    state: State<TM, A>,  // state
    time:  Clock<A>,       // must implement clock_interface

    latest_time_by_replica: HashMap<A, Clock<A>>,
}

impl<TM: TreeMeta, A: Actor + std::fmt::Debug> Replica<TM, A> {

    pub fn new(id: A) -> Self {
        Self {
            id: id.clone(),
            state: State::new(),
            time: Clock::<A>::new(id, None),
            latest_time_by_replica: HashMap::<A, Clock<A>>::new()
        }
    }

    pub fn id(&self) -> &A {
        &self.id
    }

    pub fn apply_ops_noref(&mut self, ops: Vec<OpMove<TM, A>>) {
        for op in ops.clone() {
            self.time = self.time.merge(&op.timestamp);

            // store latest timestamp for this actor.
            let id = op.timestamp.actor_id();
            let result = self.latest_time_by_replica.get(id);
            match result {
                Some(latest) if !(op.timestamp > *latest) => {},
                _ => { self.latest_time_by_replica.insert(id.clone(), op.timestamp.clone()); },
            };

            self.state.apply_op(op);

/*            
            if(!$latest || $op->timestamp->gt($latest)) {
                $this->latest_time_by_replica[$id] = $op->timestamp;
            }
*/            
        }
    }

    pub fn state(&self) -> &State<TM, A> {
        &self.state
    }

    pub fn tree(&self) -> &Tree<TM, A> {
        self.state.tree()
    }

    pub fn apply_ops(&mut self, ops: &Vec<OpMove<TM, A>>) {
        self.apply_ops_noref(ops.clone())
    }

    /*
    // applies ops from a log.  useful for log replay.
    function apply_log_ops(array $log_ops) {
        $ops = [];
        foreach($log_ops as $log_op) {
            $ops[] = op_move::from_log_op_move($log_op);
        }
        $this->apply_ops($ops);
    }
*/    

    pub fn causally_stable_threshold(&self) -> Option<&Clock<A>> {
        // The minimum of latest timestamp from each replica
        // is the causally stable threshold.

        let mut v: Vec<&Clock<A>> = self.latest_time_by_replica.values().collect();
        v.sort_unstable_by(|a, b| a.cmp(b));
        if v.len() > 0 { Some(v[0]) } else { None }
    }

    pub fn truncate_log(&mut self) -> bool {
        let result = self.causally_stable_threshold();
        match result.cloned() {
            Some(t) => self.state.truncate_log_before(&t),
            None => false,
        }
    }

    pub fn tick(&mut self) -> Clock<A> {
        self.time = self.time.inc();
        self.time.clone()
    }
    
}

// Returns operations representing a depth-first tree, 
// with 2 children for each parent.
fn mktree_ops(ops: &mut Vec<OpMove<&str, u64>>, r: &mut Replica<&str, u64>, parent_id: u64, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    for i in 0..2 {
        let name = if i == 0 { "a" } else { "b" };
        let child_id = new_id();
        ops.push( OpMove::new(r.tick(), parent_id, name, child_id) );
        mktree_ops(ops, r, child_id, depth+1, max_depth);
    }
}

fn apply_ops_to_replicas<TM, A>(replicas: &mut Vec<Replica<TM, A>>, ops: &Vec<OpMove<TM, A>>)
    where A: Actor + std::fmt::Debug, TM: TreeMeta {
    for r in replicas.iter_mut() {
        r.apply_ops(ops);
    }
}

// note: in practice a UUID (at least 128 bits should be used)
fn new_id() -> u64 {
    rand::random::<u64>()
}

// print a treenode, recursively
fn print_treenode<TM, A>(tree: &Tree<TM, A>, node_id: &A, depth: usize, with_id: bool) 
    where A: Actor + std::fmt::Debug, TM: TreeMeta {

    let result = tree.find(&node_id);
    let meta = match result {
        Some(tn) => {
            format!("{:?}", tn.metadata())
        },
        None if depth == 0 => {
            "forest".to_string()
        },
        None => {
            panic!("tree node {:?} not found", node_id);
        }
    };
    println!("{:indent$}{}", "", meta, indent=depth*2);

    for c in tree.children(&node_id) {
        print_treenode(tree, &c, depth+1, with_id);
    }
}

// print a tree.
fn print_tree<TM, A>(tree: &Tree<TM, A>, root: &A)
    where A: Actor + std::fmt::Debug, TM: TreeMeta {
    print_treenode(tree, root, 0, false);
}

fn print_replica_trees<TM, A>(repl1: &Replica<TM, A>, repl2: &Replica<TM, A>, root: &A)
    where A: Actor + std::fmt::Debug, TM: TreeMeta {
    println!("\n--replica_1 --");
    print_tree(repl1.tree(), root);
    println!("\n--replica_2 --");
    print_tree(repl2.tree(), root);
    println!("");
}

// See paper for diagram.
fn test_concurrent_moves() {
    let mut r1: Replica<&str, u64> = Replica::new(new_id());
    let mut r2: Replica<&str, u64> = Replica::new(new_id());

    let ids: HashMap<&str, u64> = [
        ("root", 0), 
        ("a", new_id()), 
        ("b", new_id()), 
        ("c", new_id())]
    .iter().cloned().collect();

    // Setup initial tree state.
    let ops = vec![OpMove::new(r1.tick(), 0, "root", ids["root"]),
                   OpMove::new(r1.tick(), ids["root"], "a", ids["a"]),
                   OpMove::new(r1.tick(), ids["root"], "b", ids["b"]),
                   OpMove::new(r1.tick(), ids["root"], "c", ids["c"]),
    ];

    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/a to /root/b
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["b"], "a", ids["a"])];

    // replica_2 "simultaneously" moves /root/a to /root/c
    let repl2_ops = vec![OpMove::new(r2.tick(), ids["c"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/c/a because last-writer-wins
    // and replica_2's op has a later timestamp.
//    if r1.state.is_equal(&r2.state) {
    if r1.state == r2.state {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state);
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state);
    }
    
    r1.state.check_log_is_descending();
    r2.state.check_log_is_descending();
}

fn test_concurrent_moves_cycle() {
    let mut r1: Replica<&str, u64> = Replica::new(new_id());
    let mut r2: Replica<&str, u64> = Replica::new(new_id());

    let ids: HashMap<&str, u64> = [
        ("root", 0), 
        ("a", new_id()), 
        ("b", new_id()), 
        ("c", new_id())]
    .iter().cloned().collect();

    // Setup initial tree state.
    let ops = vec![OpMove::new(r1.tick(), 0, "root", ids["root"]),
                   OpMove::new(r1.tick(), ids["root"], "a", ids["a"]),
                   OpMove::new(r1.tick(), ids["root"], "b", ids["b"]),
                   OpMove::new(r1.tick(), ids["a"], "c", ids["c"]),
    ];

    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/b to /root/a
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["a"], "b", ids["b"])];

    // replica_2 "simultaneously" moves /root/a to /root/b
    let repl2_ops = vec![OpMove::new(r2.tick(), ids["b"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/c/a because last-writer-wins
    // and replica_2's op has a later timestamp.
    if r1.state == r2.state {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state);
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state);
    }
    
    r1.state.check_log_is_descending();
    r2.state.check_log_is_descending();
}

fn test_walk_deep_tree() {

    let mut r1: Replica<&str, u64> = Replica::new(new_id());

    let ids: HashMap<&str, u64> = [
        ("root", 0),
    ].iter().cloned().collect();

    // Generate initial tree state.
    println!("generating ops...");
    let mut ops = vec![OpMove::new(r1.tick(), 0, "root", ids["root"]),];
    mktree_ops(&mut ops, &mut r1, ids["root"], 2, 13);

    println!("applying ops...");
    r1.apply_ops(&ops);

    println!("walking tree...");
    r1.tree().walk(&ids["root"], &|tree, node_id, depth| {
        if false {
            let meta = match tree.find(node_id) {
                Some(tn) => format!("{:?}", tn.metadata()),
                None => format!("{:?}", node_id),
            };
            println!("{:indent$}{}", "", meta, indent=depth);
        }
    });

    println!("\nnodes in tree: {}", ops.len());
}


fn test_truncate_log() {

    let mut replicas: Vec<Replica<&str, u64>> = Vec::new();
    let num_replicas = 5;

    // start some replicas.
    for _i in 0..num_replicas {
        let r: Replica<&str, u64> = Replica::new(new_id());
        replicas.push(r);
    }

    let root_id = new_id();

    // Generate initial tree state.
    let mut ops = vec![OpMove::new(replicas[0].tick(), 0, "root", root_id)];

    println!("generating move operations...");

    // generate some initial ops from all replicas.
    for mut r in replicas.iter_mut() {
        let finaldepth = rand::thread_rng().gen_range(3,6);
        mktree_ops(&mut ops, &mut r, root_id, 2, finaldepth);
    }

    // apply all ops to all replicas
    println!("applying {} operations to all {} replicas...\n", ops.len(), replicas.len());
    apply_ops_to_replicas(&mut replicas, &ops);

    #[derive(Debug)]
    struct Stat {
        pub replica: u64,
        pub ops_before_truncate: usize,
        pub ops_after_truncate: usize,
    }

    let mut stats: Vec<Stat> = Vec::new();
    for r in replicas.iter_mut() {
        println!("truncating log of replica {}...", r.id());
        println!("causally stable threshold: {:?}\n", r.causally_stable_threshold() );
        let ops_b4 = r.state().log().len();
        r.truncate_log();
        let ops_after = r.state().log().len();
        stats.push( 
            Stat{ 
                replica: *r.id(),
                ops_before_truncate: ops_b4, 
                ops_after_truncate: ops_after,
            }
        );
    }


    println!("-- Stats -- ");
    println!("\n{:#?}", stats);
}


fn print_help() {
    let buf = "
Usage: tree <test>

<test> can be any of:
  test_concurrent_moves
  test_concurrent_moves_cycle
  test_truncate_log
  test_walk_deep_tree

";
    println!("{}", buf);
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let test = if args.len() > 1 { &args[1] } else { "" };

    match test {
        "test_concurrent_moves" => test_concurrent_moves(),
        "test_concurrent_moves_cycle" => test_concurrent_moves_cycle(),
        "test_truncate_log" => test_truncate_log(),
        "test_walk_deep_tree" => test_walk_deep_tree(),
        _ => print_help(),
    }
}
