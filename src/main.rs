use std::thread;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::time;
use std::ops::Deref;

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
    fn inner_copy(&self, from: usize, to: usize) {
        unsafe { (&mut *self.arr.get())[to] = (&*self.arr.get())[from]; }
    }
}

impl<T: Clone> TrustCell<T> {
    fn inner_clone(&self, from: usize, to: usize) {
        unsafe { (&mut *self.arr.get())[to] = (&*self.arr.get())[from].clone(); }
    }
}

struct TrustRc<T> {
    ptr: *const T,
    value: Option<T>,
}

impl<T> Clone for TrustRc<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            value: None,
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
            ptr: &value as *const T,
            value: Some(value),
        }
    }
}

struct TlValue<T: Copy> {
    cell: TrustCell<T>,
}

impl<T: Copy> Deref for TlValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.cell.get()
    }
}

impl<T: Copy> TlValue<T> {
    fn new(value: T) -> Self {
        Self {
            cell: TrustCell::new([value; THREADS]),
        }
    }

    fn to_mut(&self) -> &mut T {
        self.cell.get_mut()
    }

    fn sync(&self, from: usize, to: usize) {
        self.cell.inner_copy(from, to);
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
    struct A(TlValue<i32>);

    let a = Arc::new(A(TlValue::new(1)));
    
    let handle = {
        let a = a.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            println!("test a = {}", *a.0);
            thread::sleep(time::Duration::from_millis(20));
            println!("Done heavy in test");
            *a.0.to_mut() = 2;
            println!("test a = {}", *a.0);
        }).unwrap()
    };

    thread::sleep(time::Duration::from_millis(100));
    println!("main a = {}", *a.0);
    println!("Done heavy in main");
    handle.join().unwrap();
    
    a.0.sync(1, 0);
    println!("SYNC");
    println!("main a = {}", *a.0);
}

fn case02() {
    struct Home<'a> {
        progress: TlValue<f32>,
        result: TlValue<Option<&'a str>>,
        table: TlValue<Table<'a>>,
    }

    impl<'a> Home<'a> {
        fn sync(&self, from: usize, to: usize) {
            self.progress.sync(from, to);
            self.result.sync(from, to);
            self.table.sync(from, to);
        }
    }

    #[derive(Copy, Clone, Default)]
    struct Table<'a> {
        progress: f32,
        result: &'a str,
    }

    let h = Arc::new(Home {
        progress: TlValue::new(0.),
        result: TlValue::new(None),
        table: TlValue::new(Default::default())
    });

    let handle = {
        let h = h.clone();
        thread::Builder::new().name("1_test".into()).spawn(move || {
            *h.progress.to_mut() = 1.;
            *h.result.to_mut() = Some("Big Result");
            let table = h.table.to_mut();
            table.progress = 0.49;
            table.result = "Almost halfway";
        }).unwrap()
    };

    handle.join().unwrap();
    h.sync(1, 0);

    println!("{:?} {:?}", *h.progress, *h.result);
    let table  = &h.table;
    println!("{:?} {:?}", table.progress, table.result);
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
