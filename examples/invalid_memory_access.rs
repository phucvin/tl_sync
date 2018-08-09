extern crate tl_sync;

use tl_sync::*;

fn main() {
    {
        let a = Wrc::new(Box::new(5));
        let b = a.clone_weak();

        {
            let _c = a.clone();
            println!("{:?}", *b);
        }
        println!("{:?}", *b);

        let i = &*b;
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

        let i = &*b;
        let _strong = b.make_strong();
        std::mem::drop(a);

        // This will not print 5, but a random number each run
        println!("{:?}", i);
    }
}