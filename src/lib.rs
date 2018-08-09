#![feature(box_into_raw_non_null)]

use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

mod rc;
pub use rc::*;

mod cell;
use cell::*;

mod manual_copy;
pub use manual_copy::*;

const THREADS: usize = 3;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match thread::current().name() {
        Some(name) => match 1 + (name.as_bytes()[0] - '1' as u8) as usize {
            i if i < THREADS => i,
            _ => 0,
        },
        None => panic!("Invalid thread name to get index")
    };
}

pub fn thread_index() -> usize {
    CACHED_THREAD_INDEX.with(|c| *c)
}

pub trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn is_same_pointer(&self, usize) -> bool;
    fn notify(&self);
}

pub struct Tl<T> {
    // TODO Retry TrustRc (simple Rc inside) when possible
    cell: Arc<TrustCell<T>>,
}

impl<T> Clone for Tl<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<T> Deref for Tl<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.cell.get(thread_index())
    }
}

static mut DIRTIES: Option<TrustCell<Vec<(u8, Box<Dirty>)>>> = None;
static mut LISTENERS: Option<TrustCell<HashMap<usize, Vec<&'static mut FnMut()>>>> = None;

pub fn init_dirties() {
    unsafe {
        DIRTIES = Some(TrustCell::new(Default::default()));
        LISTENERS = Some(TrustCell::new(Default::default()));
    }
}

fn get_dirties<'a>() -> &'a TrustCell<Vec<(u8, Box<Dirty>)>> {
    unsafe {
        match DIRTIES {
            Some(ref d) => d,
            None => panic!("Uninitialized DIRTIES"),
        }
    }
}

fn get_listeners<'a>() -> &'a TrustCell<HashMap<usize, Vec<&'static mut FnMut()>>> {
    unsafe {
        match LISTENERS {
            Some(ref l) => l,
            None => panic!("Uninitialized LISTENERS"),
        }
    }
}

pub fn sync_to(to: usize) {
    let from = thread_index();
    let d = get_dirties().to_mut(from);

    println!("SYNC {} -> {} : {}", from, to, d.len());
    d.iter().for_each(|it| it.1.sync(from, to));
    d.clear();
}

pub fn sync_from(from: usize) {
    let to = thread_index();
    let d = get_dirties().to_mut(to);

    println!("SYNC {} <- {} : {}", to, from, d.len());
    d.iter_mut().for_each(|it| {
        it.0 = 1;
        it.1.sync(from, to);
    });
}

// TODO Remove 'static here
impl<T: 'static + ManualCopy<T>> Tl<T> {
    pub fn to_mut(&self) -> &mut T {
        // TODO Dev check if caller come from different places
        // even in different sync calls, then should panic

        {
            let d = get_dirties().to_mut(thread_index());
            let tmp = Box::new(self.clone());
            let ptr = tmp.cell.arr.get();

            let mut is_unique = true;
            for it in d.iter() {
                if it.1.is_same_pointer(ptr as usize) {
                    if it.0 > 1 {
                        panic!("Only allow one mutation each sync");
                    }
                    is_unique = false;
                    break;
                }
            }

            if is_unique {
                d.push((2, tmp));
            }
        }

        self.cell.to_mut(THREADS - 1)
    }
}

impl<T: ManualCopy<T>> Dirty for Tl<T> {
    fn sync(&self, from: usize, to: usize) {
        self.cell.inner_manual_copy(from, to);
    }

    fn is_same_pointer(&self, other: usize) -> bool {
        self.cell.arr.get() as usize == other
    }

    fn notify(&self) {}
}

impl<T: Default + Clone + ManualCopy<T>> Default for Tl<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Tl<T> {
    pub fn new(value: T) -> Self {
        // TODO Find a way that flexible with thread,
        // but also not using std::mem::zeroed (error with Rc)
        let tmp1 = value.clone();
        let tmp2 = value.clone();
        let a = [value, tmp1, tmp2];

        Self {
            cell: Arc::new(TrustCell::new(a)),
        }
    }
}
