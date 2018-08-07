#![feature(box_into_raw_non_null)]

use std::thread;
use std::cell::UnsafeCell;
use std::time;
use std::ops::{Deref, DerefMut};

const THREADS: usize = 2;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match thread::current().name() {
        Some("main") => 0,
        Some(name) => 1 + (name.as_bytes()[0] - '1' as u8) as usize,
        None => panic!("Invalid thread name to get index")
    };
}

unsafe fn thread_index() -> usize {
    CACHED_THREAD_INDEX.with(|c| *c)
}

trait ManualCopy<T> {
    fn copy_from(&mut self, &T);
}

struct TrustCell<T> {
    arr: UnsafeCell<[T; THREADS]>,
}

unsafe impl<T> Sync for TrustCell<T> {}

impl<T> TrustCell<T> {
    fn new(arr: [T; THREADS]) -> Self {
        Self {
            arr: UnsafeCell::new(arr),
        }
    }

    fn get(&self) -> &T {
        unsafe { &(&*self.arr.get())[thread_index()] }
    }

    fn get_mut(&self) -> &mut T {
        unsafe { &mut (&mut *self.arr.get())[thread_index()] }
    }
}

impl<T: ManualCopy<T>> TrustCell<T>  {
    fn inner_manual_copy(&self, from: usize, to: usize) {
        unsafe { (&mut *self.arr.get())[to].copy_from(&(&*self.arr.get())[from]); }
    }
}

struct TrustRc<T> {
    ptr: *mut T,
    is_org: bool,
}

unsafe impl<T> Send for TrustRc<T> {}
unsafe impl<T> Sync for TrustRc<T> {}

impl<T> Drop for TrustRc<T> {
    fn drop(&mut self) {
        if !self.is_org { return; }
        unsafe { std::ptr::write(self.ptr, std::mem::zeroed()); }
        unsafe { std::ptr::drop_in_place(self.ptr); }
    }
}

impl<T> Clone for TrustRc<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            is_org: false,
        }
    }
}

impl<T> Deref for TrustRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> TrustRc<T> {
    fn new(value: T) -> Self {
        Self {
            ptr: Box::into_raw_non_null(Box::new(value)).as_ptr(),
            is_org: true,
        }
    }
}

struct TlValue<T> {
    cell: TrustRc<TrustCell<T>>,
}

impl<T> Clone for TlValue<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<T> Deref for TlValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.cell.get()
    }
}

impl<T> DerefMut for TlValue<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.cell.get_mut()
    }
}

impl<T: Default + Clone + ManualCopy<T>> TlValue<T> {
    fn new(value: T) -> Self {
        let mut a: [T; THREADS] = Default::default();
        
        for i in 1..THREADS {
            a[i] = value.clone();
        }
        a[0] = value;

        Self {
            cell: TrustRc::new(TrustCell::new(a)),
        }
    }

    fn sync(&self, from: usize, to: usize) {
        self.cell.inner_manual_copy(from, to);
    }
}

impl<U: Copy + Default> ManualCopy<Vec<U>> for Vec<U> {
    fn copy_from(&mut self, other: &Vec<U>) {
        self.resize(other.len(), Default::default());
        self.copy_from_slice(other);
    }
}

fn case01() {
    let a: TlValue<Vec<u8>> = TlValue::new(vec![1; 1024*1024]);
    let b: Vec<TlValue<Vec<u8>>> = vec![TlValue::new(vec![1; 100]); 1024*100];
    
    let handle = {
        let mut a = a.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            println!("test a = {}", a[0]);
            thread::sleep(time::Duration::from_millis(20));
            println!("Done heavy in test");
            a[0] = 2;
            println!("test a = {}", a[0]);
        }).unwrap()
    };

    thread::sleep(time::Duration::from_millis(100));
    println!("main a = {}", a[0]);
    println!("Done heavy in main");
    handle.join().unwrap();
    
    {
        let now = time::Instant::now();
        a.sync(1, 0);
        // TODO Try par_iter
        b.iter().for_each(|it| it.sync(1, 0));
        let duration = now.elapsed();
        println!("sync takes {}s + {}us", duration.as_secs(), duration.subsec_micros());
    }
    println!("SYNC");
    println!("main a = {}", a[0]);
}

fn main() {
    case01();
}
