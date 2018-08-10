extern crate tl_sync;

use std::thread;
use tl_sync::*;

fn main() {
    init_dirties();
{
    let thing: Tl<String> = Tl::new("banana".into());

    let thread = {
        let thing = thing.clone();

        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                *thing.to_mut() = "orange".into();
                thread::park();
                sync_from(2);
                sync_to(0);
            }).unwrap()
    };

    println!("still be banana, thing = {}", *thing);
    thread.thread().unpark();
    thread.join().unwrap();
    println!("different now, thing = {}", *thing);
}
    drop_dirties();
}
