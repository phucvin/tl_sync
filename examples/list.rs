extern crate tl_sync;

use std::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
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
    item_map: Tl<HashMap<String, Item>>,
    on_upgrade_item: VerifyAction<String>,
    on_iap: Action<usize>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Root {
    fn new() -> Self {
        Self {
            money: Tl::new(1000),
            item_map: Tl::new(HashMap::new()),
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
                    let item = this.item_map.get(it).unwrap();

                    dec += *item.value;
                }

                println!("money: {}", *this.money + inc - dec);
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
    value: Tl<usize>,
    // Demo only, action should be at top level
    on_use: VerifyAction<usize>,
    listeners: Wrc<Mutex<Vec<ListenerHandleRef>>>,
}

impl Item {
    fn new(id: String, value: usize) -> Self {
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

    fn setup(&self, on_upgrade_item: &VerifyAction<String>) {
        self.defer(register_listener_2(&self.on_use.verified, &self.on_use.trigger, {
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

        self.defer(register_listener_2(&self.on_use.trigger, &self.on_use.verified, {
            let this = self.clone_weak();
            // let on_upgrade_item = on_upgrade_item.clone();
            move || {
                let mut inc = 0;
                let mut dec = 0;

                // for it in on_upgrade_item.verified.iter() {
                //     if *it == this.id {
                //         inc += 10;
                //     }
                // }

                for it in this.on_use.verified.iter() {
                    dec += it;
                }

                println!("{} value: {}", this.id, *this.value + inc - dec);
                if inc == 0 && dec == 0 { return; }
                assert!(dec <= *this.value + inc);
                *this.value.to_mut() = *this.value + inc - dec;
            }
        }));
    }

    fn defer(&self, h: ListenerHandleRef) {
        let mut l = self.listeners.lock().unwrap();
        l.push(h);
    }
}

impl UiSetup for Root {
    fn setup_ui(&self) {
        //
    }
}

impl ComputeSetup for Root {
    fn setup_compute(&self) {
        self.setup();

        let item_map = self.item_map.to_mut();
        item_map.insert(
            "i001".into(),
            Item::new("i001".into(), 19)
        );
        item_map.get("i001").unwrap().setup(&self.on_upgrade_item);

        self.on_upgrade_item.trigger.fire("i001".into());
    }
}

fn main() {
    let stop = {
        let root = Root::new();
        let (mut tick, stop) = setup(root.clone(), Duration::from_millis(1));
        
        for _ in 1..10 {
            tick();
        }

        stop
    };

    stop();
}
