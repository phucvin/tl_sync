extern crate tl_sync;

use std::thread;
use std::time;
use std::sync::mpsc;
use tl_sync::*;

const LOOPS: usize = 5;
const SLEEP: time::Duration = time::Duration::from_millis(1);

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
                    for _ in 0..(LOOPS + 1) {
                        notify();
                        println!("ui_thread      | counter: {}", *root);

                        thread::park();
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
                            thread::sleep(SLEEP);
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
        thread::sleep(SLEEP);
        ui_thread.thread().unpark();

        ui_thread.join().unwrap();
        compute_thread.join().unwrap();
    }
    drop_dirties();
}
