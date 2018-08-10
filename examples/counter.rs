extern crate tl_sync;

use std::sync::{Arc, Mutex};
use tl_sync::*;

#[derive(Clone)]
struct Counter {
    phase: Tl<u8>,
    counter: Tl<usize>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Counter {
    fn setup_logic(&self) {
        println!("setup logic");

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                println!("compute thread | counter changed to: {}", *this.counter);
            }
        })));
    }

    fn setup_ui(&self) {
        println!("setup ui");

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                println!("ui thread     | counter changed to: {}", *this.counter);
            }
        })));
    }
    
    fn push(&self, handle_ref: ListenerHandleRef) {
        let mut v = self.listeners.lock().unwrap();
        v.push(handle_ref);
    }
}

fn ui(root: Arc<Counter>) -> bool {
    if *root.phase == 0 {
        root.setup_ui();
    }

    true
}

fn compute(root: Arc<Counter>) -> bool {
    if *root.phase == 0 {
        *root.phase.to_mut() = 1;
        root.setup_logic();

        *root.counter.to_mut() += 1;
    }
    
    true
}

fn main() {
    let root = Arc::new(Counter {
        phase: Tl::new(0),
        counter: Tl::new(0),
        listeners: Default::default(),
    });

    run(root, ui, compute);
}
