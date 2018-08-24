extern crate tl_sync;

use std::sync::Mutex;
use std::time::Duration;
use tl_sync::*;

#[derive(Clone)]
struct VerifyAction<T: 'static + Clone> {
    trigger: Action<T>,
    verified: Action<T>,
}

impl<T: 'static + Clone> VerifyAction<T> {
    fn new() -> Self {
        Self {
            trigger: Action::new(),
            verified: Action::new(),
        }
    }

    fn transfer(&self) {
        for it in self.trigger.iter() {
            self.verified.fire(it.clone());
        }
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
        self.defer(register_listener_1(&self.on_upgrade_item.trigger, {
            let this = self.clone_weak();
            move || {
                let mut required_money = 0;

                for item_id in this.on_upgrade_item.trigger.iter() {
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

        self.defer(register_listener_2(&self.on_upgrade_item.verified, &self.on_iap, {
            let this = self.clone_weak();
            move || {
                let mut inc = 0;
                let mut dec = 0;

                for it in this.on_iap.iter() {
                    inc += it;
                }

                for it in this.on_upgrade_item.verified.iter() {
                    let item = this.item_map.get(it);

                    dec += *item.value;
                }

                assert!(*this.money + inc <= 2_000_000_000, "overflow");
                assert!(dec <= *this.money + inc, "invalid money inc/dec");
                *this.money.to_mut() = *this.money + inc - dec;
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
    on_use: VerifyAction<isize>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Item {
    fn new(id: String, value: isize) -> Self {
        Self {
            id,
            value: Tl::new(value),
            on_use: VerifyAction::new(),
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

                for it in this.on_use.trigger.iter() {
                    required_value += it;
                }

                if *this.value > required_value {
                    this.on_use.transfer();
                } else {
                    // TODO Show/Toast error to UI
                }
            }
        }));

        self.defer(register_listener_2(&root.on_upgrade_item.verified, &self.on_use.verified, {
            let this = self.clone_weak();
            move || {
                let root = this.root.make_strong();
                let mut inc = 0;
                let mut dec = 0;

                for it in root.on_upgrade_item.verified.iter() {
                    if it == this.id {
                        inc += 10;
                    }
                }

                for it in this.on_use.verified.iter() {
                    dec += it;
                }

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
        let root = Root::new();
        let (mut tick, stop) = setup(root.clone(), Duration::from_millis(15));
        
        for _ in 1..100 {
            tick();
        }

        stop
    };

    stop();
}
