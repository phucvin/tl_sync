use std::thread;
use std::sync::mpsc;
use super::*;

#[derive(PartialEq)]
enum SyncStatus {
    Idle,
    JustSync,
    JustPrepareNotify,
    Quit,
}

pub fn run<T: 'static + Send + Sync + Clone>(
    root: T,
    ui: fn(T) -> bool,
    compute: fn(T) -> bool
) {
    init_dirties();
    {
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
                    loop {
                        let tmp = prepare_notify();
                        tx.send(SyncStatus::JustPrepareNotify).unwrap();
                        rx.recv().unwrap();
                        notify(tmp);

                        let ret = ui(root.clone());

                        tx.send(SyncStatus::Idle).unwrap();
                        
                        if ret == false {
                            tx.send(SyncStatus::Quit).unwrap();
                            break;
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
            match ui_rx.recv() {
                Ok(SyncStatus::JustPrepareNotify) => (),
                Ok(SyncStatus::Quit) => break,
                _ => break,
            }
            ui_rtx.send(()).unwrap();

            match compute_rx.recv() {
                Ok(SyncStatus::Idle) => (),
                Ok(SyncStatus::Quit) => break,
                _ => break,
            }
            match ui_rx.recv() {
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
        }

        ui_thread.join().unwrap();
        compute_thread.join().unwrap();
    }
    drop_dirties();
}
