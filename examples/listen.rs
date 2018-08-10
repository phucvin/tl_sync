extern crate tl_sync;

use std::thread;
use tl_sync::*;

fn main() {
    init_dirties();

    let thing: Tl<String> = Tl::new("banana".into());

    thing.register_listener({
        let thing = thing.clone();

        Box::new(move || {
            println!("thing changed to: {}", *thing);
        })
    });

    let thread = {
        let thing = thing.clone();

        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                *thing.to_mut() = "orange".into();
                sync_from(2);
                sync_to(0);
            }).unwrap()
    };

    thread.join().unwrap();
    notify();
}
