extern crate tl_sync;

use tl_sync::*;

fn main() {
    {
        let a = Wrc::new(Box::new(5));
        let b = a.clone_weak();

        {
            let _c = a.clone();
            println!("{:?}", *b.make_strong());
        }
        println!("{:?}", *b.make_strong());

        let i = &*b.make_strong();
        std::mem::drop(a);

        // Without require make_strong everytime access a weak rc
        // This will not print 5, but a random number each run
        println!("{:?}", i);
    }
}
