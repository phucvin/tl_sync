// #![windows_subsystem="windows"]

extern crate tl_sync;

use std::sync::Mutex;
use std::time::Duration;
use tl_sync::*;

#[derive(Clone)]
struct Root {
    money: Tl<usize>,
    on_inc_money: Action<(usize, String)>,
    on_dec_money: Action<(usize, String)>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Root {
    fn new() -> Self {
        Self {
            money: Tl::new(1000),
            on_inc_money: Action::new(),
            on_dec_money: Action::new(),
            listeners: Default::default(),
        }
    }

    fn clone_weak(&self) -> Self {
        let mut ret = self.clone();

        ret.listeners.be_weak();

        ret
    }

    fn setup(&self) {
        self.push(register_listener_2(&self.on_inc_money, &self.on_dec_money, {
            let this = self.clone_weak();
            move || {
                let mut total_inc = 0;
                let mut total_dec = 0;
                
                this.on_inc_money.for_each(|it| total_inc += it.0);
                this.on_dec_money.for_each(|it| total_dec -= it.0);

                assert!(*this.money + total_inc <= 2_000_000_000, "overflow");
                assert!(total_dec <= *this.money + total_inc, "invalid money inc/dec");
                *this.money.to_mut() = *this.money + total_inc - total_dec;
            }
        }));
    }

    fn defer(&self, h: ListenerHandleRef) {
        let mut l = self.listeners.lock().unwrap();
        l.push(h);
    }
}

#[derive(Clone)]
struct Item {
    id: String,
    value: Tl<isize>,
    on_upgrade: Action<()>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
    root: Wrc<Root>,
}

impl Item {
    fn new(id: String, value: isize, root: Root) -> Self {
        Self {
            id,
            value: Tl::new(value),
            on_upgrade: Action::new(),
            listeners: Default::default(),
            root: Wrc::new(root),
        }
    }

    fn clone_weak(&self) -> Self {
        let mut ret = self.clone();

        ret.listeners.be_weak();
        ret.root.be_weak();

        ret
    }

    fn setup(&self) {
        self.defer(register_listener_1(&self.on_upgrade, {
            let this = self.clone_weak();
            move || {
                let root = this.root.make_strong();

                if *root.money > 100 {
                    *this.value.to_mut() += 1;
                    root.on_dec_money.fire((100, "upgrade item"));
                }
            }
        }));
    }

    fn defer(&self, h: ListenerHandleRef) {
        let mut l = self.listeners.lock().unwrap();
        l.push(h);
    }
}

fn main() {
    let stop = {
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
