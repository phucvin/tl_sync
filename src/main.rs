extern crate tl_sync;

use std::thread;
use tl_sync::*;

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

fn main() {
    init_dirties();

    // case04();
    test_closure();
}
