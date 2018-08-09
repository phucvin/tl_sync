#![feature(box_into_raw_non_null)]
#![feature(const_fn)]

extern crate rayon;

use rayon::prelude::*;
use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time;

const THREADS: usize = 3;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match thread::current().name() {
        Some("main") => 0,
        Some(name) => 1 + (name.as_bytes()[0] - '1' as u8) as usize,
        None => panic!("Invalid thread name to get index")
    };
}

fn thread_index() -> usize {
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
    const fn new(arr: [T; THREADS]) -> Self {
        Self {
            arr: UnsafeCell::new(arr),
        }
    }

    fn get(&self, i: usize) -> &T {
        unsafe { &(&*self.arr.get())[i] }
    }

    fn to_mut(&self, i: usize) -> &mut T {
        unsafe { &mut (&mut *self.arr.get())[i] }
    }
}

impl<T: ManualCopy<T>> TrustCell<T> {
    fn inner_manual_copy(&self, from: usize, to: usize) {
        unsafe {
            (&mut *self.arr.get())[to].copy_from(&(&*self.arr.get())[from]);
        }
    }
}

trait Dirty {
    fn sync(&self, from: usize, to: usize);
    fn is_same_pointer(&self, usize) -> bool;
    fn notify(&self);
}

struct Tl<T> {
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

fn init_dirties() {
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

fn sync_to(to: usize) {
    let from = thread_index();
    let d = get_dirties().to_mut(from);

    println!("SYNC {} -> {} : {}", from, to, d.len());
    d.iter().for_each(|it| it.1.sync(from, to));
    d.clear();
}

fn sync_from(from: usize) {
    let to = thread_index();
    let d = get_dirties().to_mut(to);

    println!("SYNC {} <- {} : {}", to, from, d.len());
    d.iter_mut().for_each(|it| {
        it.0 = 1;
        it.1.sync(from, to);
    });
}

impl<T: 'static + ManualCopy<T>> Tl<T> {
    fn to_mut(&self) -> &mut T {
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
    fn new(value: T) -> Self {
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

impl ManualCopy<usize> for usize {
    fn copy_from(&mut self, other: &usize) {
        *self = *other;
    }
}

impl ManualCopy<String> for String {
    fn copy_from(&mut self, other: &String) {
        self.clear();
        self.push_str(other);
    }
}

impl<T: Clone> ManualCopy<Option<T>> for Option<T> {
    fn copy_from(&mut self, other: &Option<T>) {
        *self = match *other {
            None => None,
            Some(ref v) => Some(v.clone()),
        }
    }
}

impl<T1: Clone, T2: Clone> ManualCopy<(T1, T2)> for (T1, T2) {
    fn copy_from(&mut self, other: &(T1, T2)) {
        // TODO If U: copy, try to use memcpy (=)
        self.0 = other.0.clone();
        self.1 = other.1.clone();
    }
}

impl<U: Clone> ManualCopy<Vec<U>> for Vec<U> {
    fn copy_from(&mut self, other: &Vec<U>) {
        // TODO If U: Copy, try to use memcpy (copy_from_slice)
        let slen = self.len();
        let olen = other.len();

        if slen < olen {
            for i in slen..olen {
                self.push(other[i].clone());
            }
        } else if slen > olen {
            self.truncate(olen)
        }

        for i in 0..(std::cmp::min(slen, olen)) {
            self[i] = other[i].clone();
        }
    }
}

#[allow(dead_code)]
fn case01() {
    let a: Tl<Vec<u8>> = Tl::new(vec![1; 1024 * 1024]);
    let mut b: Vec<Tl<Vec<u8>>> = vec![];
    for _i in 1..100 {
        b.push(Tl::new(vec![1; 1024 * 100]));
    }

    let handle = {
        let a = a.clone();
        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                println!("test a = {}", a[0]);
                thread::sleep(time::Duration::from_millis(5));
                println!("Done heavy in test");
                a.to_mut()[0] = 2;
                a.sync(2, 1);
                println!("test a = {}", a[0]);
            }).unwrap()
    };

    thread::sleep(time::Duration::from_millis(10));
    println!("main a = {}", a[0]);
    println!("Done heavy in main");
    handle.join().unwrap();

    {
        let now = time::Instant::now();
        a.sync(1, 0);
        b.par_iter().for_each(|it| it.sync(1, 0));
        let duration = now.elapsed();
        println!(
            "sync takes {}s + {}ms",
            duration.as_secs(),
            duration.subsec_millis()
        );
    }
    println!("main a = {}", a[0]);
}

#[allow(dead_code)]
fn case02() {
    #[derive(Clone, Default)]
    struct Holder {
        inner: Tl<Vec<Wrapper>>,
    }
    impl ManualCopy<Holder> for Holder {
        fn copy_from(&mut self, _other: &Holder) {
            panic!("SHOULD NEVER BE CALLED");
        }
    }
    impl Debug for Holder {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Holder {{ inner: {:?} }}", unsafe {
                &*self.inner.cell.arr.get()
            })
        }
    }
    #[derive(Clone, Default)]
    struct Wrapper {
        value: Tl<usize>,
    }
    impl Debug for Wrapper {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Wrapper {{ value: {:?} }}", unsafe {
                &*self.value.cell.arr.get()
            })
        }
    }

    let c: Arc<Holder> = Arc::new(Default::default());
    c.inner.to_mut().push(Wrapper { value: Tl::new(22) });
    sync_from(2);
    sync_to(1);
    println!("main pre {:?}", c);

    let handle = {
        let mut c = c.clone();
        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                println!("test pre {:?}", c);
                let tmp = c.inner[0].value.to_mut();
                *tmp = 100;
                c.inner.to_mut().push(Wrapper { value: Tl::new(33) });
                println!("test change {:?}", c);
                sync_from(2);
                sync_to(0);
                println!("test post {:?}", c);
            }).unwrap()
    };

    handle.join().unwrap();
    println!("main post {:?}", c);
    println!("{:?}", *c.inner);
}

