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

impl<T: Copy> TrustCell<T> {
    // TODO Use copy when possible
    #[allow(dead_code)]
    fn inner_copy(&self, from: usize, to: usize) {
        unsafe { (&mut *self.arr.get())[to] = (&*self.arr.get())[from]; }
    }
}

impl<T: Clone> TrustCell<T> {
    fn inner_clone(&self, from: usize, to: usize) {
        unsafe { (&mut *self.arr.get())[to] = (&*self.arr.get())[from].clone(); }
    }
}

// TODO In-place Copy

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

impl<T: Clone> TlValue<T> {
    fn new(value: T) -> Self {
        // TODO Flexible array with THREADS
        let tmp1 = value.clone();
        // let tmp2 = value.clone();

        Self {
            cell: TrustRc::new(TrustCell::new([value, tmp1])),
        }
    }

    fn sync(&self, from: usize, to: usize) {
        let now = time::Instant::now();
        self.cell.inner_clone(from, to);
        let duration = now.elapsed();
        println!("sync_clone takes {}s + {}ns", duration.as_secs(), duration.subsec_nanos());
    }
}

trail ManualCopy {

}

fn case01() {
    let a : TlValue<Vec<u8>> = TlValue::new(vec![1; 1024*1024]);
    
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
    
    a.sync(1, 0);
    println!("SYNC");
    println!("main a = {}", a[0]);
}

#[allow(dead_code)]
fn case02() {
    #[derive(Clone)]
    struct Home<'a> {
        progress: TlValue<f32>,
        result: TlValue<Option<&'a str>>,
        table: TlValue<Table<'a>>,
        //image_data: TlValue<[f64; 100]>,
    }

    impl<'a> Home<'a> {
        fn sync(&self, from: usize, to: usize) {
            self.progress.sync(from, to);
            self.result.sync(from, to);
            self.table.sync(from, to);
            //self.image_data.sync(from, to);
        }
    }

    #[derive(Copy, Clone, Default)]
    struct Table<'a> {
        progress: f32,
        result: &'a str,
    }

    let h = Home {
        progress: TlValue::new(0.),
        result: TlValue::new(None),
        table: TlValue::new(Default::default()),
        //image_data: TlValue::new([1.; 100]),
    };

    let handle = {
        let mut h = h.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            *h.progress = 1.;
            *h.result = Some("Big Result");
            let mut table = h.table;
            table.progress = 0.49;
            table.result = "Almost halfway";
        }).unwrap()
    };

    handle.join().unwrap();
    h.sync(1, 0);

    println!("{:?} {:?}", *h.progress, *h.result);
    println!("{:?} {:?}", h.table.progress, h.table.result);
}

fn main() {
    //case02();
    println!();
    case01();
}
