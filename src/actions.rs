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

pub struct ActionCreator<T> {
    // TODO Retry TrustRc (simple Rc inside) when possible
    listeners: Arc<TrustCell<Vec<Box<FnMut(&T)>>>>,
    queue: Arc<Mutex<Vec<(u8, Box<T>)>>>,
}

impl<T> Clone for ActionCreator<T> {
    fn clone(&self) -> Self {
        Self {
            listeners: self.listeners.clone(),
            queue: self.queue.clone(),
        }
    }
}

impl<T> ActionCreator<T> {
    pub fn new() -> Self {
        // TODO Find a way that flexible with thread,
        let a = [
            Default::default(),
            Default::default(),
            Default::default(),
        ];

        Self {
            listeners: Arc::new(TrustCell::new(a)),
            queue: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn fire(&self, a: T) {
        self.queue.lock().unwrap().push((0, Box::new(a)));
        // TODO mark as dirty (possible not notify enough ??)
    }

    // TODO Return drop handle
    pub fn register_listener(&self, f: Box<FnMut(&T)>) -> () {
        self.listeners.to_mut(thread_index()).push(f);
    }

    pub fn notify(&self) {
        let mask = (thread_index() + 1) as u8;
        let mut queue = {
            let mut tmp = vec![];
            tmp.append(&mut self.queue.lock().unwrap());
            tmp
        };
        for it in queue.iter_mut() {
            if it.0 == 0 || it.0 == mask {
                self.notify_with(&it.1);
                it.0 += (3 - mask) as u8;
            }
        }
        queue.retain(|it| it.0 < 3);
        {
            self.queue.lock().unwrap().append(&mut queue);
        }
    }

    fn notify_with(&self, a: &T) {
        let listeners = self.listeners.to_mut(thread_index());
        let mut tmp = vec![];

        tmp.append(listeners);
        for i in 0..tmp.len() {
            tmp.get_mut(i).unwrap()(a);
        }
        listeners.append(&mut tmp);
    }
}
