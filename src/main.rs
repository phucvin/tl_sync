use std::thread;
use std::sync::Arc;
use std::cell::{UnsafeCell};
use std::time;
use std::ops::{Deref, DerefMut};

struct TrustCell<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for TrustCell<T> {}

impl<T> TrustCell<T> {
    fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    fn get(&self) -> &T {
        unsafe { &*self.inner.get() }
    }

    fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }
}

const THREADS: usize = 2;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match thread::current().name() {
        Some("main") => 0,
        Some(name) => (name.as_bytes()[0] - '0' as u8) as usize,
        None => panic!("Invalid thread name to get index")
    };
}

fn thread_index() -> usize {
    CACHED_THREAD_INDEX.with(|c| *c)
}

struct TlValue<T: Copy> {
    arr: TrustCell<[T; THREADS]>,
}

impl<T: Copy> Deref for TlValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.arr.get()[thread_index()]
    }
}

impl<T: Copy> DerefMut for TlValue<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.arr.get_mut()[thread_index()]
    }
}

impl<T: Copy> TlValue<T> {
    fn new(value: T) -> Self {
        Self {
            arr: TrustCell::new([value; THREADS]),
        }
    }

    fn reref(&self) -> Reref<T> {
        Reref {
            arr: &self.arr,
        }
    }

    fn sync(&self, from: usize, to: usize) {
        let arr = self.arr.get_mut();
        arr[to] = arr[from];
    }
}

struct Reref<'a, T: 'a + Copy> {
    arr: &'a TrustCell<[T; THREADS]>,
}

impl<'a, T: Copy> Deref for Reref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.arr.get()[thread_index()]
    }
}

impl<'a, T: Copy> DerefMut for Reref<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.arr.get_mut()[thread_index()]
    }
}

/*
struct TlVec<T: Copy> {
    arr: TrustCell<[Vec<T>; THREADS]>,
}

impl<T: Copy> TlVec<T> {
    fn new(value: Vec<T>) -> Self {
        // TODO Flexible with THREADS
        let tmp = value.clone();
        let tmp = [value, tmp];
        Self {
            arr: TrustCell::new(tmp),
        }
    }

    fn borrow(&self) -> &Vec<T> {
        &self.arr.borrow()[thread_index()]
    }

    fn borrow_mut(&self) -> &mut Vec<T> {
        &mut self.arr.borrow_mut()[thread_index()]
    }

    fn sync(&self, from: usize, to: usize) {
        let arr = self.arr.borrow_mut();
        // TODO Use current to copy
        arr[to] = arr[from].clone();
    }
}

struct TlMap<T: Clone> {
    arr: TrustCell<[Vec<T>; THREADS]>,
}

impl<T: Clone> TlMap<T> {
    fn new(value: Vec<T>) -> Self {
        // TODO Flexible with THREADS
        let tmp = value.clone();
        let tmp = [value, tmp];
        Self {
            arr: TrustCell::new(tmp),
        }
    }

    fn borrow(&self) -> &Vec<T> {
        &self.arr.borrow()[thread_index()]
    }

    fn borrow_mut(&self) -> &mut Vec<T> {
        &mut self.arr.borrow_mut()[thread_index()]
    }

    fn sync(&self, from: usize, to: usize) {
        let arr = self.arr.borrow_mut();
        // TODO Use current to copy
        arr[to] = arr[from].clone();
    }
}
*/

fn case01() {
    let a = Arc::new(TlValue::new(1));
    
    let handle = {
        let a = a.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            let mut a = a.reref();
            println!("test a = {}", *a);
            thread::sleep(time::Duration::from_millis(20));
            println!("Done heavy in test");
            *a = 2;
            println!("test a = {}", *a);
        }).unwrap()
    };

    let oa = &a;
    let a = a.reref();

    thread::sleep(time::Duration::from_millis(100));
    println!("main a = {}", *a);
    println!("Done heavy in main");
    handle.join().unwrap();
    
    oa.sync(1, 0);
    println!("SYNC");
    println!("main a = {}", *a);
}

fn case02() {
    struct Home<'a> {
        progress: TlValue<f32>,
        result: TlValue<Option<&'a str>>,
    }

    impl<'a> Home<'a> {
        fn sync(&self, from: usize, to: usize) {
            self.progress.sync(from, to);
            self.result.sync(from, to);
        }
    }

    let h = Arc::new(Home {
        progress: TlValue::new(0.),
        result: TlValue::new(None),
    });

    let handle = {
        let h = h.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            *h.progress.reref() = 1.;
            *h.result.reref() = Some("Big Result");
        }).unwrap()
    };

    handle.join().unwrap();
    h.sync(1, 0);

    println!("{:?} {:?}", *h.progress.reref(), *h.result.reref());
}

/*
fn case03() {
    let _1 = TlVec::new(vec![0, 1]);
    let _2 = TlMap::new(vec![TlValue::new("a"), TlValue::new("b")]);
    
    #[derive(Clone)]
    struct Home<'a> {
        progress: TlValue<f32>,
        result: TlValue<Option<&'a str>>,
        tmp: Arc<Vec<i32>>,
    }

    let _3 = TlMap::new(vec![
        Home {
            progress: TlValue::new(0.),
            result: TlValue::new(None),
            tmp: Arc::new(vec![]),
        },
    ]);
}
*/

fn main() {
    //case03();
    case02();
    println!();
    case01();
}
