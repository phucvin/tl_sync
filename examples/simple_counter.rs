// #![windows_subsystem="windows"]

extern crate iui;
extern crate rayon;
extern crate tl_sync;

use iui::controls::{Button, HorizontalBox, Label};
use iui::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tl_sync::*;

#[derive(Clone)]
struct Counter {
    value: Tl<isize>,
    on_inc: Action<()>,
    on_dec: Action<()>,
    iui: Trust<UI>,
    listeners: Arc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Counter {
    fn new(iui: UI) -> Self {
        Self {
            value: Tl::new(0),
            on_inc: Action::new(),
            on_dec: Action::new(),
            iui: Trust::new(iui),
            listeners: Default::default(),
        }
    }

    fn setup(&self) {
        self.defer(register_listener_2(&self.on_inc, &self.on_dec, {
            let this = self.clone();
            move || {
                let value = this.value.to_mut();
                *value += this.on_inc.len() as isize;
                *value -= this.on_dec.len() as isize;
            }
        }));
    }

    fn defer(&self, h: ListenerHandleRef) {
        let mut l = self.listeners.lock().unwrap();
        l.push(h);
    }
}

impl UiSetup for Counter {
    fn setup_ui(&self) {
        let lbl_value = Label::new(&self.iui, "");

        let mut btn_inc = Button::new(&self.iui, "     +     ");
        btn_inc.on_clicked(&self.iui, {
            let this = self.clone();
            move |_| {
                this.on_inc.fire(());
            }
        });

        let mut btn_dec = Button::new(&self.iui, "     -     ");
        btn_dec.on_clicked(&self.iui, {
            let this = self.clone();
            move |_| {
                this.on_dec.fire(());
                this.on_inc.fire(());
                this.on_dec.fire(());
            }
        });

        let mut hbox = HorizontalBox::new(&self.iui);
        hbox.append(&self.iui, btn_dec.clone(), LayoutStrategy::Stretchy);
        hbox.append(&self.iui, lbl_value.clone(), LayoutStrategy::Stretchy);
        hbox.append(&self.iui, btn_inc.clone(), LayoutStrategy::Stretchy);

        let mut win = Window::new(&self.iui, "Counter", 400, 100, WindowType::NoMenubar);
        win.set_child(&self.iui, hbox.clone());
        win.show(&self.iui);

        self.defer(register_listener_1(&self.value, {
            let this = self.clone();
            let mut lbl_value = lbl_value.clone();
            move || {
                lbl_value.set_text(&this.iui, &format!("                     {}", *this.value));
            }
        }));
    }
}

impl ComputeSetup for Counter {
    fn setup_compute(&self) {
        self.setup();
    }
}

fn main() {
    let stop = {
        let iui = UI::init().unwrap();
        let root = Counter::new(iui.clone());
        let (mut tick, stop) = setup(root.clone(), Duration::from_millis(15));
        let mut ev = iui.event_loop();

        ev.on_tick(&iui, move || {
            tick();
        });

        loop {
            if !ev.next_tick(&iui) {
                break;
            }
        }

        stop
    };

    stop();
}
