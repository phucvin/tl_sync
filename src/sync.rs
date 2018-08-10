use super::*;
use std::collections::HashMap;

pub trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn get_ptr(&self) -> usize;
    fn register_listener(&self, Box<Fn()>);
}

static mut DIRTIES: Option<TrustCell<Vec<(u8, Box<Dirty>)>>> = None;
static mut LISTENERS: Option<TrustCell<HashMap<usize, Vec<Box<Fn()>>>>> = None;

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

pub fn get_listeners<'a>() -> &'a TrustCell<HashMap<usize, Vec<Box<Fn()>>>> {
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
    get_dirties().to_mut(to).append(d);
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

pub fn notify() {
    let to = thread_index();
    let d = get_dirties().to_mut(to);
    let l = get_listeners().get(to);

    println!("NOTIFY {} : {}", to, d.len());
    d.iter().for_each(|it| {
        let ptr = it.1.get_ptr();
        let l = l.get(&ptr).unwrap();
        l.iter().for_each(|it| it());
    });
    d.clear();
}

pub fn drop_dirties() {
    unsafe {
        DIRTIES = None;
        LISTENERS = None;
    }
}
