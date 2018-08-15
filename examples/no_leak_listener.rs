extern crate tl_sync;

use std::thread;
use std::sync::Mutex;
use tl_sync::*;

#[derive(Clone)]
struct Container {
    thing: Tl<String>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Container {
    fn clone_weak(&self) -> Self {
        let mut ret = self.clone();
        
        ret.listeners.be_weak();
        
        ret
    }
}

fn main() {
    init_dirties();
    {
        let container = Container {
            thing: Tl::new("banana".into()),
            listeners: Wrc::new(Mutex::new(vec![])),
        };

        container.listeners.lock().unwrap().push(register_listener_1(&container.thing, {
            let container = container.clone_weak();

            move || {
                println!("thing changed to: {}", *container.thing);
            }
        }));

        let thread = {
            let container = container.clone();

            thread::Builder::new()
                .name("1_test".into())
                .spawn(move || {
                    *container.thing.to_mut() = "orange".into();
                    sync_from(2);
                    sync_to(0);
                }).unwrap()
        };

        thread.join().unwrap();
        peek_notify(prepare_peek_notify());
        sync_clear();
    }
    ensure_empty_dirties();
    drop_dirties();
}
