use std::sync::{Arc, Mutex};
use std::any::Any;

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
