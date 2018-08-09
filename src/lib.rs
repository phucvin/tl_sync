#![feature(box_into_raw_non_null)]

mod rc;
pub use rc::*;

mod cell;
use cell::*;

mod manual_copy;
pub use manual_copy::*;

mod tl;
pub use tl::*;

mod sync;
pub use sync::*;

const THREADS: usize = 3;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match std::thread::current().name() {
        Some(name) => match 1 + (name.as_bytes()[0] - '1' as u8) as usize {
            i if i < THREADS => i,
            _ => 0,
        },
        None => panic!("Invalid thread name to get index")
    };
}

pub fn thread_index() -> usize {
    CACHED_THREAD_INDEX.with(|c| *c)
}
