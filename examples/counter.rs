extern crate tl_sync;

use std::thread;
use std::sync::mpsc;
use tl_sync::*;

const LOOPS: usize = 5;

fn heavy_computation() {
    let tmp = Tl::new(vec![1; 1024 * 1024 * 10]);
    tmp.sync(0, 1);
}

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
}

fn main() {
    init_dirties();
    {
        let root = Tl::new(0);
        let (compute_tx, compute_rx) = mpsc::channel();

        let ui_thread = {
            let root = root.clone();

            thread::Builder::new()
                .name("main_ui".into())
                .spawn(move || {
                    for i in 0..(LOOPS + 1) {
                        notify();
                        println!("ui_thread      | counter: {}", *root);

                        for _ in 0..2 {
                            heavy_computation();
                        }
                        if i < LOOPS {
                            thread::park();
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
    drop_dirties();
}
