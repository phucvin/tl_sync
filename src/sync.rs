use super::*;
use std::collections::HashMap;
use std::ptr;

pub trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn get_ptr(&self) -> usize;
    fn register_listener(&self, Box<FnMut()>) -> ListenerHandleRef;
    fn re_add(&self);
}

pub struct ListenerHandle {
    pub ptr: usize,
}

pub struct ListenerHandleRef {
    pub handle: &'static ListenerHandle,
}

impl Drop for ListenerHandleRef {
    fn drop(&mut self) {
        // TODO Incorrect, it maybe drop at different thread from registering
        let l = get_listeners().to_mut(thread_index());
        let mut is_zeroed = false;

        if let Some(l) = l.get_mut(&self.handle.ptr) {
            let mut found = None;

            for (i, it) in l.iter().enumerate() {
                if ptr::eq(&it.0, self.handle) {
                    found = Some(i);
                    break;
                }
            }

            if let Some(i) = found {
                l.remove(i);
                is_zeroed = l.len() == 0;
            }
        }

        if is_zeroed {
            l.remove(&self.handle.ptr);
        }
    }
}

// TODO Use context instead of static
static mut DIRTIES: Option<TrustCell<Vec<(u8, Box<Dirty>)>>> = None;
static mut LISTENERS: Option<TrustCell<HashMap<usize, Vec<(ListenerHandle, Box<FnMut()>)>>>> = None;

pub fn init_dirties() {
    unsafe {
        DIRTIES = Some(TrustCell::new(Default::default()));
        LISTENERS = Some(TrustCell::new(Default::default()));
    }
}

pub fn drop_dirties() {
    // let d = get_dirties();
    // let l = get_listeners();

    // println!();
    // println!(
    //     "DROP DIRTIES {} {} {}",
    //     d.get(0).len(),
    //     d.get(1).len(),
    //     d.get(2).len()
    // );
    // println!(
    //     "DROP LISTENERS {} {} {}",
    //     l.get(0).len(),
    //     l.get(1).len(),
    //     l.get(2).len()
    // );
    // println!();

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

pub fn get_listeners<'a>() -> &'a TrustCell<HashMap<usize, Vec<(ListenerHandle, Box<FnMut()>)>>> {
    unsafe {
        match LISTENERS {
            Some(ref l) => l,
            None => panic!("Uninitialized LISTENERS"),
        }
    }
}

pub fn sync_to(to: usize) {
    let from = thread_index();
    let df = get_dirties().to_mut(from);

    let mut tmp = vec![];
    tmp.append(df);

    // let mut v = vec![];
    tmp.iter_mut().for_each(|it| {
        if it.0 > 1 { return; }

        it.1.sync(from, to);

        if it.0 == 1 {
            it.1.re_add();
        }

        it.0 = 2;
        // v.push(it.1.get_ptr());
    });
    // println!("SYNC {} -> {} : {:?}", from, to, v);

    let dt = get_dirties().to_mut(to);
    // TODO Consider allow remain dirties in case ui thread take too long
    // assert!(
    //     dt.len() == 0,
    //     format!("Should notify before sync {} -> {} : {}", from, to, dt.len())
    // );
    dt.append(&mut tmp);
}

pub fn sync_from(from: usize) {
    let to = thread_index();
    let dt = get_dirties().to_mut(to);

    // let mut v = vec![];
    for it in dt.iter_mut() {
        if it.0 != 1 {
            continue;
        } else {
            it.0 = 2;
        }

        it.1.sync(from, to);
        // v.push(it.1.get_ptr() as usize);
    }
    // println!("SYNC {} <- {} : {:?}", to, from, v);
}

pub fn notify(d: Vec<(u8, Box<Dirty>)>) {
    let to = thread_index();
    let l = get_listeners().to_mut(to);

    // println!("NOTIFY -> {} : {}", to, d.len());
    d.iter().for_each(|it| {
        let ptr = it.1.get_ptr();
        if let Some(l) = l.get_mut(&ptr) {
            l.iter_mut().for_each(|it| it.1());
        }
    });
}

pub fn prepare_notify() -> Vec<(u8, Box<Dirty>)> {
    let to = thread_index();
    let d = get_dirties().to_mut(to);
    let mut tmp = vec![];

    tmp.append(d);
    tmp
}

pub fn peek_notify(d: Vec<usize>) -> usize {
    let to = thread_index();
    let l = get_listeners().to_mut(to);

    // println!("PEEK NOTIFY -> {} : {:?}", to, d);
    for ptr in d.iter() {
        if let Some(l) = l.get_mut(&ptr) {
            l.iter_mut().for_each(|it| it.1());
        }
    }

    d.len()
}

pub fn prepare_peek_notify() -> Vec<usize> {
    let to = thread_index();
    let d = get_dirties().to_mut(to);
    let mut tmp = vec![];

    for it in d.iter_mut() {
        if it.0 != 2 {
            continue;
        } else {
            it.0 = 3;
        }

        tmp.push(it.1.get_ptr());
    }

    tmp
}
