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
            Weak(ref w) => match w.upgrade() {
                Some(ref s) => {
                    let tmp = s.deref();
                    let tmp = tmp as *const T;

                    // FIXME Avoid unsafe
                    // Unsafe because right now rc for T is 2,
                    // But there after this fn returns, it is 1,
                    // So if rc for T drop to 0 in the future,
                    // this ref to T is point to invalid memory
                    unsafe { &*tmp }
                }
                None => panic!("Value already dropped"),
            },
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
