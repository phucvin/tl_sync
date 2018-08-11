use std::sync::{Arc, Mutex};
use std::any::Any;
use super::*;

// TODO Use context instead of static
static mut ACTIONS: Option<Arc<Mutex<Vec<(u8, Box<Any>)>>>> = None;

pub fn init_actions() {
    unsafe {
        ACTIONS = Some(Default::default());
    }
}

pub fn drop_actions() {
    unsafe {
        ACTIONS = None;
    }
}

pub fn get_actions() -> Arc<Mutex<Vec<(u8, Box<Any>)>>> {
    unsafe {
        match ACTIONS {
            Some(ref d) => d.clone(),
            None => panic!("Uninitialized ACTIONS"),
        }
    }
}

pub fn fire(action: Box<Any>) {
    let actions = get_actions();
    actions.lock().unwrap().push((0, action));
}

pub fn notify_actions<T: UiSetup + ComputeSetup>(root: &T, mask: u8) {
    let mut actions = {
        let actions = get_actions();
        let mut actions = actions.lock().unwrap();
        let mut tmp = vec![];
        tmp.append(&mut actions);
        tmp
    };
    for it in actions.iter_mut() {
        if it.0 == 0 || it.0 == mask {
            if mask == 1 {
                root.ui_act_on(&it.1);
                it.0 += 2;
            } else if mask == 2 {
                root.compute_act_on(&it.1);
                it.0 += 1;
            }
        }
    }
    actions.retain(|it| it.0 < 3);
    {
        let tmp = get_actions();
        let mut tmp = tmp.lock().unwrap();
        tmp.append(&mut actions);
    }
}
