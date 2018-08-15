use self::Wrc::*;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub enum Wrc<T> {
    Strong(Rc<T>),
    Weak(Weak<T>),
}

impl<T> Clone for Wrc<T> {
    fn clone(&self) -> Self {
        match self {
            Strong(ref s) => Strong(s.clone()),
            Weak(ref w) => Weak(w.clone()),
        }
    }
}

impl<T> Deref for Wrc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            Strong(ref s) => s.deref(),
            Weak(_) => panic!("Cannot deref weak, please make_strong"),
        }
    }
}

impl<T> Wrc<T> {
    pub fn new(value: T) -> Self {
        Strong(Rc::new(value))
    }

    pub fn clone_weak(&self) -> Self {
        match *self {
            Strong(ref s) => Weak(Rc::downgrade(s)),
            Weak(ref w) => Weak(w.clone()),
        }
    }

    pub fn make_strong(&self) -> Wrc<T> {
        match *self {
            Strong(ref s) => Strong(s.clone()),
            Weak(ref w) => match w.upgrade() {
                Some(ref s) => Strong(s.clone()),
                None => panic!("Value already dropped"),
            },
        }
    }
}
