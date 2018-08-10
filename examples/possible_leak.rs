extern crate tl_sync;

use std::thread;
use tl_sync::*;

fn main() {
    init_dirties();
{
    let tmp = Tl::new(1);
    let a = Tl::new((true, tmp));

    println!("{}", *a.1);

    {
        let a = a.clone();
        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                a.to_mut().1 = Tl::new(3);
                // Possible leak if not using Arc inside Tl
                let _not_leak = Tl::new(100);
                sync_from(2);
                sync_to(0);
            }).unwrap()
            .join()
            .unwrap();
    }

    println!("{}", *a.1);
}
    drop_dirties();
}
