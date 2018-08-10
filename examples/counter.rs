extern crate tl_sync;

use std::thread;
use std::sync::mpsc;
use tl_sync::*;

const LOOPS: u32 = 5;

fn heavy_computation() {
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
}

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
    JustPrepareNotify,
}

fn main() {
    init_dirties();
    let now = std::time::Instant::now();
    {
        let root = Tl::new(0);
        let (ui_tx, ui_rx) = mpsc::channel();
        let (compute_tx, compute_rx) = mpsc::channel();
        let compute_rtx: mpsc::Sender<()>;
        let ui_rtx: mpsc::Sender<()>;

        let ui_thread = {
            let root = root.clone();
            let (tx, rx) = mpsc::channel();
            ui_rtx = tx;
            let tx = ui_tx.clone();

            thread::Builder::new()
                .name("main_ui".into())
                .spawn(move || {
                    for i in 0..(LOOPS + 1) {
                        let tmp = prepare_notify();
                        tx.send(SyncStatus::JustPrepareNotify).unwrap();
                        rx.recv().unwrap();
                        notify(tmp);

                        for _ in 0..3 {
                            heavy_computation();
                        }
                        println!("ui_thread      | counter: {}", *root);

                        if i < LOOPS {
                            rx.recv().unwrap();
                        }
                    }
                }).unwrap()
        };

        let compute_thread = {
            let root = root.clone();
            let (tx, rx) = mpsc::channel();
            compute_rtx = tx.clone();
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
                        rx.recv().unwrap();
                        sync_to(0);
                        tx.send(SyncStatus::JustSync).unwrap();
                    }
                }).unwrap()
        };

        for _ in 0..LOOPS {
            assert!(ui_rx.recv().unwrap() == SyncStatus::JustPrepareNotify);
            ui_rtx.send(()).unwrap();

            assert!(compute_rx.recv().unwrap() == SyncStatus::Idle);
            compute_rtx.send(()).unwrap();
            assert!(compute_rx.recv().unwrap() == SyncStatus::JustSync);

            ui_rtx.send(()).unwrap();
        }
        
        assert!(ui_rx.recv().unwrap() == SyncStatus::JustPrepareNotify);
        ui_rtx.send(()).unwrap();

        ui_thread.join().unwrap();
        compute_thread.join().unwrap();
    }
    println!("\nFPS: {}", 1000 / (now.elapsed().subsec_millis() / LOOPS));
    drop_dirties();
}
