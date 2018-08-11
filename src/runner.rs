use super::*;
use std::boxed::FnBox;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::any::Any;

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
}

pub trait UiSetup {
    fn setup_ui(&self);
    fn ui_act_on(&self, &Box<Any>);
}

pub trait ComputeSetup {
    fn setup_compute(&self);
    fn compute_act_on(&self, &Box<Any>);
}

pub fn setup<T: 'static + Send + Clone + UiSetup + ComputeSetup>(
    root: T,
    compute_update_duration: Duration,
) -> (Box<Fn()>, Box<FnBox()>) {
    init_dirties();
    init_actions();

    let (compute_tx, compute_rx) = mpsc::channel();
    let compute_rtx: mpsc::Sender<bool>;

    let _compute_thread = {
        let root = root.clone();
        let (tx, rx) = mpsc::channel();
        compute_rtx = tx.clone();
        let tx = compute_tx.clone();

        thread::Builder::new()
            .name("1_compute".into())
            .spawn(move || {
                root.setup_compute();
                loop {
                    notify_actions(&root, 2);

                    let mut still_dirty = true;
                    let now = Instant::now();
                    while still_dirty && now.elapsed() < compute_update_duration {
                        sync_from(2);
                        still_dirty = peek_notify() > 0;
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
                }
            }).unwrap()
    };

    root.setup_ui();

    let stop = Box::new({
        let compute_rtx = compute_rtx.clone();

        move || {
            compute_rtx.send(false).unwrap();
            prepare_notify();
            drop_actions();
            drop_dirties();
        }
    });

    let tick = Box::new(move || {
        let now = Instant::now();

        let prepared = prepare_notify();
        notify(prepared);

        notify_actions(&root, 1);

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

        // TODO Check if current remaining time is enough to sync, before send
        // if not, can postpone this sync to next tick,
        // remember not recv from compute_rx in that next tick
        compute_rtx.send(true).unwrap();
        // Should not recv_timeout here
        // must wait until receive JustSync before continue,
        // to avoid incomplete data sync when render UI
        match compute_rx.recv() {
            Ok(SyncStatus::JustSync) => (),
            _ => panic!("Unexpected error on ticker"),
        }
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
