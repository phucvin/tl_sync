use super::*;
use std::cell::UnsafeCell;

pub struct TrustCell<T> {
    pub arr: UnsafeCell<[T; THREADS]>,
}

unsafe impl<T> Sync for TrustCell<T> {}

impl<T> TrustCell<T> {
    pub fn new(arr: [T; THREADS]) -> Self {
        Self {
            arr: UnsafeCell::new(arr),
        }
    }

    pub fn get(&self, i: usize) -> &T {
        unsafe { &(&*self.arr.get())[i] }
    }

    pub fn to_mut(&self, i: usize) -> &mut T {
        unsafe { &mut (&mut *self.arr.get())[i] }
    }
}

impl<T: ManualCopy<T>> TrustCell<T> {
    pub fn inner_manual_copy(&self, from: usize, to: usize) {
        unsafe {
            (&mut *self.arr.get())[to].copy_from(&mut (&mut *self.arr.get())[from]);
        }
    }

    pub fn inner_manual_clear(&self, to: usize) {
        unsafe {
            (&mut *self.arr.get())[to].clear();
        }
    }
}
