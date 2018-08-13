// #![windows_subsystem="windows"]

extern crate iui;
extern crate rayon;
extern crate tl_sync;

use iui::controls::Button;
use iui::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::any::Any;
use tl_sync::*;

#[derive(Clone)]
struct Counter {
    counter: Tl<Vec<usize>>,
    last_time: Tl<Instant>,
    iui: Trust<UI>,
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
        let mut win = Window::new(&self.iui, "Counter", 400, 300, WindowType::NoMenubar);

        let mut btn_test = Button::new(&self.iui, "Click Me");
        btn_test.on_clicked(&self.iui, {
            let this = self.clone();
            move |_| fire(Box::new(this.counter[0]))
        });

        self.push(self.counter.register_listener(Box::new({
            let this = self.clone();
            let mut btn_test = btn_test.clone();
            move || {
                // let dt = Instant::now() - *this.last_time;
                // println!("Compute's FPS: {}", 1000 / (dt.subsec_millis() + 1));

                btn_test.set_text(&this.iui, &format!(
                    "Counter: {}", this.counter[0]
                ));
            }
        })));

        win.set_child(&self.iui, btn_test);
        win.show(&self.iui);
    }

    fn ui_act_on(&self, action: &Box<Any>) {
        println!("ui_act_on {}", action.downcast_ref::<usize>().unwrap());
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

    fn compute_act_on(&self, action: &Box<Any>) {
        println!("compute_act_on {}", action.downcast_ref::<usize>().unwrap());
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(3)
        .build_global()
        .unwrap();

    let stop = {
        let iui = UI::init().unwrap();
        let root = Counter {
            counter: Tl::new(vec![0; 1024 * 1024 * 5]),
            last_time: Tl::new(Instant::now()),
            iui: Trust::new(iui.clone()),
            listeners: Default::default(),
        };
        let (tick, stop) = setup(root, Duration::from_millis(15));
        let mut ev = iui.event_loop();

        ev.on_tick(&iui, move || tick());

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
