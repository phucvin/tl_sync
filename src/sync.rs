use std::collections::HashMap;
use super::*;

pub trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn is_same_pointer(&self, usize) -> bool;
    fn notify(&self);
}

static mut DIRTIES: Option<TrustCell<Vec<(u8, Box<Dirty>)>>> = None;
static mut LISTENERS: Option<TrustCell<HashMap<usize, Vec<&'static mut FnMut()>>>> = None;

pub fn init_dirties() {
    unsafe {
        DIRTIES = Some(TrustCell::new(Default::default()));
        LISTENERS = Some(TrustCell::new(Default::default()));
    }
}

pub fn get_dirties<'a>() -> &'a TrustCell<Vec<(u8, Box<Dirty>)>> {
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