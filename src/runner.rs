use std::thread;
use std::sync::mpsc;
use std::boxed::FnBox;
use std::time::{Instant, Duration};
use super::*;

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
    Quit,
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
) -> (Box<Fn()>, Box<FnBox()>) {
    init_dirties();

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
            drop_dirties();
        }
    });

    let tick = Box::new(move || {
        let now = Instant::now();
        
        let prepared = prepare_notify();
        notify(prepared);
        let ui_elapsed = now.elapsed();

        if ui_elapsed > compute_update_duration {
            let total_elapsed = now.elapsed();
            println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
                ui_elapsed.subsec_millis(),
                0,
                0,
                1000 / (total_elapsed.subsec_millis() + 1),
            );
            return;
        }
        match compute_rx.recv_timeout(compute_update_duration - ui_elapsed) {
            Ok(SyncStatus::Idle) => (),
            Ok(SyncStatus::Quit) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let total_elapsed = now.elapsed();
                let compute_elapsed = total_elapsed - ui_elapsed;
                println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
                    ui_elapsed.subsec_millis(),
                    compute_elapsed.subsec_millis(),
                    0,
                    1000 / (total_elapsed.subsec_millis() + 1),
                );
                return
            }
            _ => return,
        }
        let compute_elapsed = now.elapsed() - ui_elapsed;

        // TODO Check if current remaining time is enough to sync, before send
        // if not, can postpone this sync to next tick,
        // remember not recv from compute_rx in that next tick
        compute_rtx.send(true).unwrap();
        // Should not recv_timeout here
        // must wait until receive JustSync before continue,
        // to avoid incomplete data sync when render UI
        match compute_rx.recv() {
            Ok(SyncStatus::JustSync) => (),
            Ok(SyncStatus::Quit) => return,
            _ => return,
        }
        let sync_elapsed = now.elapsed() - ui_elapsed - compute_elapsed;

        let total_elapsed = now.elapsed();
        println!("UI: {}ms \t | COM: | {}ms\t | SYNC: | {}ms\t | FPS: {}",
            ui_elapsed.subsec_millis(),
            compute_elapsed.subsec_millis(),
            sync_elapsed.subsec_millis(),
            1000 / (total_elapsed.subsec_millis() + 1),
        );
    });

    (tick, stop)
}

pub fn run<T: 'static + Send + Sync + Clone>(
    root: T,
    ui: fn(T) -> bool,
    compute: fn(T) -> bool
) {
    init_dirties();
    {
        let (compute_tx, compute_rx) = mpsc::channel();
        let compute_rtx: mpsc::Sender<()>;

        let compute_thread = {
            let root = root.clone();
            let (tx, rx) = mpsc::channel();
            compute_rtx = tx.clone();
            let tx = compute_tx.clone();

            thread::Builder::new()
                .name("1_compute".into())
                .spawn(move || {
                    loop {
                        let ret = compute(root.clone());
                        sync_from(2);

                        tx.send(SyncStatus::Idle).unwrap();
                        rx.recv().unwrap();
                        sync_to(0);
                        tx.send(SyncStatus::JustSync).unwrap();

                        if ret == false {
                            tx.send(SyncStatus::Quit).unwrap();
                            break;
                        }
                    }
                }).unwrap()
        };

        loop {
            let prepared = prepare_notify();
            notify(prepared);
            let ret = ui(root.clone());

            match compute_rx.recv() {
                Ok(SyncStatus::Idle) => (),
                Ok(SyncStatus::Quit) => break,
                _ => break,
            }

            compute_rtx.send(()).unwrap();
            match compute_rx.recv() {
                Ok(SyncStatus::JustSync) => (),
                Ok(SyncStatus::Quit) => break,
                _ => break,
            }
            
            if ret == false {
                prepare_notify();
                break;
            }
        }

        compute_thread.join().unwrap();
    }
    drop_dirties();
}

pub fn run_single<T: 'static + Clone>(
    root: T,
    ui: fn(T) -> bool,
    compute: fn(T) -> bool
) {
    init_dirties();
    {
        loop {
            notify(prepare_notify());
            get_dirties().to_mut(0).clear();

            let ui_ret = ui(root.clone());
            let compute_ret = compute(root.clone());
            sync_from(2);

            if ui_ret == false || compute_ret == false {
                break;
            }
        }
    }
    drop_dirties();
}