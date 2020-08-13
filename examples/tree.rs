extern crate crdts;

use crdts::Actor;
use crdts::tree::{Tree, TreeMeta, State, Clock, OpMove, apply_op};
use std::collections::HashMap;
use std::env;

#[derive(Debug)]
struct Replica<TM: TreeMeta, A: Actor> {
    id: A,                // globally unique id.
    state: State<TM, A>,  // state
    time:  Clock<A>       // must implement clock_interface

    // latest_time_by_replica: Vec<Clock<A>>;
}

impl<TM: TreeMeta, A: Actor + std::fmt::Debug> Replica<TM, A> {

    pub fn new(id: A) -> Self {
        Self {
            id: id.clone(),
            state: State::new(),
            time: Clock::<A>::new(id, None),
        }
    }

    pub fn apply_ops_noref(&mut self, ops: Vec<OpMove<TM, A>>) {
        for op in ops.clone() {
            self.time = self.time.merge(&op.timestamp);
            apply_op(op, &mut self.state);

/*            
            // store latest timestamp for this actor.
            let id = op.timestamp.actor_id();
            $latest = @$this->latest_time_by_replica[$id];
            if(!$latest || $op->timestamp->gt($latest)) {
                $this->latest_time_by_replica[$id] = $op->timestamp;
            }
*/            
        }
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

    function causally_stable_threshold(): ?clock_interface {

        // The minimum of latest timestamp from each replica
        // is the causally stable threshold.
        $oldest = null;
        foreach( $this->latest_time_by_replica as $id => $timestamp ) {
            if(!$oldest || $timestamp->lt($oldest)) {
                $oldest = $timestamp;
            }
        }
        return $oldest;
    }

    function truncate_log(): bool {
        $t = $this->causally_stable_threshold();
        if($t) {
            return $this->state->truncate_log_before($t);
        }
        return false;
    }
*/

    pub fn tick(&mut self) -> Clock<A> {
        self.time = self.time.inc();
        self.time.clone()
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

// print a tree.  (by first converting to a treenode)
fn print_tree<TM, A>(tree: &Tree<TM, A>, root: &A)
    where A: Actor + std::fmt::Debug, TM: TreeMeta {
    print_treenode(tree, root, 0, false);
}

fn print_replica_trees<TM, A>(repl1: &Replica<TM, A>, repl2: &Replica<TM, A>, root: &A)
    where A: Actor + std::fmt::Debug, TM: TreeMeta {
    println!("\n--replica_1 --");
    print_tree(&repl1.state.tree, root);
    println!("\n--replica_2 --");
    print_tree(&repl2.state.tree, root);
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
    print_tree(&r1.state.tree, &ids["root"]);

    // replica_1 moves /root/a to /root/b
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["b"], "a", ids["a"])];

    // replica_2 "simultaneously" moves /root/a to /root/c
    let repl2_ops = vec![OpMove::new(r1.tick(), ids["c"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(&r1.state.tree, &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(&r2.state.tree, &ids["root"]);
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
    print_tree(&r1.state.tree, &ids["root"]);

    // replica_1 moves /root/b to /root/a
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["a"], "b", ids["b"])];

    // replica_2 "simultaneously" moves /root/a to /root/b
    let repl2_ops = vec![OpMove::new(r1.tick(), ids["b"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(&r1.state.tree, &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(&r2.state.tree, &ids["root"]);
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


fn print_help() {
    let buf = "
Usage: tree <test>

<test> can be any of:
  test_concurrent_moves
  test_concurrent_moves_cycle

";
    println!("{}", buf);
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let test = if args.len() > 1 { &args[1] } else { "" };

    match test {
        "test_concurrent_moves" => test_concurrent_moves(),
        "test_concurrent_moves_cycle" => test_concurrent_moves_cycle(),
        _ => print_help(),
    }
}
