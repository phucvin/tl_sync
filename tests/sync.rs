extern crate tl_sync;

use std::thread;
use tl_sync::*;

#[test]
fn simple() {
    init_dirties();

    let a: Tl<usize> = Tl::new(1);
    assert!(*a == 1);

    let thread = {
        let a = a.clone();

        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                *a.to_mut() = 22;
                assert!(*a == 1);

                thread::park();
                assert!(*a == 1);

                sync_from(2);
                assert!(*a == 22);

                sync_to(0);
            }).unwrap()
    };

    assert!(*a == 1);
    thread.thread().unpark();
    thread.join().unwrap();
    assert!(*a == 22);
}
