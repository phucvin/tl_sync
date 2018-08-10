extern crate tl_sync;

use std::sync::Arc;
use tl_sync::*;

const LOOPS: usize = 5;

struct Counter {
    counter: Tl<usize>,
    loops: Tl<usize>,
}

fn heavy_computation() {
    let mut tmp = vec![];
    for _i in 1..200 {
        tmp.push(Tl::new(vec![1; 10_000]));
    }
    for it in tmp.iter() {
        it.sync(2, 1);
    }
    for it in tmp.iter() {
        it.sync(1, 0);
    }
}

fn ui(root: Arc<Counter>) -> bool {
    println!("ui thread      | counter: {}", *root.counter);
    heavy_computation();
    
    *root.loops < LOOPS
}

fn compute(root: Arc<Counter>) -> bool {
    *root.counter.to_mut() += 1;
    heavy_computation();
    
    *root.loops.to_mut() += 1;
    *root.loops < LOOPS
}

fn main() {
    let now = std::time::Instant::now();
    {
        let root: Arc<Counter> = Arc::new(Counter {
            counter: Tl::new(0),
            loops: Tl::new(0),
        });
        run_sync(root, ui, compute);
    }
    println!("\n{}s {}ms", now.elapsed().as_secs(), now.elapsed().subsec_millis());
}
