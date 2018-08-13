use super::*;
use std::ops::Deref;
use std::sync::Arc;

pub struct Stl<T> {
    // TODO Retry TrustRc (simple Rc inside) when possible
    cell: Arc<TrustCell<T>>,
}

impl<T> Clone for Stl<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<T> Deref for Stl<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.cell.get(thread_index())
    }
}

impl<T: Default> Stl<T> {
    pub fn new() -> Self {
        // TODO Find a way that flexible with thread,
        let a = [
            Default::default(),
            Default::default(),
            Default::default(),
        ];

        Self {
            cell: Arc::new(TrustCell::new(a)),
        }
    }

    pub fn to_mut(&self) -> &mut T {
        self.cell.to_mut(MUTATE_THREAD_INDEX)
    }
}

impl<'a, T> Stl<Vec<Box<Fn(&T)>>> {
    pub fn register_listener(&self, f: Box<Fn(&T)>) -> () {
        self.to_mut().push(f);
    }

    pub fn notify(&self, e: &T) {
        let v = self.to_mut();
        let mut tmp = vec![];

        tmp.append(v);
        for i in 0..tmp.len() {
            tmp[i](e);
        }
        v.append(&mut tmp);
    }
}
