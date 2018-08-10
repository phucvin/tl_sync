extern crate iui;
extern crate tl_sync;

use iui::prelude::*;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use tl_sync::*;

struct Trust<T> {
    inner: T,
}

unsafe impl<T> Send for Trust<T> {}
unsafe impl<T> Sync for Trust<T> {}

impl<T> Deref for Trust<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: Clone> Clone for Trust<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}

impl<T> Trust<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
        }
    }
}

#[derive(Clone)]
struct Counter {
    phase: Tl<u8>,
    counter: Tl<usize>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
    iui: Trust<UI>,
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

        let mut win = Window::new(
            &self.iui, "Counter",
            400, 300,
            WindowType::NoMenubar
        );
        win.show(&self.iui);

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

fn ui(root: Arc<Counter>) {
    if *root.phase == 0 {
        root.setup_ui();
    }
}

fn logic(root: Arc<Counter>) {
    if *root.phase == 0 {
        root.setup_logic();

        *root.phase.to_mut() = 1;
    }
}

fn main() {
    let stop = {
        let iui = UI::init().unwrap();
        let root = Arc::new(Counter {
            phase: Tl::new(0),
            counter: Tl::new(0),
            listeners: Default::default(),
            iui: Trust::new(iui.clone()),
        });
        let (tick, stop) = setup(root, ui, logic);
        let mut ev = iui.event_loop();
        
        ev.on_tick(&iui, move || {
            tick()
        });
        ev.run(&iui);

        stop
    };

    stop();
}
