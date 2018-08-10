use std::thread;
use std::sync::mpsc;
use super::*;

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
    Quit,
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

pub fn run_sync<T: 'static + Clone>(
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