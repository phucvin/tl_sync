extern crate tl_sync;

use std::thread;
use tl_sync::*;

fn main() {
    init_dirties();
    {
        let thing: Tl<String> = Tl::new("banana".into());
        let decoy: Tl<String> = Tl::new("wood".into());

        let _must_live = register_listener_1(&thing, {
            let thing = thing.clone();

            move || {
                println!("thing changed to: {}", *thing);
            }
        });

        let _must_live_2 = register_listener_1(&thing, {
            let thing = thing.clone();

            move || {
                println!("thing changed to: {}", *thing);
            }
        });

        let _must_live_3 = register_listener_1(&thing, {
            let thing = thing.clone();

            move || {
                println!("thing changed to: {}", *thing);
            }
        });

        let _must_live_4 = register_listener_1(&thing, {
            let thing = thing.clone();

            move || {
                println!("thing changed to: {}", *thing);
            }
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
        peek_notify(prepare_peek_notify());
    }
    drop_dirties();
}
