use super::*;
use std::collections::HashMap;
use std::ptr;
use uuid::Uuid;

pub trait GetPtr {
    fn get_ptr(&self) -> usize;
}

pub trait Dirty: GetPtr {
    fn sync(&self, from: usize, to: usize);
    fn clear(&self, to: usize);
    fn re_add(&self);
}

#[derive(Clone)]
pub struct ListenerHandle {
    ptr: usize,
    ptr_bk: usize,
    uuid: Option<Uuid>,
}

pub struct ListenerHandleRef {
    ptr1: usize,
    handles: Vec<ListenerHandle>,
    from: usize,
}

impl Drop for ListenerHandleRef {
    fn drop(&mut self) {
        println!("drop {} - {:?}, {}", self.handles[0].ptr, self.handles[0].uuid, self.handles[0].ptr_bk);
        // TODO Maybe drop at different thread
        // accessing listeners here is not thread-safe
        let l = get_listeners().to_mut(self.from);

        for handle in self.handles.iter() {
            let mut is_zeroed = false;

            if let Some(l) = l.get_mut(&handle.ptr) {
                let a = l.len();
                l.retain(|it| it.0.uuid != handle.uuid);
                is_zeroed = l.len() == 0;
                println!("{} - {:?}: {} - {}", handle.ptr, handle.uuid, a, l.len());
                for it in l.iter() {
                    println!("it: {}", it.0.ptr);
                }
            } else {
                println!("empty");
            }

            if is_zeroed {
                l.remove(&handle.ptr);
            }
        }
    }
}

pub fn register_listener_1<T1, F>(t1: &T1, mut f: F) -> ListenerHandleRef
where
    T1: GetPtr,
    F: 'static + FnMut() + Clone,
{
    let uuid = Some(Uuid::new_v4());

    let h1 = {
        let l = get_listeners().to_mut(thread_index());
        let ptr1 = t1.get_ptr();
        if !l.contains_key(&ptr1) {
            l.insert(ptr1, vec![]);
        }

        let l = l.get_mut(&ptr1).unwrap();
        let h = ListenerHandle { ptr: ptr1, ptr_bk: ptr1, uuid };
        l.push((h.clone(), Box::new(f.clone())));
        for it in l.iter() {
            println!("it: {}", it.0.ptr);
        }

        println!("reg_1 {} - {}", ptr1, &l[l.len() - 1].0 as *const ListenerHandle as usize);
        h
    };

    f();

    ListenerHandleRef {
        ptr1: t1.get_ptr(),
        handles: vec![h1],
        from: thread_index(),
    }
}

pub fn register_listener_2<T1, T2, F>(t1: &T1, t2: &T2, mut f: F) -> ListenerHandleRef
where
    T1: GetPtr,
    T2: GetPtr,
    F: 'static + FnMut() + Clone,
{
    let uuid = Some(Uuid::new_v4());

    let h1 = {
        let l = get_listeners().to_mut(thread_index());
        let ptr1 = t1.get_ptr();
        if !l.contains_key(&ptr1) {
            l.insert(ptr1, vec![]);
        }

        let l = l.get_mut(&ptr1).unwrap();
        let h = ListenerHandle { ptr: ptr1, ptr_bk: ptr1, uuid };
        l.push((h.clone(), Box::new(f.clone())));

        println!("reg_2 {}", &l[l.len() - 1].0 as *const ListenerHandle as usize);
        h
    };

    let h2 = {
        let l = get_listeners().to_mut(thread_index());
        let ptr2 = t2.get_ptr();
        if !l.contains_key(&ptr2) {
            l.insert(ptr2, vec![]);
        }

        let l = l.get_mut(&ptr2).unwrap();
        let h = ListenerHandle { ptr: ptr2, ptr_bk: ptr2, uuid };
        l.push((h.clone(), Box::new(f.clone())));

        println!("reg_2 {}", &l[l.len() - 1].0 as *const ListenerHandle as usize);
        h
    };

    f();

    ListenerHandleRef {
        ptr1: t1.get_ptr(),
        handles: vec![h1, h2],
        from: thread_index(),
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

pub fn ensure_empty_dirties() {
    let d = get_dirties();
    let l = get_listeners();

    for i in 0..THREADS {
        assert!(d.get(i).len() == 0);
        // assert!(l.get(i).len() == 0);
    }
}

pub fn drop_dirties() {
    let d = get_dirties();
    let l = get_listeners();

    println!();
    println!(
        "DROP DIRTIES {} {} {}",
        d.get(0).len(),
        d.get(1).len(),
        d.get(2).len()
    );
    println!(
        "DROP LISTENERS {} {} {}",
        l.get(0).len(),
        l.get(1).len(),
        l.get(2).len()
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
        if it.0 >= 4 {
            return;
        }

        it.1.sync(from, to);

        if it.0 == 1 {
            it.1.re_add();
        }

        it.0 = 2;
        // v.push(it.1.get_ptr());
    });
    tmp.retain(|it| it.0 < 4);
    tmp.iter_mut().for_each(|it| it.0 = 4);
    // println!("SYNC {} -> {} : {:?}", from, to, v);

    let dt = get_dirties().to_mut(to);
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

pub fn peek_notify(d: Vec<usize>) -> usize {
    let to = thread_index();
    let l = get_listeners().to_mut(to);
    let mut uuids = vec![];

    // println!("PEEK NOTIFY -> {} : {:?}", to, d);
    for ptr in d.iter() {
        if let Some(l) = l.get_mut(&ptr) {
            l.iter_mut().for_each(|it| {
                if let Some(uuid) = it.0.uuid {
                    if uuids.contains(&uuid) {
                        return;
                    }
                    uuids.push(uuid);
                }
                it.1();
            });
        }
    }

    d.len()
}

pub fn prepare_peek_notify() -> Vec<usize> {
    let to = thread_index();
    let d = get_dirties().to_mut(to);
    let mut tmp = vec![];

    for it in d.iter_mut() {
        if it.0 != 4 {
            if it.0 != 2 {
                continue;
            } else {
                it.0 = 3;
            }
        } else {
            it.0 = 5;
        }

        tmp.push(it.1.get_ptr());
    }

    tmp
}

pub fn sync_clear() {
    let to = thread_index();
    let d = get_dirties().to_mut(to);

    d.retain(|it| {
        if it.0 == 5 {
            it.1.clear(to);
            false
        } else {
            true
        }
    });
}
