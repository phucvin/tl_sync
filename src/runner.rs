use super::*;
use std::boxed::FnBox;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
}

pub trait UiSetup {
    fn setup_ui(&self);
}

pub trait ComputeSetup {
    fn setup_compute(&self);
}

pub fn setup<T: 'static + Send + Clone + UiSetup + ComputeSetup>(
    root: T,
    compute_update_duration: Duration,
) -> (Box<FnMut()>, Box<FnBox()>) {
    init_dirties();

    let (compute_tx, compute_rx) = mpsc::channel();
    let compute_rtx: mpsc::Sender<bool>;

    let compute_thread = {
        let root = root.clone();
        let (tx, rx) = mpsc::channel();
        compute_rtx = tx.clone();
        let tx = compute_tx.clone();

        thread::Builder::new()
            .name("1_compute".into())
            .spawn(move || {
                root.setup_compute();
                loop {
                    let mut still_dirty = true;
                    let now = Instant::now();
                    while still_dirty && now.elapsed() < compute_update_duration {
                        sync_from(2);
                        still_dirty = peek_notify(prepare_peek_notify()) > 0;
                        sync_clear();
                    }

                    match tx.send(SyncStatus::Idle) {
                        Ok(_) => (),
                        _ => break,
                    }
                    match rx.recv() {
                        Ok(true) => (),
                        _ => break,
                    }
                    sync_to(0);
                    match tx.send(SyncStatus::JustSync) {
                        Ok(_) => (),
                        _ => break,
                    }

                    match rx.recv() {
                        Ok(true) => (),
                        _ => break,
                    }
                }
            }).unwrap()
    };

    root.setup_ui();

    let stop = Box::new({
        let compute_rtx = compute_rtx.clone();

        move || {
            compute_rtx.send(false).unwrap();
            compute_thread.join().unwrap();

            prepare_peek_notify();
            ensure_empty_dirties();
            drop_dirties();
        }
    });

    let mut just_sync = false;

    let tick = Box::new(move || {
        let now = Instant::now();

        sync_from(2);
        let prepared = prepare_peek_notify();

        if just_sync {
            just_sync = false;
            sync_to(1);
            compute_rtx.send(true).unwrap();
        }

        peek_notify(prepared);
        sync_clear();

        let ui_elapsed = now.elapsed();
        if ui_elapsed > compute_update_duration {
            // let total_elapsed = now.elapsed();
            // println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
            //     ui_elapsed.subsec_millis(),
            //     0,
            //     0,
            //     1000 / (total_elapsed.subsec_millis() + 1),
            // );
            return;
        }
        match compute_rx.recv_timeout(compute_update_duration - ui_elapsed) {
            Ok(SyncStatus::Idle) => (),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // let total_elapsed = now.elapsed();
                // let compute_elapsed = total_elapsed - ui_elapsed;
                // println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
                //     ui_elapsed.subsec_millis(),
                //     compute_elapsed.subsec_millis(),
                //     0,
                //     1000 / (total_elapsed.subsec_millis() + 1),
                // );
                return;
            }
            _ => panic!("Unexpected error on ticker"),
        }
        // let compute_elapsed = now.elapsed() - ui_elapsed;

        compute_rtx.send(true).unwrap();
        // Should not recv_timeout here
        // must wait until receive JustSync before continue,
        // to avoid incomplete data sync when render UI
        match compute_rx.recv() {
            Ok(SyncStatus::JustSync) => (),
            _ => panic!("Unexpected error on ticker"),
        }
        just_sync = true;
        // let sync_elapsed = now.elapsed() - ui_elapsed - compute_elapsed;

        let total_elapsed = now.elapsed();
        if total_elapsed < compute_update_duration {
            thread::sleep(compute_update_duration - total_elapsed);
        }
        // println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
        //     ui_elapsed.subsec_millis(),
        //     compute_elapsed.subsec_millis(),
        //     sync_elapsed.subsec_millis(),
        //     1000 / (total_elapsed.subsec_millis() + 1),
        // );
    });

    (tick, stop)
}
