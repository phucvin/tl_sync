// #![windows_subsystem="windows"]

extern crate iui;
extern crate rayon;
extern crate tl_sync;

use iui::controls::Button;
use iui::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::cell::Cell;
use std::rc::Rc;
use tl_sync::*;

struct Click {
    counter_at: usize,
}

#[derive(Clone)]
struct Counter {
    counter: Tl<Vec<usize>>,
    on_click: Action<Click>,
    time: Tl<Instant>,
    last_time: Tl<Instant>,
    ticks: Tl<u64>,
    iui: Trust<UI>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Counter {
    fn register_listener<T: RegisterListener, U: 'static + FnMut()>(&self, v: &T, f: U) {
        let mut l = self.listeners.lock().unwrap();
        l.push(v.register_listener(Box::new(f)));
    }
}

impl UiSetup for Counter {
    fn setup_ui(&self) {
        let mut win = Window::new(&self.iui, "Counter", 400, 300, WindowType::NoMenubar);

        let mut btn_test = Button::new(&self.iui, "Click Me");
        btn_test.on_clicked(&self.iui, {
            let this = self.clone();
            move |_| {
                this.on_click.fire(Click { counter_at: this.counter[0] });
            }
        });

        self.register_listener(&self.counter, {
            let this = self.clone();
            let mut btn_test = btn_test.clone();
            let last_ticks = Rc::new(Cell::new(0));

            move || {
                if *this.ticks <= last_ticks.get() { return; }
                last_ticks.set(*this.ticks);

                // let dt = *this.time - *this.last_time;
                // println!("FPS: {}", 1000 / (dt.subsec_millis() + 1));

                btn_test.set_text(&this.iui, &format!(
                    "Counter: {}", this.counter[0]
                ));
            }
        });

        self.register_listener(&self.on_click, {
            let this = self.clone();
            move || {
                println!("ui on_click: {}", this.on_click.len());
            }
        });

        win.set_child(&self.iui, btn_test);
        win.show(&self.iui);
    }
}

impl ComputeSetup for Counter {
    fn setup_compute(&self) {
        self.counter.to_mut()[0] = 0;

        self.register_listener(&self.counter, {
            let this = self.clone();
            move || {
                if this.counter[0] < 250 {
                    // for it in this.counter.to_mut().iter_mut() {
                    //     *it += 1;
                    // }
                    this.counter.to_mut().par_iter_mut().for_each(|it| {
                        *it += 1;
                    });
                }
            }
        });

        self.register_listener(&self.on_click, {
            let this = self.clone();
            move || {
                println!("compute on_click: {}", this.on_click[0].counter_at);
            }
        });
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
            on_click: Action::new(),
            time: Tl::new(Instant::now()),
            last_time: Tl::new(Instant::now()),
            ticks: Tl::new(0),
            iui: Trust::new(iui.clone()),
            listeners: Default::default(),
        };
        let (mut tick, stop) = setup(root.clone(), Duration::from_millis(15));
        let mut ev = iui.event_loop();

        ev.on_tick(&iui, move || {
            *root.last_time.to_mut() = *root.time;
            *root.time.to_mut() = Instant::now();
            *root.ticks.to_mut() += 1;
            tick();
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
