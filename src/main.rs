extern crate tl_sync;

use tl_sync::*;

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

    test_closure();
}
