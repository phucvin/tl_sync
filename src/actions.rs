use super::*;
use std::ops::Deref;

struct Wrapper<T>(Vec<T>);

pub struct Action<T> {
    queue: Tl<Wrapper<T>>,
}

impl<T> Clone for Action<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> Deref for Action<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.queue.0
    }
}

impl<T: 'static> Action<T> {
    pub fn new() -> Self {
        Self {
            queue: Tl::new_advanced([
                Wrapper(vec![]),
                Wrapper(vec![]),
                Wrapper(vec![]),
            ]),
        }
    }

    pub fn fire(&self, a: T) {
        println!("before fire: {}", self.queue.0.len());
        self.queue.to_mut_advanced().0.push(a);
        println!("after fire: {}", self.queue.0.len());
    }
}

impl<T: 'static> RegisterListener for Action<T> {
    fn register_listener(&self, f: Box<FnMut()>) -> ListenerHandleRef {
        self.queue.register_listener(f)
    }
}

impl<T> ManualCopy<Wrapper<T>> for Wrapper<T> {
    fn copy_from(&mut self, other: &mut Wrapper<T>) {
        println!("old actions: {}", self.0.len());
        self.0.clear();
        self.0.append(&mut other.0);
    }
}