use super::*;

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
            (&mut *self.arr.get())[to].copy_from(&(&*self.arr.get())[from]);
        }
    }
}