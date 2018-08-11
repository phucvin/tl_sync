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
                        still_dirty = peek_notify() > 0;
                    }

                    tx.send(SyncStatus::Idle).unwrap();
                    match rx.recv() {
                        Ok(true) => (),
                        _ => break,
                    }
                    sync_to(0);
                    tx.send(SyncStatus::JustSync).unwrap();
                }
            }).unwrap()
    };

    root.setup_ui();

    let stop = Box::new({
        let compute_rtx = compute_rtx.clone();

        move || {
            compute_rtx.send(false).unwrap();
            compute_thread.join().unwrap();

            prepare_notify();
            drop_dirties();
        }
    });

    let tick = Box::new(move || {
        let prepared = prepare_notify();
        notify(prepared);

        match compute_rx.recv() {
            Ok(SyncStatus::Idle) => (),
            Ok(SyncStatus::Quit) => return,
            _ => return,
        }

        compute_rtx.send(true).unwrap();
        match compute_rx.recv() {
            Ok(SyncStatus::JustSync) => (),
            Ok(SyncStatus::Quit) => return,
            _ => return,
        }
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