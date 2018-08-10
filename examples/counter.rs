extern crate tl_sync;

use std::thread;
use std::time;
use tl_sync::*;

fn main() {
    init_dirties();
    {
        let root = Tl::new(0);

        let ui_thread = {
            let root = root.clone();

            thread::Builder::new()
                .name("main_ui".into())
                .spawn(move || {
                    for _ in 0..2 {
                        sync_from(1);
                        println!("ui_thread      | counter: {}", *root);
                        thread::sleep(time::Duration::from_secs(1));
                    }
                }).unwrap()
        };

        let compute_thread = {
            let root = root.clone();

            thread::Builder::new()
                .name("1_compute".into())
                .spawn(move || {
                    for _ in 0..2 {
                        *root.to_mut() += 1;
                        sync_from(2);
                        sync_to(0);
                        thread::sleep(time::Duration::from_secs(1));
                    }
                }).unwrap()
        };

        ui_thread.join().unwrap();
        compute_thread.join().unwrap();
    }
    drop_dirties();
}
