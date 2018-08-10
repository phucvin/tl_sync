extern crate tl_sync;

use std::thread;
use std::sync::mpsc;
use tl_sync::*;

const LOOPS: u32 = 5;

fn heavy_computation() {/*
    let mut tmp = vec![];
    for _i in 1..10 {
        tmp.push(Tl::new(vec![1; 10_000]));
    }
    for it in tmp.iter() {
        it.sync(2, 1);
    }
    for it in tmp.iter() {
        it.sync(1, 0);
    }
*/}

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
}

fn main() {
    init_dirties();
    let now = std::time::Instant::now();
    {
        let root = Tl::new(0);
        let (compute_tx, compute_rx) = mpsc::channel();

        let ui_thread = {
            let root = root.clone();

            thread::Builder::new()
                .name("main_ui".into())
                .spawn(move || {
                    let mut i = 2;
                    while i > 0 {
                        notify();
                        for _ in 0..3 {
                            heavy_computation();
                        }
                        println!("ui_thread      | counter: {}", *root);

                        if *root != (2 * LOOPS) as usize {
                            thread::park();
                        } else {
                            i -= 1;
                        }
                    }
                }).unwrap()
        };

        let compute_thread = {
            let root = root.clone();
            let tx = compute_tx.clone();

            thread::Builder::new()
                .name("1_compute".into())
                .spawn(move || {
                    for _ in 0..LOOPS {
                        for _ in 0..2 {
                            heavy_computation();
                            *root.to_mut() += 1;

                            sync_from(2);
                        }
                        println!("compute_thread | counter: {}", *root);

                        tx.send(SyncStatus::Idle).unwrap();
                        thread::park();
                        sync_to(0);
                        tx.send(SyncStatus::JustSync).unwrap();
                    }
                }).unwrap()
        };

        for _ in 0..LOOPS {
            assert!(compute_rx.recv().unwrap() == SyncStatus::Idle);
            compute_thread.thread().unpark();
            assert!(compute_rx.recv().unwrap() == SyncStatus::JustSync);

            ui_thread.thread().unpark();
        }

        ui_thread.join().unwrap();
        compute_thread.join().unwrap();
    }
    println!("\nFPS: {}", 1000 / (now.elapsed().subsec_millis() / LOOPS));
    drop_dirties();
}
