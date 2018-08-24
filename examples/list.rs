// #![windows_subsystem="windows"]

extern crate tl_sync;

use std::sync::Mutex;
use std::time::Duration;
use tl_sync::*;

struct VerifyAction<T: Clone> {
    trigger: Action<T>,
    verified: Action<T>,
}

impl<T: Clone> VerifyAction<T> {
    fn new() -> Self {
        Self {
            trigger: Action::new(),
            verified: Action::new(),
        }
    }

    fn transfer(&self) {
        trigger.for_each(|it| self.verified.fire(it.clone()));
    }
}

#[derive(Clone)]
struct Root {
    money: Tl<usize>,
    on_upgrade_item: VerifyAction<String>,
    on_iap: Action<usize>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Root {
    fn new() -> Self {
        Self {
            money: Tl::new(1000),
            on_upgrade_item: VerifyAction::new(),
            on_iap: Action::new(),
            listeners: Default::default(),
        }
    }

    fn clone_weak(&self) -> Self {
        let mut ret = self.clone();

        ret.listeners.be_weak();

        ret
    }

    fn setup(&self) {
        self.push(register_listener_1(&self.on_upgrade_item.trigger, {
            let this = self.clone_weak();
            move || {
                let mut required_money = 0;

                for item_id in this.on_upgrade_item.trigger {
                    let item = this.item_map.get(item_id).unwrap();

                    required_money += *item.value;
                }

                if *this.money >= required_money {
                    this.on_upgrade_item.transfer();
                } else {
                    // TODO Show/Toast error to UI
                }
            }
        }));

        self.push(register_listener_2(&self.on_upgrade_item.verified, &self.on_iap, {
            let this = self.clone_weak();
            move || {
                let mut inc = 0;
                let mut dec = 0;

                this.on_iap.for_each(|it| inc += it);

                this.on_upgrade_item.verified.for_each(|it| {
                    let item = this.item_map.get(it);

                    dec += *item.value;
                });

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
    // Demo only, action should be at top level
    on_use: Action<isize>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Item {
    fn new(id: String, value: isize) -> Self {
        Self {
            id,
            value: Tl::new(value),
            on_use: Action::new(),
            listeners: Default::default(),
        }
    }

    fn clone_weak(&self) -> Self {
        let mut ret = self.clone();

        ret.listeners.be_weak();

        ret
    }

    fn setup(&self) {
        let root = self.root.make_strong();

        self.defer(register_listener_1(&self.on_use.trigger, {
            let this = self.clone_weak();
            move || {
                let mut required_value = 0;

                for it in this.on_use.trigger {
                    required_value += it;
                }

                if *this.value > required_value {
                    this.on_use.transfer();
                } else {
                    // TODO Show/Toast error to UI
                }
            }
        }));

        self.defer(register_listener_1(&root.on_upgrade_item, &root.on_use_item, {
            let this = self.clone_weak();
            move || {
                let root = this.root.make_strong();
                let mut inc = 0;
                let mut dec = 0;

                root.on_upgrade_item.verified.for_each(|it| if it == this.id {
                    inc += 10;
                });

                root.on_use_item.verified.for_each(|it| if it.0 == this.id {
                    dec += it.1;
                });

                if inc == 0 && dec == 0 { return; }
                assert!(*this.value + inc - dec >= 0);
                *this.value.to_mut() = *this.value + inc - dec;
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
