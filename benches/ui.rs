#![feature(test)]

extern crate test;
extern crate tl_sync;

use std::sync::Arc;
use test::Bencher;
use tl_sync::*;

const LOOPS: usize = 1;

struct Counter {
    counter: Tl<usize>,
    loops: Tl<usize>,
}

fn heavy_computation() {
    let mut tmp = vec![];
    for _i in 1..100 {
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
    heavy_computation();

    *root.loops < LOOPS
}

fn compute(root: Arc<Counter>) -> bool {
    *root.counter.to_mut() += 1;
    heavy_computation();

    *root.loops.to_mut() += 1;
    *root.loops < LOOPS
}

#[bench]
fn ui_multiple_threads(bench: &mut Bencher) {
    let root = Arc::new(Counter {
        counter: Tl::new(0),
        loops: Tl::new(0),
    });
    bench.iter(|| {
        // let now = std::time::Instant::now();
        run(root.clone(), ui, compute);
        // let duration = now.elapsed();
        // let duration = duration.as_secs() * 1000 + duration.subsec_millis() as u64;
        // let fps = 1000 / (duration / 5);
        // println!("T   : {}ms", duration);
        // println!("FPS : {}", fps);
    });
}

#[bench]
fn ui_single_thread(bench: &mut Bencher) {
    let root = Arc::new(Counter {
        counter: Tl::new(0),
        loops: Tl::new(0),
    });
    bench.iter(|| {
        run_single(root.clone(), ui, compute);
    });
}
