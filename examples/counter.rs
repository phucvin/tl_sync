// #![windows_subsystem="windows"]

extern crate iui;
extern crate tl_sync;

use iui::prelude::*;
use iui::controls::{Button};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::time::Duration;
use tl_sync::*;

#[derive(Clone)]
struct Controls {
    btn_test: Button,
}

#[derive(Clone)]
struct Counter {
    counter: Tl<Vec<usize>>,
    iui: Trust<UI>,
    controls: Trust<RefCell<Option<Controls>>>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
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

        let btn_test = Button::new(&self.iui, "Click Me");
        win.set_child(&self.iui, btn_test.clone());

        *self.controls.borrow_mut() = Some(Controls {
            btn_test
        });

        win.show(&self.iui);

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                println!("ui thread      | counter update");
                let mut controls = this.controls.borrow().clone().unwrap();
                controls.btn_test.set_text(&this.iui,
                    &format!("Counter: {}", this.counter[0])
                );
            }
        })));
    }
}

impl ComputeSetup for Counter {
    fn setup_compute(&self) {
        self.counter.to_mut()[0] = 1;

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                println!("compute thread | counter changed to: {}", this.counter[0]);
            }
        })));

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                if this.counter[0] < 2500 {
                    for it in this.counter.to_mut().iter_mut() {
                        *it += 1;
                    }
                }
            }
        })));
    }
}

fn main() {
    let stop = {
        let iui = UI::init().unwrap();
        let root = Counter {
            counter: Tl::new(vec![0; 1024 * 1024 * 5]),
            iui: Trust::new(iui.clone()),
            controls: Trust::new(RefCell::new(None)),
            listeners: Default::default(),
        };
        let (tick, stop) = setup(
            root,
            Duration::from_millis(5)
        );
        let mut ev = iui.event_loop();
        
        ev.on_tick(&iui, move || {
            tick()
        });
        ev.run(&iui);

        stop
    };

    stop();
}