#[allow(dead_code)]
fn case03() {
    #[derive(Clone)]
    struct SceneRoot {
        stack: Tl<Vec<Scene>>,
        popup: Tl<Option<Scene>>,
        top_ui: Tl<Button>,
    }
    impl Default for SceneRoot {
        fn default() -> Self {
            Self {
                stack: Default::default(),
                popup: Tl::new(None),
                top_ui: Tl::new(Default::default()),
            }
        }
    }
    impl ManualCopy<SceneRoot> for SceneRoot {
        fn copy_from(&mut self, _other: &SceneRoot) {
            panic!("SHOULD NEVER BE CALLED");
        }
    }
    #[derive(Default, Clone)]
    struct Scene {
        title: Tl<String>,
        buttons: Tl<Vec<Button>>,
        image_data: Arc<Vec<u8>>,
    }
    #[derive(Default, Clone)]
    struct Button {
        pos: Tl<(u32, u32)>,
        txt: Tl<String>,
    }

    let r = Arc::new(SceneRoot::default());
    r.stack.to_mut().push(Scene {
        title: Tl::new("Home".into()),
        buttons: Tl::new(vec![Button {
            pos: Tl::new((100, 50)),
            txt: Tl::new("Click Me!".into()),
        }]),
        image_data: Arc::new(vec![8; 1024]),
    });

    sync_from(2);
    sync_to(1);
    println!(
        "{}: {} @ {:?}",
        *r.stack[0].title, *r.stack[0].buttons[0].txt, *r.stack[0].buttons[0].pos
    );

    let handle = {
        let r = r.clone();
        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                for _ in 1..10 {
                    {
                        let tmp = &r.stack[0].buttons[0];
                        *tmp.txt.to_mut() = "Play".into();
                        *tmp.pos.to_mut() = (tmp.pos.0 - 2, tmp.pos.1 + 3);
                    }

                    sync_from(2);
                }

                thread::sleep(time::Duration::from_millis(10));
                sync_to(0);
            }).unwrap()
    };

    thread::sleep(time::Duration::from_millis(10));
    handle.join().unwrap();
    println!(
        "{}: {} @ {:?}",
        *r.stack[0].title, *r.stack[0].buttons[0].txt, *r.stack[0].buttons[0].pos
    );
}

