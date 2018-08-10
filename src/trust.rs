use std::ops::Deref;

pub struct Trust<T> {
    inner: T,
}

unsafe impl<T> Send for Trust<T> {}
unsafe impl<T> Sync for Trust<T> {}

impl<T> Deref for Trust<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: Clone> Clone for Trust<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}

impl<T> Trust<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
        }
    }
}
