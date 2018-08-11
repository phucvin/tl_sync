// #![windows_subsystem="windows"]

extern crate iui;
extern crate tl_sync;
extern crate rayon;

use iui::prelude::*;
use iui::controls::{Button};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::time::{Instant, Duration};
use rayon::prelude::*;
use tl_sync::*;

#[derive(Clone)]
struct Controls {
    btn_test: Button,
}

#[derive(Clone)]
struct Counter {
    counter: Tl<Vec<usize>>,
    last_time: Tl<Instant>,
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
                let dt = Instant::now() - *this.last_time;
                println!("Compute's FPS: {}", 1000 / (dt.subsec_millis() + 1));

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
            // let this = self.clone();
            move || {
                // println!("compute thread | counter changed to: {}", this.counter[0]);
            }
        })));

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            move || {
                if this.counter[0] < 250 {
                    // for it in this.counter.to_mut().iter_mut() {
                    //     *it += 1;
                    // }
                    this.counter.to_mut().par_iter_mut().for_each(|it| {
                        *it += 1;
                    });
                    *this.last_time.to_mut() = Instant::now();
                }
            }
        })));
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(3).build_global().unwrap();

    let stop = {
        let iui = UI::init().unwrap();
        let root = Counter {
            counter: Tl::new(vec![0; 1024 * 1024 * 5]),
            last_time: Tl::new(Instant::now()),
            iui: Trust::new(iui.clone()),
            controls: Trust::new(RefCell::new(None)),
            listeners: Default::default(),
        };
        let (tick, stop) = setup(
            root,
            Duration::from_millis(15)
        );
        let mut ev = iui.event_loop();
        
        ev.on_tick(&iui, move || {
            tick()
        });

        // ev.run(&iui);
        loop {
            if !ev.next_tick(&iui) {
                break;
            }
        }

        stop
    };

    stop();
}