#[allow(dead_code)]
fn case04() {
    let tmp = Tl::new(1);
    let a = Tl::new((true, tmp));
    println!("{}", *a.1);
    {
        let a = a.clone();
        thread::Builder::new()
            .name("1_test".into())
            .spawn(move || {
                // *(*a).1.to_mut() = 3;
                a.to_mut().1 = Tl::new(3);
                let _not_leak = Tl::new(100);
                sync_from(2);
                sync_to(0);
            }).unwrap()
            .join()
            .unwrap();
    }
    println!("{}", *a.1);
}

#[allow(dead_code)]
fn test_closure() {
    struct TestClosure {
        a: i32,
    }
    impl TestClosure {
        fn abc(&self, b: i32, c: i32) {
            println!("{} {} {}", self.a, b, c);
        }

        fn ma(&mut self, v: i32) {
            self.a = v;
        }
    }
    fn call_bc(f: &Fn(i32, i32), b: i32, c: i32) {
        f(b, c);
    }
    fn fake_bc(b: i32, c: i32) {
        println!("{} {}", b, c);
    }

    let mut tc = TestClosure { a: 100 };
    call_bc(&fake_bc, 5, 88);
    call_bc(&|b, c| tc.abc(b, c), 299, 0);
    tc.a = 10;
}

#[allow(dead_code)]
fn test_listeners() {
    #[derive(Default)]
    struct Emitter {
        l: Vec<Box<Fn()>>,
    }
    impl Emitter {
        fn add_listener(&mut self, f: Box<Fn()>) {
            self.l.push(f);
        }

        fn notify(&self) {
            println!("notify to {} listeners", self.l.len());
            self.l.iter().for_each(|it| it());
        }
    }

    let a = std::cell::Cell::new(0);
    let c1 = || {
        println!("c1");
    };
    let c2 = move || {
        println!("c2");
        a.set(a.get() + 1);
        println!("a = {}", a.get());
    };
    let mut e = Emitter { l: vec![] };
    e.add_listener(Box::new(c1));
    e.notify();
    e.add_listener(Box::new(c2));
    e.notify();
    e.notify();

    println!();

    #[derive(Clone)]
    struct Screen {
        elements: Rc<RefCell<Vec<Element>>>,
    }
    impl Drop for Screen {
        fn drop(&mut self) {
            println!("\t\tDROP Screen");

            // Have to manually drop to avoid cycle references
            match self.elements.try_borrow_mut() {
                Ok(mut elements) => elements.clear(),
                Err(_) => (),
            }
        }
    }
    #[derive(Default)]
    struct Element {
        on_click: Emitter,
    }
    impl Drop for Element {
        fn drop(&mut self) {
            println!("\t\tDROP Element");
        }
    }
    impl Screen {
        fn setup(&mut self) {
            let mut elements = self.elements.borrow_mut();

            elements.push(Default::default());

            {
                let this = self.clone_weak();
                elements[0]
                    .on_click
                    .add_listener(Box::new(move || this.animation()));
            }
        }

        fn animation(&self) {
            println!("animation");
        }

        fn notify_all(&self) {
            let elements = self.elements.borrow();

            elements.iter().for_each(|it| it.on_click.notify());
        }

        fn clone_weak(&self) -> Screen {
            let ret = self.clone();
            ret
        }
    }

    let mut screen = Screen {
        elements: Rc::new(RefCell::new(vec![])),
    };
    screen.setup();
    screen.notify_all();
}

fn main() {
    init_dirties();

    // case01();
    // println!();
    // case02();
    // println!();
    // case03();
    // println!();
    // case04();

    // test_closure();
    test_listeners();
}
