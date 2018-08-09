extern crate tl_sync;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::thread;
use tl_sync::{ Tl, Wrc, init_dirties, sync_from, sync_to };

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
    #[derive(Default, Clone)]
    struct Emitter {
        l: Rc<RefCell<Vec<Box<Fn()>>>>,
    }
    impl Emitter {
        fn add_listener(&self, f: Box<Fn()>) {
            let mut l = self.l.borrow_mut();

            l.push(f);
        }

        fn notify(&self) {
            let l = self.l.borrow();

            println!("notify to {} listeners", l.len());
            l.iter().for_each(|it| it());
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
    let e = Emitter::default();
    e.add_listener(Box::new(c1));
    e.notify();
    e.add_listener(Box::new(c2));
    e.notify();
    e.notify();

    println!();

    struct Screen {
        elements: Wrc<RefCell<Vec<Element>>>,
        data: Wrc<RefCell<Vec<u8>>>,
    }
    #[derive(Default, Clone)]
    struct Element {
        on_click: Emitter,
    }
    impl Drop for Element {
        fn drop(&mut self) {
            println!("\t\tDROP Element");
        }
    }
    impl Screen {
        fn clone_weak(&self) -> Self {
            Self {
                elements: self.elements.clone_weak(),
                data: self.data.clone_weak(),
            }
        }

        fn setup(&self) {
            let mut elements = self.elements.borrow_mut();

            elements.push(Default::default());
            let ref e = elements[0];

            {
                let this = self.clone_weak();

                e.on_click.add_listener(Box::new(move || {
                    this.layout();
                }));
            }

            {
                let this = self.clone_weak();

                e.on_click.add_listener(Box::new(move || {
                    this.animation();
                }));
            }
        }

        fn animation(&self) {
            let elements = self.elements.borrow();
            let data = self.data.borrow();

            println!(
                "animation for {} elements with data {:?}",
                elements.len(),
                data
            );
        }

        fn layout(&self) {
            let mut elements = self.elements.borrow_mut();
            let mut data = self.data.borrow_mut();

            elements.push(Default::default());
            for it in data.iter_mut() {
                *it *= 3;
            }
            println!("layout");
        }

        fn notify_all(&self) {
            let elements = self.elements.borrow().clone();

            elements.iter().for_each(|it| it.on_click.notify());
        }
    }

    let screen = Screen {
        elements: Wrc::new(RefCell::new(vec![])),
        data: Wrc::new(RefCell::new(vec![1, 2, 3, 4, 5])),
    };
    screen.setup();
    screen.notify_all();
    println!("--");
}

#[allow(dead_code)]
fn test_invalid_memory_access_wrc() {
    {
        let a = Wrc::new(Box::new(5));
        let b = a.clone_weak();

        {
            let _c = a.clone();
            println!("{:?}", *b);
        }
        println!("{:?}", *b);

        let i = b.deref();
        std::mem::drop(a);

        // This will not print 5, but a random number each run
        println!("{:?}", i);
    }

    println!();

    // Use make_strong instead of deref
    // to avoid invalid memory access as above
    {
        let a = Wrc::new(Box::new(6));
        let b = a.clone_weak();

        {
            let _c = a.clone();
            println!("{:?}", *b);
        }
        println!("{:?}", *b);

        let strong = b.make_strong();
        std::mem::drop(a);

        // This will not print 5, but a random number each run
        println!("{:?}", *strong);
    }
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
    // test_listeners();
    test_invalid_memory_access_wrc();
}
