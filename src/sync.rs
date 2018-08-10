use super::*;
use std::collections::HashMap;
use std::ptr;

pub trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn get_ptr(&self) -> usize;
    fn register_listener(&self, Box<Fn()>) -> &ListenerHandle;
}

pub struct ListenerHandle {
    pub ptr: usize,
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        println!("drop listener handle");
        let l = get_listeners().to_mut(thread_index());

        if let Some(l) = l.get_mut(&self.ptr) {
            let mut found = None;
            
            for (i, it) in l.iter().enumerate() {
                if ptr::eq(&it.0, self) {
                    found = Some(i);
                    println!("found");
                    break;
                }
            }

            if let Some(i) = found {
                l.remove(i);
            }
        }
    }
}

static mut DIRTIES: Option<TrustCell<Vec<(u8, Box<Dirty>)>>> = None;
static mut LISTENERS: Option<TrustCell<HashMap<usize, Vec<(ListenerHandle, Box<Fn()>)>>>> = None;

pub fn init_dirties() {
    unsafe {
        DIRTIES = Some(TrustCell::new(Default::default()));
        LISTENERS = Some(TrustCell::new(Default::default()));
    }
}

pub fn drop_dirties() {
    let d = get_dirties();
    let l = get_listeners();

    println!();
    println!("DROP DIRTIES {} {} {}",
        d.get(0).len(), d.get(1).len(), d.get(2).len()
    );
    println!("DROP LISTENERS {} {} {}",
        l.get(0).len(), l.get(1).len(), l.get(2).len()
    );
    println!();

    unsafe {
        DIRTIES = None;
        LISTENERS = None;
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

pub fn get_listeners<'a>() -> &'a TrustCell<HashMap<usize, Vec<(ListenerHandle, Box<Fn()>)>>> {
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
    get_dirties().to_mut(from).clear();
}

pub fn notify() {
    let to = thread_index();
    let d = get_dirties().to_mut(to);
    let l = get_listeners().get(to);

    println!("NOTIFY -> {} : {}", to, d.len());
    d.iter().for_each(|it| {
        let ptr = it.1.get_ptr();
        if let Some(l) = l.get(&ptr) {
            l.iter().for_each(|it| it.1());
        }
    });
    d.clear();
}
