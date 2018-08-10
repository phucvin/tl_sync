// #![windows_subsystem="windows"]

extern crate iui;
extern crate tl_sync;

use iui::prelude::*;
use std::sync::{Arc, Mutex};
use tl_sync::*;

#[derive(Clone)]
struct Counter {
    iui: Trust<UI>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
    counter: Tl<usize>,
}

impl Counter {
    fn push(&self, handle_ref: ListenerHandleRef) {
        let mut v = self.listeners.lock().unwrap();
        v.push(handle_ref);
    }
}

impl UiSetup for Counter {
    fn setup_ui(&self) {
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
}

impl ComputeSetup for Counter {
    fn setup_compute(&self) {
        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                println!("compute thread | counter changed to: {}", *this.counter);
            }
        })));
    }
}

fn main() {
    let stop = {
        let iui = UI::init().unwrap();
        let root = Counter {
            iui: Trust::new(iui.clone()),
            listeners: Default::default(),
            counter: Tl::new(0),
        };
        let (tick, stop) = setup(root);
        let mut ev = iui.event_loop();
        
        ev.on_tick(&iui, move || {
            tick()
        });
        ev.run(&iui);

        stop
    };

    stop();
}
