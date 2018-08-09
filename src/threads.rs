use std::thread;

pub const THREADS: usize = 3;
pub const MUTATE_THREAD_INDEX: usize = 2;

thread_local! {
    static CACHED_THREAD_INDEX: usize = match thread::current().name() {
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
