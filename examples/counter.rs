extern crate tl_sync;

use std::thread;
use std::time;
use std::sync::mpsc;
use tl_sync::*;

const LOOPS: usize = 2;

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
}

fn main() {
    init_dirties();
    {
        let root = Tl::new(0);
        let (ui_tx, ui_rx) = mpsc::channel();
        let (compute_tx, compute_rx) = mpsc::channel();

        let ui_thread = {
            let root = root.clone();
            let tx = ui_tx.clone();

            thread::Builder::new()
                .name("main_ui".into())
                .spawn(move || {
                    for _ in 0..LOOPS {
                        println!("ui_thread      | counter: {}", *root);

                        tx.send(SyncStatus::Idle).unwrap();
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
                            thread::sleep(time::Duration::from_millis(100));
                            *root.to_mut() += 1;

                            sync_from(2);
                        }
                        println!("compute_thread | counter: {}", *root);

                        tx.send(SyncStatus::Idle).unwrap();
                        thread::park();
                        sync_to(0);
                        tx.send(SyncStatus::JustSync).unwrap();
                        thread::park();
                    }
                }).unwrap()
        };

        loop {
            assert!(ui_rx.recv().unwrap() == SyncStatus::Idle);
            assert!(compute_rx.recv().unwrap() == SyncStatus::Idle);
            
            compute_thread.thread().unpark();
            assert!(compute_rx.recv().unwrap() == SyncStatus::JustSync);
            compute_thread.thread().unpark();
            
            ui_thread.thread().unpark();
        }

        // ui_thread.join().unwrap();
        // compute_thread.join().unwrap();
    }
    drop_dirties();
}
